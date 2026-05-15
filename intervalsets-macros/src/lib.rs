//! Compile-time-checked interval literal macros for the
//! [`intervalsets`](https://docs.rs/intervalsets) family of crates.
//!
//! This crate provides two procedural macros:
//!
//! - [`interval!`] — produces an `intervalsets::Interval<T>`.
//! - [`enum_interval!`] — produces an `intervalsets_core::EnumInterval<T>`
//!   (no-std / no-alloc friendly).
//!
//! Both macros accept a single string literal in the same grammar as
//! the runtime
//! [`FromStr`](https://docs.rs/intervalsets-core/latest/intervalsets_core/sets/enum.EnumInterval.html#impl-FromStr-for-EnumInterval%3CT%3E)
//! impl. Malformed input fails to **build** instead of panicking at
//! runtime. Bound expressions inside the string literal are tokenized
//! as Rust source — numeric literals work, but so do arbitrary
//! expressions like `BigDecimal::from(0)`.
//!
//! The macros are re-exported from their respective parent crates;
//! depend on those crates directly rather than on `intervalsets-macros`.
//!
//! # Grammar
//!
//! | Form           | Example         | Constructor                          |
//! |----------------|-----------------|--------------------------------------|
//! | empty          | `{}`            | `empty()`                            |
//! | closed-closed  | `[a, b]`        | `closed(a, b)`                       |
//! | open-open      | `(a, b)`        | `open(a, b)`                         |
//! | closed-open    | `[a, b)`        | `closed_open(a, b)`                  |
//! | open-closed    | `(a, b]`        | `open_closed(a, b)`                  |
//! | closed-unbound | `[a, ..)`       | `closed_unbound(a)`                  |
//! | open-unbound   | `(a, ..)`       | `open_unbound(a)`                    |
//! | unbound-closed | `(.., b]`       | `unbound_closed(b)`                  |
//! | unbound-open   | `(.., b)`       | `unbound_open(b)`                    |
//! | unbounded      | `(.., ..)`      | `unbounded()`                        |
//!
//! The unbounded side **must** use an open delimiter — `[.., x]` is a
//! compile error, mirroring the runtime parser's behavior (and
//! catching it earlier).
//!
//! # Compile-time checks
//!
//! These conditions fail at build time via `compile_error!`:
//!
//! - The macro argument isn't a single string literal.
//! - The string isn't one of the grammar forms above (bracket mismatch, missing comma, etc.).
//! - A closed bracket appears on an unbounded side (`[.., x]`, `[0, ..]`, …).
//! - A non-`{}` set form (`{[0, 5], [10, 15]}`) — that's `IntervalSet` syntax,
//!   which neither this macro nor the runtime `FromStr` accepts.
//! - A bound body fails to parse as a Rust expression.
//! - Both bounds are numeric literals (optionally with unary `-`) and `lhs > rhs`.
//!   Non-literal bounds (identifiers, function calls, casts) fall through to a
//!   runtime panic, matching the existing panicking factory methods.
//!
//! # Caveats
//!
//! - The macro input must be a **string literal**, not a `const` or an
//!   identifier. Procedural macros only see source tokens.
//! - The macro is a compile-time *syntactic* check; the produced
//!   expression is still a runtime constructor call. It is not a `const`
//!   constructor — factory methods aren't `const fn`.
//! - The comma splitter in bound bodies tracks `()`, `[]`, `{}`, and
//!   string/char literals (via token-stream parsing), so expressions
//!   like `[(1, 2).0, 10]` work. The one rare exception is a turbofish
//!   with a top-level comma (`Vec::<i32, A>::new()`); avoid those
//!   inside an interval literal.

extern crate proc_macro;

mod cross;
mod expand;
mod shape;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::Ident;

use crate::expand::Paths;
use crate::shape::parse_shape;

/// Compile-time-checked literal for [`intervalsets::Interval<T>`](https://docs.rs/intervalsets/latest/intervalsets/struct.Interval.html).
///
/// Accepts a single string literal in the interval grammar (see the
/// crate-level docs for the full table) and expands to a constructor
/// call. Bound bodies are arbitrary Rust expressions tokenized at the
/// macro call site.
///
/// ```ignore
/// use intervalsets::prelude::*;
///
/// let a: Interval<i32> = interval!("[0, 10]");
/// let b: Interval<f64> = interval!("(0.0, 10.0)");
/// let c: Interval<i32> = interval!("[0, ..)");
/// let d: Interval<i32> = interval!("(.., ..)");
/// let e: Interval<i32> = interval!("{}");
/// ```
///
/// Malformed input produces a build error:
///
/// ```compile_fail
/// use intervalsets::interval;
/// let _: intervalsets::Interval<i32> = interval!("[10, 0]"); // crossed bounds
/// ```
#[proc_macro]
pub fn interval(input: TokenStream) -> TokenStream {
    expand_entry(input, Target::Interval)
}

/// Compile-time-checked literal for [`intervalsets_core::EnumInterval<T>`](https://docs.rs/intervalsets-core/latest/intervalsets_core/sets/enum.EnumInterval.html).
///
/// Accepts a single string literal in the interval grammar (see the
/// crate-level docs for the full table) and expands to a constructor
/// call on `EnumInterval`. Suitable for `no_std` / `no_alloc`
/// callers.
///
/// ```ignore
/// use intervalsets_core::prelude::*;
///
/// let a: EnumInterval<i32> = enum_interval!("[0, 10]");
/// let b: EnumInterval<f64> = enum_interval!("(.., 10.5]");
/// ```
#[proc_macro]
pub fn enum_interval(input: TokenStream) -> TokenStream {
    expand_entry(input, Target::EnumInterval)
}

#[derive(Copy, Clone)]
enum Target {
    Interval,
    EnumInterval,
}

fn expand_entry(input: TokenStream, target: Target) -> TokenStream {
    match build(input.into(), target) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn build(input: TokenStream2, target: Target) -> syn::Result<TokenStream2> {
    let lit = syn::parse2::<syn::LitStr>(input).map_err(|e| {
        syn::Error::new(
            e.span(),
            "expected a single string literal, e.g. `interval!(\"[0, 10)\")`",
        )
    })?;

    let span = lit.span();
    let s = lit.value();

    let form = parse_shape(&s).map_err(|e| syn::Error::new(span, e.message()))?;

    let paths = paths_for(target);

    expand::build(form, &paths, span)
}

fn paths_for(target: Target) -> Paths {
    match target {
        Target::Interval => {
            let root = resolve_crate("intervalsets");
            Paths {
                type_path: quote!(#root::Interval),
                crate_root: root,
            }
        }
        Target::EnumInterval => {
            let root = resolve_crate("intervalsets-core");
            Paths {
                type_path: quote!(#root::EnumInterval),
                crate_root: root,
            }
        }
    }
}

fn resolve_crate(orig_name: &str) -> TokenStream2 {
    let canonical = Ident::new(&orig_name.replace('-', "_"), Span::call_site());
    match crate_name(orig_name) {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        // `Itself` fires for doctests in the same crate and for integration
        // tests in `<crate>/tests/`. Rustdoc and Cargo both make the
        // canonical crate name addressable in those contexts, but `crate`
        // does not refer to the documented crate. Emit canonical.
        // `Err` fires when the manifest can't be read — same fallback.
        Ok(FoundCrate::Itself) | Err(_) => quote!(::#canonical),
    }
}

//! Compile-time-checked interval literal macros for the
//! [`intervalsets`](https://docs.rs/intervalsets) family of crates.
//!
//! This crate provides three procedural macros:
//!
//! - [`interval!`] — produces an `intervalsets::Interval<T>`.
//! - [`set!`] — produces an `intervalsets::IntervalSet<T>` (multi-piece).
//! - [`enum_interval!`] — produces an `intervalsets_core::EnumInterval<T>`
//!   (no-std / no-alloc friendly).
//!
//! Both macros accept a string literal in the same grammar as the
//! runtime
//! [`FromStr`](https://docs.rs/intervalsets-core/latest/intervalsets_core/sets/enum.EnumInterval.html#impl-FromStr-for-EnumInterval%3CT%3E)
//! impl, with an optional second argument supplying the storage type
//! as a turbofish hint. Malformed input fails to **build** instead of
//! panicking at runtime. Bound expressions inside the string literal
//! are tokenized as Rust source — numeric literals work, but so do
//! arbitrary expressions like `BigDecimal::from(0)`.
//!
//! ```ignore
//! interval!("[0, 10)")            // → Interval<_>, T inferred from context
//! interval!("[0, 10)", i32)       // → Interval::<i32>::closed_open(0, 10)
//! interval!("(.., ..)", f64)      // → resolves T for forms with no T-bearing arg
//! ```
//!
//! # Storage-type coercion
//!
//! When the second argument is supplied, every bound expression is
//! wrapped in `<T as From<_>>::from(expr)`. Existing call sites keep
//! compiling via the reflexive `From<T> for T` blanket impl. The
//! noteworthy effect is on heterogeneous bounds:
//!
//! ```ignore
//! struct Unit(f64);
//! impl From<f64> for Unit { fn from(v: f64) -> Self { Self(v) } }
//!
//! enum_interval!("[0.0, 1.0]", Unit)   // → Unit::from(0.0), Unit::from(1.0)
//! enum_interval!("[0, 1]", Unit)       // compile error: no From<i32> for Unit
//! interval!("[0_i32, 10_i32]", f64)    // works: From<i32> for f64
//! ```
//!
//! The wrap uses fully-qualified UFCS so an inherent `from` method on
//! the user's type can't accidentally shadow the trait method.
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
//! - The macro arguments aren't `"<literal>"` or `"<literal>", <Type>`.
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
mod set;
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
/// Accepts a string literal in the interval grammar (see the
/// crate-level docs for the full table) and expands to a constructor
/// call. An optional second argument supplies a storage-type hint
/// emitted as a turbofish on the constructor — useful when there's no
/// T-bearing argument to infer from (`{}`, `(.., ..)`). When the hint
/// is present each bound expression is additionally wrapped in
/// `<T as From<_>>::from(...)`, so any type with `T: From<U>` can be
/// used as a bound source. Bound bodies are arbitrary Rust expressions
/// tokenized at the macro call site.
///
/// ```ignore
/// use intervalsets::prelude::*;
///
/// let a: Interval<i32> = interval!("[0, 10]");
/// let b: Interval<f64> = interval!("(0.0, 10.0)");
/// let c: Interval<i32> = interval!("[0, ..)");
///
/// // With a storage-type hint (no ascription required):
/// let d = interval!("(.., ..)", i32);
/// let e = interval!("{}", f64);
///
/// // Hint also drives `From`-conversion on each bound:
/// let f = interval!("[0_i32, 10_i32]", f64);  // From<i32> for f64
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
/// Accepts a string literal in the interval grammar (see the
/// crate-level docs for the full table) and expands to a constructor
/// call on `EnumInterval`. An optional second argument supplies a
/// storage-type hint as a turbofish; when present each bound is also
/// wrapped in `<T as From<_>>::from(...)`. Suitable for `no_std` /
/// `no_alloc` callers.
///
/// ```ignore
/// use intervalsets_core::prelude::*;
///
/// let a: EnumInterval<i32> = enum_interval!("[0, 10]");
/// let b: EnumInterval<f64> = enum_interval!("(.., 10.5]");
/// let c = enum_interval!("(.., ..)", i32);
///
/// // Hint coerces bound sources via `From`:
/// struct Unit(f64);
/// impl From<f64> for Unit { fn from(v: f64) -> Self { Self(v) } }
/// // ... Element impl elided ...
/// let d = enum_interval!("[0.0, 1.0]", Unit);
/// ```
#[proc_macro]
pub fn enum_interval(input: TokenStream) -> TokenStream {
    expand_entry(input, Target::EnumInterval)
}

/// Compile-time-checked literal for [`intervalsets::IntervalSet<T>`](https://docs.rs/intervalsets/latest/intervalsets/struct.IntervalSet.html).
///
/// Accepts a string literal in the multi-piece set grammar, matching
/// the runtime `Display` form for `IntervalSet`:
///
/// - `{}` — empty set.
/// - `{[0, 10]}` — single-piece set.
/// - `{[0, 5] U [10, 15] U [20, 30]}` — multi-piece, ASCII `U`
///   separator with whitespace on both sides.
///
/// Each piece is a valid interval per the [`interval!`] grammar. The
/// macro emits a chain of `Interval::ctor(...).union(...)` calls — the
/// resulting `IntervalSet<T>` is normalized (sorted, merged) by the
/// existing `Union::union` machinery, so pieces don't need to be
/// sorted, non-overlapping, or non-empty in source.
///
/// An optional second argument supplies a storage-type hint as a
/// turbofish on the **first** piece (subsequent pieces inherit `T`
/// through type inference on `Union::union`).
///
/// ```ignore
/// use intervalsets::prelude::*;
///
/// let a: IntervalSet<i32> = set!("{}");
/// let b: IntervalSet<i32> = set!("{[0, 10]}");
/// let c = set!("{[0, 1] U (10, 24) U [20, 35)}", i32);
/// ```
#[proc_macro]
pub fn set(input: TokenStream) -> TokenStream {
    match build_set(input.into()) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
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

struct MacroInput {
    lit: syn::LitStr,
    ty: Option<syn::Type>,
}

impl syn::parse::Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: syn::LitStr = input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                "expected a string literal followed by an optional type, \
                 e.g. `interval!(\"[0, 10)\")` or `interval!(\"[0, 10)\", i32)`",
            )
        })?;
        let ty = if input.peek(syn::Token![,]) {
            let _: syn::Token![,] = input.parse()?;
            if input.is_empty() {
                return Err(syn::Error::new(input.span(), "expected a type after `,`"));
            }
            Some(input.parse::<syn::Type>()?)
        } else {
            None
        };
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "unexpected tokens after macro arguments; \
                 expected at most `\"<literal>\", <Type>`",
            ));
        }
        Ok(MacroInput { lit, ty })
    }
}

fn build(input: TokenStream2, target: Target) -> syn::Result<TokenStream2> {
    let MacroInput { lit, ty } = syn::parse2::<MacroInput>(input)?;

    let span = lit.span();
    let s = lit.value();

    let form = parse_shape(&s).map_err(|e| syn::Error::new(span, e.message()))?;

    let paths = paths_for(target, ty);

    expand::build(form, &paths, span)
}

fn build_set(input: TokenStream2) -> syn::Result<TokenStream2> {
    let MacroInput { lit, ty } = syn::parse2::<MacroInput>(input)?;
    let span = lit.span();
    let s = lit.value();

    let parts = crate::set::parse_set(&s).map_err(|e| syn::Error::new(span, e.message()))?;

    let root = resolve_crate("intervalsets");

    // Pre-compute the `IntervalSet`-targeted turbofish before consuming `ty`
    // into the first piece's Paths. Used by the empty and single-piece arms;
    // the multi-piece arm doesn't need it.
    let qualified_set = match &ty {
        Some(t) => quote!(#root::IntervalSet::<#t>),
        None => quote!(#root::IntervalSet),
    };

    if parts.pieces.is_empty() {
        return Ok(quote!(#qualified_set::empty()));
    }

    let mut pieces_iter = parts.pieces.into_iter();
    let first_form = pieces_iter.next().expect("non-empty pieces");
    // Every piece carries the type hint so each piece's bound
    // expressions get the `<T as From<_>>::from(...)` coercion wrap;
    // `Union::union`'s signature would propagate `T` from the first
    // piece, but the coercion happens per-bound and needs the hint at
    // each call site.
    let piece_paths = Paths {
        crate_root: root.clone(),
        type_path: quote!(#root::Interval),
        type_param: ty,
    };
    let first_ctor = expand::build_ctor(first_form, &piece_paths, span)?;

    let mut rest_ctors: Vec<TokenStream2> = Vec::new();
    for form in pieces_iter {
        rest_ctors.push(expand::build_ctor(form, &piece_paths, span)?);
    }

    let body = if rest_ctors.is_empty() {
        // Single piece — coerce to IntervalSet via From.
        quote!(#qualified_set::from(#first_ctor))
    } else {
        // Multi-piece — chain unions. Interval::union(Interval) → IntervalSet<T>,
        // then IntervalSet::union(Interval) → IntervalSet<T>.
        quote!(#first_ctor #( .union(#rest_ctors) )*)
    };

    Ok(quote!({
        #[allow(unused_imports)]
        use #root::factory::traits::*;
        #[allow(unused_imports)]
        use #root::ops::Union;
        #body
    }))
}

fn paths_for(target: Target, type_param: Option<syn::Type>) -> Paths {
    match target {
        Target::Interval => {
            let root = resolve_crate("intervalsets");
            Paths {
                type_path: quote!(#root::Interval),
                crate_root: root,
                type_param,
            }
        }
        Target::EnumInterval => {
            let root = resolve_crate("intervalsets-core");
            Paths {
                type_path: quote!(#root::EnumInterval),
                crate_root: root,
                type_param,
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

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::MacroInput;

    #[test]
    fn parses_lit_only() {
        let mi: MacroInput = syn::parse2(quote!("[0, 10]")).unwrap();
        assert_eq!(mi.lit.value(), "[0, 10]");
        assert!(mi.ty.is_none());
    }

    #[test]
    fn parses_lit_with_type() {
        let mi: MacroInput = syn::parse2(quote!("[0, 10]", i32)).unwrap();
        assert_eq!(mi.lit.value(), "[0, 10]");
        assert!(mi.ty.is_some());
    }

    #[test]
    fn parses_lit_with_generic_type() {
        let mi: MacroInput = syn::parse2(quote!("[0, 10]", core::num::Wrapping<i32>)).unwrap();
        assert!(mi.ty.is_some());
    }

    #[test]
    fn parses_underscore_placeholder() {
        let mi: MacroInput = syn::parse2(quote!("[0, 10]", _)).unwrap();
        assert!(mi.ty.is_some());
    }

    #[test]
    fn rejects_trailing_comma() {
        assert!(syn::parse2::<MacroInput>(quote!("[0, 10]",)).is_err());
    }

    #[test]
    fn rejects_third_arg() {
        assert!(syn::parse2::<MacroInput>(quote!("[0, 10]", i32, 0)).is_err());
    }

    #[test]
    fn rejects_non_string_literal_first() {
        assert!(syn::parse2::<MacroInput>(quote!(0, 10)).is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(syn::parse2::<MacroInput>(quote!()).is_err());
    }
}

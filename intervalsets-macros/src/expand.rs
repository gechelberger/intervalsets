//! Expansion: take a classified [`Form`] plus a target type's
//! [`Paths`] and emit the constructor call.
//!
//! Each emitted expression is wrapped in a block that brings the
//! factory traits into scope locally:
//!
//! ```ignore
//! {
//!     use ::intervalsets::factory::traits::*;
//!     ::intervalsets::Interval::closed_open(0, 10)
//! }
//! ```
//!
//! The block keeps the trait imports from leaking into the caller's
//! scope. `#[allow(unused_imports)]` silences the lint for forms that
//! only need one of the traits (`empty()` for instance hits the
//! inherent method, leaving the imports formally unused).
//!
//! When a storage-type hint is supplied, each bound expression is
//! wrapped in `<T as ::core::convert::From<_>>::from(expr)`. The
//! reflexive `From<T> for T` blanket impl keeps already-T arguments
//! working; mismatched source types now succeed when `T: From<U>` and
//! fail with a missing-`From`-impl error otherwise. UFCS is used
//! instead of the shorthand `T::from(expr)` to avoid shadowing by an
//! inherent `from` method on the user's type.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Result};

use crate::cross::detect_crossed;
use crate::shape::Form;

/// Crate / type path tokens for the macro to emit.
pub(crate) struct Paths {
    /// Path to the parent crate (e.g. `::intervalsets`,
    /// `::intervalsets_core`). Used to import factory traits.
    pub crate_root: TokenStream,
    /// Fully-qualified target type (e.g. `::intervalsets::Interval`,
    /// `::intervalsets_core::EnumInterval`).
    pub type_path: TokenStream,
    /// Optional storage-type hint. When `Some(T)`, every emitted
    /// factory call is qualified as `type_path::<T>::ctor(...)`,
    /// forcing rustc to use `T` and disabling inference at the
    /// call site. Each bound expression is also coerced via
    /// `<T as From<_>>::from(expr)`.
    pub type_param: Option<syn::Type>,
}

pub(crate) fn build(form: Form, paths: &Paths, span: Span) -> Result<TokenStream> {
    let ctor = build_ctor(form, paths, span)?;
    let crate_root = &paths.crate_root;
    Ok(quote!({
        #[allow(unused_imports)]
        use #crate_root::factory::traits::*;
        #ctor
    }))
}

/// Build just the constructor call (no surrounding block / trait imports).
/// Used by `interval!` (wrapped in a block) and by `set!` (chained with `.union(...)`).
pub(crate) fn build_ctor(form: Form, paths: &Paths, span: Span) -> Result<TokenStream> {
    let Paths {
        crate_root: _,
        type_path,
        type_param,
    } = paths;

    let qualified = match type_param {
        Some(t) => quote!(#type_path::<#t>),
        None => quote!(#type_path),
    };

    let wrap = |e: &Expr| -> TokenStream {
        match type_param {
            Some(t) => quote!(<#t as ::core::convert::From<_>>::from(#e)),
            None => quote!(#e),
        }
    };

    Ok(match form {
        Form::Empty => quote!(#qualified::empty()),
        Form::Unbounded => quote!(#qualified::unbounded()),
        Form::Closed(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            let (lhs, rhs) = (wrap(&lhs), wrap(&rhs));
            quote!(#qualified::closed(#lhs, #rhs))
        }
        Form::Open(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            let (lhs, rhs) = (wrap(&lhs), wrap(&rhs));
            quote!(#qualified::open(#lhs, #rhs))
        }
        Form::ClosedOpen(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            let (lhs, rhs) = (wrap(&lhs), wrap(&rhs));
            quote!(#qualified::closed_open(#lhs, #rhs))
        }
        Form::OpenClosed(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            let (lhs, rhs) = (wrap(&lhs), wrap(&rhs));
            quote!(#qualified::open_closed(#lhs, #rhs))
        }
        Form::ClosedUnbound(l) => {
            let lhs = body_expr(l, span)?;
            let lhs = wrap(&lhs);
            quote!(#qualified::closed_unbound(#lhs))
        }
        Form::OpenUnbound(l) => {
            let lhs = body_expr(l, span)?;
            let lhs = wrap(&lhs);
            quote!(#qualified::open_unbound(#lhs))
        }
        Form::UnboundClosed(r) => {
            let rhs = body_expr(r, span)?;
            let rhs = wrap(&rhs);
            quote!(#qualified::unbound_closed(#rhs))
        }
        Form::UnboundOpen(r) => {
            let rhs = body_expr(r, span)?;
            let rhs = wrap(&rhs);
            quote!(#qualified::unbound_open(#rhs))
        }
    })
}

fn parse_pair(l: TokenStream, r: TokenStream, span: Span) -> Result<(Expr, Expr)> {
    let lhs = body_expr(l, span)?;
    let rhs = body_expr(r, span)?;
    if let Some(msg) = detect_crossed(&lhs, &rhs) {
        return Err(syn::Error::new(span, msg));
    }
    Ok((lhs, rhs))
}

fn body_expr(ts: TokenStream, span: Span) -> Result<Expr> {
    let rendered = ts.to_string();
    syn::parse2::<Expr>(ts).map_err(|e| {
        syn::Error::new(
            span,
            format!("failed to parse `{rendered}` as a Rust expression: {e}"),
        )
    })
}

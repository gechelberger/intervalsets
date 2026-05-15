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
}

pub(crate) fn build(form: Form, paths: &Paths, span: Span) -> Result<TokenStream> {
    let Paths {
        crate_root,
        type_path,
    } = paths;

    let body = match form {
        Form::Empty => quote!(#type_path::empty()),
        Form::Unbounded => quote!(#type_path::unbounded()),
        Form::Closed(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            quote!(#type_path::closed(#lhs, #rhs))
        }
        Form::Open(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            quote!(#type_path::open(#lhs, #rhs))
        }
        Form::ClosedOpen(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            quote!(#type_path::closed_open(#lhs, #rhs))
        }
        Form::OpenClosed(l, r) => {
            let (lhs, rhs) = parse_pair(l, r, span)?;
            quote!(#type_path::open_closed(#lhs, #rhs))
        }
        Form::ClosedUnbound(l) => {
            let lhs = body_expr(l, span)?;
            quote!(#type_path::closed_unbound(#lhs))
        }
        Form::OpenUnbound(l) => {
            let lhs = body_expr(l, span)?;
            quote!(#type_path::open_unbound(#lhs))
        }
        Form::UnboundClosed(r) => {
            let rhs = body_expr(r, span)?;
            quote!(#type_path::unbound_closed(#rhs))
        }
        Form::UnboundOpen(r) => {
            let rhs = body_expr(r, span)?;
            quote!(#type_path::unbound_open(#rhs))
        }
    };

    Ok(quote!({
        #[allow(unused_imports)]
        use #crate_root::factory::traits::*;
        #body
    }))
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

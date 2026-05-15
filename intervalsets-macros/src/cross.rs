//! Best-effort compile-time crossed-bound detection.
//!
//! When both bounds of an interval literal are numeric-literal
//! expressions (with an optional leading unary `-`), we can compare
//! their values at expansion time and reject the input via
//! `compile_error!` if `lhs > rhs`. Any other bound shape (suffixed
//! literals' value side is still recognized via `base10_parse`,
//! identifiers, function calls, casts) falls through; crossed bounds
//! in those cases panic at runtime via the panicking factory methods.
//!
//! The detector is intentionally conservative — false positives would
//! reject inputs the runtime accepts.

use syn::{Expr, ExprLit, ExprUnary, Lit, UnOp};

/// Returns the formatted error message when `lhs > rhs` is detectable
/// at compile time, or `None` otherwise.
pub(crate) fn detect_crossed(lhs: &Expr, rhs: &Expr) -> Option<String> {
    if let (Some(l), Some(r)) = (as_i128(lhs), as_i128(rhs)) {
        if l > r {
            return Some(format!("interval bounds are crossed: {l} > {r}"));
        }
        return None;
    }
    if let (Some(l), Some(r)) = (as_f64(lhs), as_f64(rhs)) {
        // NaN cannot appear as a Rust numeric literal, so `>` is well-defined here.
        if l > r {
            return Some(format!("interval bounds are crossed: {l} > {r}"));
        }
        return None;
    }
    None
}

fn as_i128(expr: &Expr) -> Option<i128> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<i128>().ok(),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => match &**expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit), ..
            }) => lit.base10_parse::<i128>().ok().map(|v| -v),
            _ => None,
        },
        _ => None,
    }
}

fn as_f64(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Float(lit),
            ..
        }) => lit.base10_parse::<f64>().ok(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<i128>().ok().map(|v| v as f64),
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => match &**expr {
            Expr::Lit(ExprLit {
                lit: Lit::Float(lit),
                ..
            }) => lit.base10_parse::<f64>().ok().map(|v| -v),
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit), ..
            }) => lit.base10_parse::<i128>().ok().map(|v| -(v as f64)),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Expr {
        syn::parse_str(s).unwrap()
    }

    #[test]
    fn crossed_ints() {
        assert!(detect_crossed(&parse("10"), &parse("0")).is_some());
        assert!(detect_crossed(&parse("0"), &parse("10")).is_none());
        assert!(detect_crossed(&parse("0"), &parse("0")).is_none());
    }

    #[test]
    fn crossed_negative_ints() {
        assert!(detect_crossed(&parse("-1"), &parse("-10")).is_some());
        assert!(detect_crossed(&parse("-10"), &parse("-1")).is_none());
        assert!(detect_crossed(&parse("1"), &parse("-1")).is_some());
    }

    #[test]
    fn crossed_suffixed_ints() {
        // Suffixed literals work — the suffix is metadata, base10_parse strips it.
        assert!(detect_crossed(&parse("10_i32"), &parse("0_i32")).is_some());
        assert!(detect_crossed(&parse("10u8"), &parse("0u8")).is_some());
    }

    #[test]
    fn crossed_hex() {
        // 0xFF = 255, 0x10 = 16. base10_parse converts any base correctly.
        assert!(detect_crossed(&parse("0xFF"), &parse("0x10")).is_some());
    }

    #[test]
    fn crossed_floats() {
        assert!(detect_crossed(&parse("1.5"), &parse("0.5")).is_some());
        assert!(detect_crossed(&parse("0.5"), &parse("1.5")).is_none());
        assert!(detect_crossed(&parse("-0.5"), &parse("-1.5")).is_some());
    }

    #[test]
    fn mixed_int_float_uses_f64() {
        // Mixed comparisons promote int to f64. 5 < 5.5 → not crossed.
        assert!(detect_crossed(&parse("5"), &parse("5.5")).is_none());
        // 6 > 5.5 → crossed.
        assert!(detect_crossed(&parse("6"), &parse("5.5")).is_some());
    }

    #[test]
    fn skips_non_literal_expressions() {
        // Identifiers, function calls, casts — not detectable. Falls
        // through to runtime panic in the panicking factory.
        assert!(detect_crossed(&parse("x"), &parse("y")).is_none());
        assert!(detect_crossed(&parse("foo(10)"), &parse("foo(0)")).is_none());
        assert!(detect_crossed(&parse("5 as i32"), &parse("0 as i32")).is_none());
        assert!(detect_crossed(&parse("a + b"), &parse("c + d")).is_none());
    }

    #[test]
    fn skips_overflow_literals() {
        // i128::MAX + 1 won't parse as i128 — detector skips it.
        let too_big = "170141183460469231731687303715884105728"; // i128::MAX + 1
        assert!(detect_crossed(&parse(too_big), &parse("0")).is_none());
    }
}

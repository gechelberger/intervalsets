//! End-to-end tests for `interval!`. For each grammar form,
//! the macro output must equal both the runtime `FromStr` result and
//! the hand-written factory call.

use intervalsets::prelude::*;

#[test]
fn empty() {
    let m: Interval<i32> = interval!("{}");
    let s: Interval<i32> = "{}".parse().unwrap();
    let f = Interval::<i32>::empty();
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn closed_int() {
    let m: Interval<i32> = interval!("[0, 10]");
    let s: Interval<i32> = "[0, 10]".parse().unwrap();
    let f = Interval::closed(0, 10);
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn open_float() {
    let m: Interval<f64> = interval!("(0.0, 10.0)");
    let s: Interval<f64> = "(0.0, 10.0)".parse().unwrap();
    let f = Interval::open(0.0_f64, 10.0);
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn closed_open() {
    let m: Interval<i32> = interval!("[0, 10)");
    let f = Interval::closed_open(0, 10);
    assert_eq!(m, f);
}

#[test]
fn open_closed() {
    let m: Interval<i32> = interval!("(0, 10]");
    let f = Interval::open_closed(0, 10);
    assert_eq!(m, f);
}

#[test]
fn closed_unbound() {
    let m: Interval<i32> = interval!("[0, ..)");
    let f = Interval::closed_unbound(0);
    assert_eq!(m, f);
}

#[test]
fn open_unbound() {
    let m: Interval<i32> = interval!("(0, ..)");
    let f = Interval::open_unbound(0);
    assert_eq!(m, f);
}

#[test]
fn unbound_closed() {
    let m: Interval<i32> = interval!("(.., 10]");
    let f = Interval::unbound_closed(10);
    assert_eq!(m, f);
}

#[test]
fn unbound_open() {
    let m: Interval<i32> = interval!("(.., 10)");
    let f = Interval::unbound_open(10);
    assert_eq!(m, f);
}

#[test]
fn unbounded() {
    let m: Interval<i32> = interval!("(.., ..)");
    let f = Interval::<i32>::unbounded();
    assert_eq!(m, f);
}

#[test]
fn negative_literal_bounds() {
    let m: Interval<i32> = interval!("[-10, -1]");
    let f = Interval::closed(-10, -1);
    assert_eq!(m, f);
}

#[test]
fn whitespace_lenience() {
    let m: Interval<i32> = interval!("  [ 0 , 10 ]  ");
    let f = Interval::closed(0, 10);
    assert_eq!(m, f);
}

#[test]
fn non_literal_bound_expression() {
    // The macro tokenizes the bound body as a Rust expression —
    // it does not invoke `T::from_str`.
    let n: i32 = 5;
    let m: Interval<i32> = interval!("[n, n + 10]");
    let f = Interval::closed(5, 15);
    assert_eq!(m, f);
}

#[test]
#[should_panic]
fn crossed_bounds_at_runtime_for_non_literal() {
    // Crossed-bound compile-time detection only fires for numeric
    // literals; non-literal bounds still hit the panicking factory.
    let hi: i32 = 10;
    let lo: i32 = 0;
    let _x: Interval<i32> = interval!("[hi, lo]");
}

// --- Storage-type hint forms ---

#[test]
fn hint_resolves_unbounded_inference() {
    // Without the hint, `let _ = interval!("(.., ..)")` can't infer T.
    let x = interval!("(.., ..)", i32);
    assert_eq!(x, Interval::<i32>::unbounded());
}

#[test]
fn hint_resolves_empty_inference() {
    let x = interval!("{}", f64);
    assert_eq!(x, Interval::<f64>::empty());
}

#[test]
fn hint_pins_float_width() {
    // Float literals default to f64; the hint pins them to f32.
    let x = interval!("[0.0, 10.0]", f32);
    assert_eq!(x, Interval::<f32>::closed(0.0, 10.0));
}

#[test]
fn hint_works_with_half_unbounded() {
    let x = interval!("(.., 10]", i32);
    assert_eq!(x, Interval::unbound_closed(10));
}

#[test]
fn hint_accepts_underscore_placeholder() {
    // `_` is a valid syn::Type and means "infer", same as omitting the hint.
    let x: Interval<i32> = interval!("[0, 10]", _);
    assert_eq!(x, Interval::closed(0, 10));
}

#[test]
fn hint_accepts_generic_type() {
    use core::num::Saturating;
    let x = interval!("[Saturating(0_i32), Saturating(10_i32)]", Saturating<i32>);
    assert_eq!(x, Interval::closed(Saturating(0_i32), Saturating(10)));
}

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

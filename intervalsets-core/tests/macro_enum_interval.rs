//! End-to-end tests for `enum_interval!`. For each grammar form,
//! the macro output must equal both the runtime `FromStr` result and
//! the hand-written factory call.

use intervalsets_core::prelude::*;

#[test]
fn empty() {
    let m: EnumInterval<i32> = enum_interval!("{}");
    let s: EnumInterval<i32> = "{}".parse().unwrap();
    let f: EnumInterval<i32> = EnumInterval::empty();
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn closed_int() {
    let m: EnumInterval<i32> = enum_interval!("[0, 10]");
    let s: EnumInterval<i32> = "[0, 10]".parse().unwrap();
    let f = EnumInterval::closed(0, 10);
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn open_float() {
    let m: EnumInterval<f64> = enum_interval!("(0.0, 10.0)");
    let s: EnumInterval<f64> = "(0.0, 10.0)".parse().unwrap();
    let f = EnumInterval::open(0.0_f64, 10.0);
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn closed_open() {
    let m: EnumInterval<i32> = enum_interval!("[0, 10)");
    let f = EnumInterval::closed_open(0, 10);
    assert_eq!(m, f);
}

#[test]
fn open_closed() {
    let m: EnumInterval<i32> = enum_interval!("(0, 10]");
    let f = EnumInterval::open_closed(0, 10);
    assert_eq!(m, f);
}

#[test]
fn closed_unbound() {
    let m: EnumInterval<i32> = enum_interval!("[0, ..)");
    let f = EnumInterval::closed_unbound(0);
    assert_eq!(m, f);
}

#[test]
fn open_unbound() {
    let m: EnumInterval<i32> = enum_interval!("(0, ..)");
    let f = EnumInterval::open_unbound(0);
    assert_eq!(m, f);
}

#[test]
fn unbound_closed() {
    let m: EnumInterval<i32> = enum_interval!("(.., 10]");
    let f = EnumInterval::unbound_closed(10);
    assert_eq!(m, f);
}

#[test]
fn unbound_open() {
    let m: EnumInterval<i32> = enum_interval!("(.., 10)");
    let f = EnumInterval::unbound_open(10);
    assert_eq!(m, f);
}

#[test]
fn unbounded() {
    let m: EnumInterval<i32> = enum_interval!("(.., ..)");
    let f = EnumInterval::unbounded();
    assert_eq!(m, f);
}

#[test]
fn negative_literal_bounds() {
    let m: EnumInterval<i32> = enum_interval!("[-10, -1]");
    let f = EnumInterval::closed(-10, -1);
    assert_eq!(m, f);
}

#[test]
fn whitespace_lenience() {
    let m: EnumInterval<i32> = enum_interval!("  [ 0 , 10 ]  ");
    let f = EnumInterval::closed(0, 10);
    assert_eq!(m, f);
}

#[test]
fn non_literal_bound_expression() {
    // Confirms the macro tokenizes the bound body as a Rust expression —
    // it does not invoke `T::from_str`. The runtime FromStr parser
    // would reject `"n"` as a `T::from_str` failure.
    let n: i32 = 5;
    let m: EnumInterval<i32> = enum_interval!("[n, n + 10]");
    let f = EnumInterval::closed(5, 15);
    assert_eq!(m, f);
}

#[test]
#[should_panic]
fn crossed_bounds_at_runtime_for_non_literal() {
    // Crossed bounds are detected at compile time for numeric literals,
    // but the detector skips non-literal expressions. Those still panic
    // at runtime via the panicking factory.
    let hi: i32 = 10;
    let lo: i32 = 0;
    let _x: EnumInterval<i32> = enum_interval!("[hi, lo]");
}

//! End-to-end tests for `set!`. Each form is checked against the
//! runtime `FromStr` result and a hand-written factory chain.

use intervalsets::prelude::*;

#[test]
fn empty() {
    let m: IntervalSet<i32> = set!("{}");
    let s: IntervalSet<i32> = "{}".parse().unwrap();
    let f = IntervalSet::<i32>::empty();
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn empty_with_hint() {
    let m = set!("{}", i32);
    assert_eq!(m, IntervalSet::<i32>::empty());
}

#[test]
fn single_piece() {
    let m: IntervalSet<i32> = set!("{[0, 10]}");
    let s: IntervalSet<i32> = "{[0, 10]}".parse().unwrap();
    let f = IntervalSet::from(Interval::closed(0, 10));
    assert_eq!(m, s);
    assert_eq!(m, f);
}

#[test]
fn single_piece_with_hint() {
    let m = set!("{[0, 10]}", i32);
    let f = IntervalSet::from(Interval::closed(0, 10));
    assert_eq!(m, f);
}

#[test]
fn headline_three_pieces_with_hint() {
    let m = set!("{[0, 1] U (10, 24) U [20, 35)}", i32);
    let f = Interval::closed(0, 1)
        .union(Interval::open(10, 24))
        .union(Interval::closed_open(20, 35));
    assert_eq!(m, f);
}

#[test]
fn three_pieces_no_hint_ascribed() {
    let m: IntervalSet<i32> = set!("{[0, 5] U [10, 15] U [20, 30]}");
    let f = Interval::closed(0, 5)
        .union(Interval::closed(10, 15))
        .union(Interval::closed(20, 30));
    assert_eq!(m, f);
}

#[test]
fn three_way_agreement_with_fromstr() {
    let s = "{[0, 5] U (10, 15] U [20, ..)}";
    let m = set!("{[0, 5] U (10, 15] U [20, ..)}", i32);
    let p: IntervalSet<i32> = s.parse().unwrap();
    let f = Interval::closed(0, 5)
        .union(Interval::open_closed(10, 15))
        .union(Interval::closed_unbound(20));
    assert_eq!(m, p);
    assert_eq!(m, f);
}

#[test]
fn round_trip_via_display() {
    let cases: [IntervalSet<i32>; 4] = [
        IntervalSet::empty(),
        IntervalSet::from(Interval::closed(0, 10)),
        Interval::closed(0, 5).union(Interval::closed(10, 15)),
        Interval::unbound_closed(-10)
            .union(Interval::open(0, 5))
            .union(Interval::closed_unbound(20)),
    ];
    for x in cases {
        let printed = format!("{x}");
        let parsed: IntervalSet<i32> = printed.parse().unwrap();
        assert_eq!(parsed, x, "round-trip failed for {printed}");
    }
}

#[test]
fn non_literal_bound_expressions() {
    // Each piece's bounds are arbitrary Rust expressions, not just literals.
    let n: i32 = 5;
    let m: IntervalSet<i32> = set!("{[n, n + 10] U [n + 20, n + 30]}");
    let f = Interval::closed(5, 15).union(Interval::closed(25, 35));
    assert_eq!(m, f);
}

#[test]
fn unsorted_input_is_normalized() {
    let unsorted = set!("{[10, 20] U [0, 5]}", i32);
    let sorted = set!("{[0, 5] U [10, 20]}", i32);
    assert_eq!(unsorted, sorted);
}

#[test]
fn overlapping_input_is_merged() {
    let overlap = set!("{[0, 10] U [5, 15]}", i32);
    let merged = set!("{[0, 15]}", i32);
    assert_eq!(overlap, merged);
}

#[test]
fn whitespace_lenience() {
    let m = set!("  {  [0, 5]   U   [10, 15]  }  ", i32);
    let f = Interval::closed(0, 5).union(Interval::closed(10, 15));
    assert_eq!(m, f);
}

#[test]
fn float_hint_pins_width() {
    let m = set!("{[0.0, 5.0] U [10.0, 15.0]}", f32);
    let f = Interval::<f32>::closed(0.0, 5.0).union(Interval::<f32>::closed(10.0, 15.0));
    assert_eq!(m, f);
}

//! Phase 3 — `Intersection::intersection` for all 9 (LHS, RHS) pairs
//! at i64.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Intersection;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::closed(kani::any(), kani::any())
}

fn make_half() -> HalfInterval<i64> {
    let at: i64 = kani::any();
    if kani::any() {
        HalfInterval::<i64>::closed_unbound(at)
    } else {
        HalfInterval::<i64>::unbound_closed(at)
    }
}

fn make_enum() -> EnumInterval<i64> {
    let kind: u8 = kani::any();
    kani::assume(kind < 5);
    match kind {
        0 => EnumInterval::<i64>::closed(kani::any(), kani::any()),
        1 => EnumInterval::<i64>::closed_unbound(kani::any()),
        2 => EnumInterval::<i64>::unbound_closed(kani::any()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

#[kani::proof]
fn intersection_finite_finite_i64_no_panic() {
    let _ = make_finite().intersection(make_finite());
}

#[kani::proof]
fn intersection_finite_half_i64_no_panic() {
    let _ = make_finite().intersection(make_half());
}

#[kani::proof]
fn intersection_half_finite_i64_no_panic() {
    let _ = make_half().intersection(make_finite());
}

#[kani::proof]
fn intersection_half_half_i64_no_panic() {
    let _ = make_half().intersection(make_half());
}

#[kani::proof]
fn intersection_finite_enum_i64_no_panic() {
    let _ = make_finite().intersection(make_enum());
}

#[kani::proof]
fn intersection_enum_finite_i64_no_panic() {
    let _ = make_enum().intersection(make_finite());
}

#[kani::proof]
fn intersection_half_enum_i64_no_panic() {
    let _ = make_half().intersection(make_enum());
}

#[kani::proof]
fn intersection_enum_half_i64_no_panic() {
    let _ = make_enum().intersection(make_half());
}

#[kani::proof]
fn intersection_enum_enum_i64_no_panic() {
    let _ = make_enum().intersection(make_enum());
}

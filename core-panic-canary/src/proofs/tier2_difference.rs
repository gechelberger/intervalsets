//! Phase 3 — `Difference::difference` for all 9 (LHS, RHS) pairs at
//! i64.
//!
//! Difference computes `A ∩ B'`, so it inherits Complement's cost on
//! top of Intersection. The pilot `difference_finite_finite` ran in
//! ~24s, so the cross-type harnesses may take similarly long.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Difference;
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
fn difference_finite_finite_i64_no_panic() {
    let _ = make_finite().difference(make_finite());
}

#[kani::proof]
fn difference_finite_half_i64_no_panic() {
    let _ = make_finite().difference(make_half());
}

#[kani::proof]
fn difference_half_finite_i64_no_panic() {
    let _ = make_half().difference(make_finite());
}

#[kani::proof]
fn difference_half_half_i64_no_panic() {
    let _ = make_half().difference(make_half());
}

#[kani::proof]
fn difference_finite_enum_i64_no_panic() {
    let _ = make_finite().difference(make_enum());
}

#[kani::proof]
fn difference_enum_finite_i64_no_panic() {
    let _ = make_enum().difference(make_finite());
}

#[kani::proof]
fn difference_half_enum_i64_no_panic() {
    let _ = make_half().difference(make_enum());
}

#[kani::proof]
fn difference_enum_half_i64_no_panic() {
    let _ = make_enum().difference(make_half());
}

#[kani::proof]
fn difference_enum_enum_i64_no_panic() {
    let _ = make_enum().difference(make_enum());
}

//! Phase 3 — `Union::union` for all 9 (LHS, RHS) pairs at i64.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Union;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::satisfy_bounds(
        FiniteBound::closed(kani::any::<i64>()),
        FiniteBound::closed(kani::any::<i64>()),
    )
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
        0 => EnumInterval::<i64>::satisfy_bounds(
            FiniteBound::closed(kani::any::<i64>()),
            FiniteBound::closed(kani::any::<i64>()),
        ),
        1 => EnumInterval::<i64>::closed_unbound(kani::any()),
        2 => EnumInterval::<i64>::unbound_closed(kani::any()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

#[kani::proof]
fn union_finite_finite_i64_no_panic() {
    let _ = make_finite().union(make_finite());
}

#[kani::proof]
fn union_finite_half_i64_no_panic() {
    let _ = make_finite().union(make_half());
}

#[kani::proof]
fn union_half_finite_i64_no_panic() {
    let _ = make_half().union(make_finite());
}

#[kani::proof]
fn union_half_half_i64_no_panic() {
    let _ = make_half().union(make_half());
}

#[kani::proof]
fn union_finite_enum_i64_no_panic() {
    let _ = make_finite().union(make_enum());
}

#[kani::proof]
fn union_enum_finite_i64_no_panic() {
    let _ = make_enum().union(make_finite());
}

#[kani::proof]
fn union_half_enum_i64_no_panic() {
    let _ = make_half().union(make_enum());
}

#[kani::proof]
fn union_enum_half_i64_no_panic() {
    let _ = make_enum().union(make_half());
}

#[kani::proof]
fn union_enum_enum_i64_no_panic() {
    let _ = make_enum().union(make_enum());
}

//! Phase 2 — `Rebound::try_with_left` / `try_with_right` for all 6
//! impls at i64.
//!
//! Rebound has 6 methods: 3 types × {try_with_left, try_with_right}.
//! No arithmetic on T, so no overflow concerns; inputs are
//! unconstrained.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Rebound;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn any_bound() -> Option<FiniteBound<i64>> {
    if !kani::any::<bool>() {
        return None;
    }
    let v: i64 = kani::any();
    Some(if kani::any() {
        FiniteBound::closed(v)
    } else {
        FiniteBound::open(v)
    })
}

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
fn try_with_left_finite_i64_no_panic() {
    let _ = make_finite().try_with_left(any_bound());
}

#[kani::proof]
fn try_with_right_finite_i64_no_panic() {
    let _ = make_finite().try_with_right(any_bound());
}

#[kani::proof]
fn try_with_left_half_i64_no_panic() {
    let _ = make_half().try_with_left(any_bound());
}

#[kani::proof]
fn try_with_right_half_i64_no_panic() {
    let _ = make_half().try_with_right(any_bound());
}

#[kani::proof]
fn try_with_left_enum_i64_no_panic() {
    let _ = make_enum().try_with_left(any_bound());
}

#[kani::proof]
fn try_with_right_enum_i64_no_panic() {
    let _ = make_enum().try_with_right(any_bound());
}

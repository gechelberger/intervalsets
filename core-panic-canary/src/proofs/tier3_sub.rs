//! Phase 2 — `TrySub` for all 9 monomorphizations at i64.
//!
//! Same input-bounding rationale as `tier3_add`: half-range bounds
//! keep `lmax - rmin` and `lmin - rmax` within i64.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TrySub;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

const HALF_MIN: i64 = i64::MIN / 2;
const HALF_MAX: i64 = i64::MAX / 2;

fn any_bounded() -> i64 {
    let v: i64 = kani::any();
    kani::assume(v >= HALF_MIN && v <= HALF_MAX);
    v
}

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::closed(any_bounded(), any_bounded())
}

fn make_half() -> HalfInterval<i64> {
    let at = any_bounded();
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
        0 => EnumInterval::<i64>::closed(any_bounded(), any_bounded()),
        1 => EnumInterval::<i64>::closed_unbound(any_bounded()),
        2 => EnumInterval::<i64>::unbound_closed(any_bounded()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

#[kani::proof]
fn try_sub_finite_finite_i64_no_panic() {
    let _ = make_finite().try_sub(make_finite());
}

#[kani::proof]
fn try_sub_half_half_i64_no_panic() {
    let _ = make_half().try_sub(make_half());
}

#[kani::proof]
fn try_sub_finite_half_i64_no_panic() {
    let _ = make_finite().try_sub(make_half());
}

#[kani::proof]
fn try_sub_half_finite_i64_no_panic() {
    let _ = make_half().try_sub(make_finite());
}

#[kani::proof]
fn try_sub_enum_finite_i64_no_panic() {
    let _ = make_enum().try_sub(make_finite());
}

#[kani::proof]
fn try_sub_enum_half_i64_no_panic() {
    let _ = make_enum().try_sub(make_half());
}

#[kani::proof]
fn try_sub_enum_enum_i64_no_panic() {
    let _ = make_enum().try_sub(make_enum());
}

#[kani::proof]
fn try_sub_finite_enum_i64_no_panic() {
    let _ = make_finite().try_sub(make_enum());
}

#[kani::proof]
fn try_sub_half_enum_i64_no_panic() {
    let _ = make_half().try_sub(make_enum());
}

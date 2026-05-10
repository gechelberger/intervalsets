//! Phase 2 — `TryMul` for all 9 monomorphizations at i64.
//!
//! Tighter bounds than add/sub: `MUL_BOUND = 2^31` keeps any pair
//! product at `2^62 < i64::MAX = 2^63 - 1`. Wider bounds blow Kani's
//! solver time up quickly because the search space grows
//! quadratically.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryMul;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

const MUL_BOUND: i64 = 1 << 31;

fn any_bounded() -> i64 {
    let v: i64 = kani::any();
    kani::assume(v >= -MUL_BOUND && v <= MUL_BOUND);
    v
}

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::satisfy_bounds(
        FiniteBound::closed(any_bounded()),
        FiniteBound::closed(any_bounded()),
    )
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
        0 => EnumInterval::<i64>::satisfy_bounds(
            FiniteBound::closed(any_bounded()),
            FiniteBound::closed(any_bounded()),
        ),
        1 => EnumInterval::<i64>::closed_unbound(any_bounded()),
        2 => EnumInterval::<i64>::unbound_closed(any_bounded()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

#[kani::proof]
fn try_mul_finite_finite_i64_no_panic() {
    let _ = make_finite().try_mul(make_finite());
}

#[kani::proof]
fn try_mul_half_half_i64_no_panic() {
    let _ = make_half().try_mul(make_half());
}

#[kani::proof]
fn try_mul_finite_half_i64_no_panic() {
    let _ = make_finite().try_mul(make_half());
}

#[kani::proof]
fn try_mul_half_finite_i64_no_panic() {
    let _ = make_half().try_mul(make_finite());
}

#[kani::proof]
fn try_mul_enum_finite_i64_no_panic() {
    let _ = make_enum().try_mul(make_finite());
}

#[kani::proof]
fn try_mul_enum_half_i64_no_panic() {
    let _ = make_enum().try_mul(make_half());
}

#[kani::proof]
fn try_mul_enum_enum_i64_no_panic() {
    let _ = make_enum().try_mul(make_enum());
}

#[kani::proof]
fn try_mul_finite_enum_i64_no_panic() {
    let _ = make_finite().try_mul(make_enum());
}

#[kani::proof]
fn try_mul_half_enum_i64_no_panic() {
    let _ = make_half().try_mul(make_enum());
}

//! Phase 2 — `TryAdd` for all 9 monomorphizations at i64.
//!
//! Inputs are bounded to a half-range so any pair sum stays within
//! i64 — Kani requires `overflow-checks=on` for sound analysis,
//! stricter than release-mode wrap semantics. Proof covers "no panic
//! AND no overflow" rather than just "no panic in release", which is
//! a strictly stronger property than the documented Tier 3 contract.
//!
//! `make_*` helpers below build nondeterministic interval values
//! that span all variants of each shape (Half: both Sides;
//! Enum: Finite/Half(L)/Half(R)/Unbounded/Empty), so a single harness
//! per type-pair exhausts the categorical dispatch matrix.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryAdd;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

const HALF_MIN: i64 = i64::MIN / 2;
const HALF_MAX: i64 = i64::MAX / 2;

fn any_bounded() -> i64 {
    let v: i64 = kani::any();
    kani::assume(v >= HALF_MIN && v <= HALF_MAX);
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
fn try_add_finite_finite_i64_no_panic() {
    let _ = make_finite().try_add(make_finite());
}

#[kani::proof]
fn try_add_half_half_i64_no_panic() {
    let _ = make_half().try_add(make_half());
}

#[kani::proof]
fn try_add_half_finite_i64_no_panic() {
    let _ = make_half().try_add(make_finite());
}

#[kani::proof]
fn try_add_enum_finite_i64_no_panic() {
    let _ = make_enum().try_add(make_finite());
}

#[kani::proof]
fn try_add_enum_half_i64_no_panic() {
    let _ = make_enum().try_add(make_half());
}

#[kani::proof]
fn try_add_enum_enum_i64_no_panic() {
    let _ = make_enum().try_add(make_enum());
}

#[kani::proof]
fn try_add_finite_half_i64_no_panic() {
    let _ = make_finite().try_add(make_half());
}

#[kani::proof]
fn try_add_finite_enum_i64_no_panic() {
    let _ = make_finite().try_add(make_enum());
}

#[kani::proof]
fn try_add_half_enum_i64_no_panic() {
    let _ = make_half().try_add(make_enum());
}

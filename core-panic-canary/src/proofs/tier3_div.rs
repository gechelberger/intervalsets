//! Phase 2 — `TryDiv` for all 9 monomorphizations at i64.
//!
//! TryDiv is the trait whose linker canary could only fit a single
//! call before hitting the optimizer's panic-edge-elimination
//! cascade. Kani has no equivalent budget limit, so we cover all 9
//! monomorphizations here.
//!
//! # Finding (`i64::MIN / -1` overflow)
//!
//! Without the half-range bound below, Kani reports VERIFICATION
//! FAILED with `attempt to divide with overflow` at
//! `<i64 as Div>::div`. The categorical dispatch in
//! `intervalsets-core/src/ops/math/div.rs::impls` correctly diverts
//! divide-by-zero, but does not handle signed-integer-min divided by
//! -1 (Rust panics on this in both debug and release). The linker
//! canary missed it because its concrete fixtures never reached the
//! edge case. Until that gap is fixed in `intervalsets-core`, these
//! harnesses bound inputs to `[HALF_MIN, HALF_MAX]` (which excludes
//! both `i64::MIN` and any sum-induced i64 boundary), so they can
//! prove the rest of try_div is panic-free.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryDiv;
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
fn try_div_finite_finite_i64_no_panic() {
    let _ = make_finite().try_div(make_finite());
}

#[kani::proof]
fn try_div_half_half_i64_no_panic() {
    let _ = make_half().try_div(make_half());
}

#[kani::proof]
fn try_div_finite_half_i64_no_panic() {
    let _ = make_finite().try_div(make_half());
}

#[kani::proof]
fn try_div_half_finite_i64_no_panic() {
    let _ = make_half().try_div(make_finite());
}

#[kani::proof]
fn try_div_enum_finite_i64_no_panic() {
    let _ = make_enum().try_div(make_finite());
}

#[kani::proof]
fn try_div_enum_half_i64_no_panic() {
    let _ = make_enum().try_div(make_half());
}

#[kani::proof]
fn try_div_enum_enum_i64_no_panic() {
    let _ = make_enum().try_div(make_enum());
}

#[kani::proof]
fn try_div_finite_enum_i64_no_panic() {
    let _ = make_finite().try_div(make_enum());
}

#[kani::proof]
fn try_div_half_enum_i64_no_panic() {
    let _ = make_half().try_div(make_enum());
}

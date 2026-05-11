//! `TryDiv` for all 9 monomorphizations at i64.
//!
//! Set-level `try_div` dispatches through `<i64 as TryDiv>::try_div`,
//! which is `checked_div` + an explicit zero pre-check (see
//! `intervalsets-core/src/ops/math/macros.rs::impl_try_div_checked`).
//! Both `i64::MIN / -1` and `x / 0` surface as `Err`, so inputs use
//! the full `i64` range with no input bounding.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryDiv;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::satisfy_bounds(
        FiniteBound::closed(kani::any()),
        FiniteBound::closed(kani::any()),
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
            FiniteBound::closed(kani::any()),
            FiniteBound::closed(kani::any()),
        ),
        1 => EnumInterval::<i64>::closed_unbound(kani::any()),
        2 => EnumInterval::<i64>::unbound_closed(kani::any()),
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

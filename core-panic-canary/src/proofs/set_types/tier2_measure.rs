//! Phase 2 — measure ops at i64. `try_count` / `try_width` /
//! `midpoint` are panic-free by contract: representation overflow
//! surfaces as `Err`, and the unbounded / empty / half-bounded shapes
//! return their canonical sentinel (`Extent::Infinite` for
//! count/width, `Err(Domain)` for midpoint).
//!
//! The panicking siblings `count()` / `width()` are explicitly
//! documented to panic on overflow and live outside the canary.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::measure::{Count, Width};
use intervalsets_core::ops::Midpoint;
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
fn try_count_finite_i64_no_panic() {
    let _ = make_finite().try_count();
}

#[kani::proof]
fn try_count_half_i64_no_panic() {
    let _ = make_half().try_count();
}

#[kani::proof]
fn try_count_enum_i64_no_panic() {
    let _ = make_enum().try_count();
}

#[kani::proof]
fn try_width_finite_i64_no_panic() {
    let _ = make_finite().try_width();
}

#[kani::proof]
fn try_width_half_i64_no_panic() {
    let _ = make_half().try_width();
}

#[kani::proof]
fn try_width_enum_i64_no_panic() {
    let _ = make_enum().try_width();
}

#[kani::proof]
fn midpoint_finite_i64_no_panic() {
    let _ = make_finite().midpoint();
}

#[kani::proof]
fn midpoint_half_i64_no_panic() {
    let _ = make_half().midpoint();
}

#[kani::proof]
fn midpoint_enum_i64_no_panic() {
    let _ = make_enum().midpoint();
}

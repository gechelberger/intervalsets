//! Phase 3 — `IntoFiniteInterval::into_finite_interval` for all 3 impls at i64.
//!
//! For FiniteInterval, `into_finite_interval` is the identity. HalfInterval
//! and EnumInterval clamp to the type bounds via `T: Bounded`, which
//! i64 satisfies through `num_traits::Bounded`.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::IntoFiniteInterval;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn make_finite() -> FiniteInterval<i64> {
    // satisfy_bounds is panic-free for i64 (no NaN) — strict factory
    // `closed` would panic on crossed input under strict semantics.
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
fn into_finite_finite_i64_no_panic() {
    let _ = make_finite().into_finite_interval();
}

#[kani::proof]
fn into_finite_half_i64_no_panic() {
    let _ = make_half().into_finite_interval();
}

#[kani::proof]
fn into_finite_enum_i64_no_panic() {
    let _ = make_enum().into_finite_interval();
}

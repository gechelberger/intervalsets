//! Phase 3 — `Complement::complement` for all 3 hand impls at i64.
//!
//! Tier 2: infallible when closed over the type invariants. The
//! validating-API factories cannot construct an invariant-violating
//! interval at i64 (no NaN), so each harness covers the full
//! validating-API surface for its impl. The blanket
//! `Complement for &X` (Clone) just delegates to the owned impl
//! after a clone, so a separate harness adds no signal.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Complement;
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
fn complement_finite_i64_no_panic() {
    let _ = make_finite().complement();
}

#[kani::proof]
fn complement_half_i64_no_panic() {
    let _ = make_half().complement();
}

#[kani::proof]
fn complement_enum_i64_no_panic() {
    let _ = make_enum().complement();
}

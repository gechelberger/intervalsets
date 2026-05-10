//! Phase 2 — `Split::try_split` for all 3 hand impls at i64.
//!
//! Split has 3 impls (FiniteInterval, HalfInterval, EnumInterval).
//! No arithmetic on T, so no overflow concerns; inputs are
//! unconstrained.

use intervalsets_core::bound::{FiniteBound, Side};
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Split;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn any_side() -> Side {
    if kani::any() {
        Side::Left
    } else {
        Side::Right
    }
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
fn try_split_finite_i64_no_panic() {
    let _ = make_finite().try_split(kani::any(), any_side());
}

#[kani::proof]
fn try_split_half_i64_no_panic() {
    let _ = make_half().try_split(kani::any(), any_side());
}

#[kani::proof]
fn try_split_enum_i64_no_panic() {
    let _ = make_enum().try_split(kani::any(), any_side());
}

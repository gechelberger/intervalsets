//! Phase 2 pilot — `Rebound::try_with_left` for `FiniteInterval<i64>`.
//!
//! Mirrors the canary's `f.try_with_left(Some(FiniteBound::closed(...)))`
//! call shape. No arithmetic on T, so no overflow concerns.

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Rebound;
use intervalsets_core::sets::FiniteInterval;

#[kani::proof]
fn try_with_left_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let new_left: i64 = kani::any();
    let bound_kind: bool = kani::any();
    let some_bound: bool = kani::any();

    let interval = FiniteInterval::<i64>::closed(lmin, lmax);
    let bound = if some_bound {
        Some(if bound_kind {
            FiniteBound::closed(new_left)
        } else {
            FiniteBound::open(new_left)
        })
    } else {
        None
    };

    let _ = interval.try_with_left(bound);
}

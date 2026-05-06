//! Phase 2 pilot — `TrySub` for `FiniteInterval<i64>`.
//!
//! Same input-bounding rationale as `tier3_add`: half-range bounds
//! keep `lmax - rmin` and `lmin - rmax` within i64.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TrySub;
use intervalsets_core::sets::FiniteInterval;

const HALF_MIN: i64 = i64::MIN / 2;
const HALF_MAX: i64 = i64::MAX / 2;

#[kani::proof]
fn try_sub_finite_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let rmin: i64 = kani::any();
    let rmax: i64 = kani::any();

    kani::assume(lmin >= HALF_MIN && lmax <= HALF_MAX);
    kani::assume(rmin >= HALF_MIN && rmax <= HALF_MAX);

    let lhs = FiniteInterval::<i64>::closed(lmin, lmax);
    let rhs = FiniteInterval::<i64>::closed(rmin, rmax);

    let _ = lhs.try_sub(rhs);
}

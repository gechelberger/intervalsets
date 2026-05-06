//! Phase 2 pilot — `TryMul` for `FiniteInterval<i64>`.
//!
//! Multiplication needs tighter bounds than add/sub: any pair product
//! must fit in i64. `MUL_BOUND = 2^31` keeps `MUL_BOUND * MUL_BOUND`
//! at `2^62 < i64::MAX = 2^63 - 1`. Wider bounds blow Kani's solver
//! time up quickly because the multiplier search space grows
//! quadratically.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryMul;
use intervalsets_core::sets::FiniteInterval;

const MUL_BOUND: i64 = 1 << 31;

#[kani::proof]
fn try_mul_finite_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let rmin: i64 = kani::any();
    let rmax: i64 = kani::any();

    kani::assume(lmin >= -MUL_BOUND && lmax <= MUL_BOUND);
    kani::assume(rmin >= -MUL_BOUND && rmax <= MUL_BOUND);

    let lhs = FiniteInterval::<i64>::closed(lmin, lmax);
    let rhs = FiniteInterval::<i64>::closed(rmin, rmax);

    let _ = lhs.try_mul(rhs);
}

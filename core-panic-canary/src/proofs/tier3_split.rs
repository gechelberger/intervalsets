//! Phase 2 pilot — `Split::try_split` for `FiniteInterval<i64>`.
//!
//! No arithmetic on T, so no overflow concerns; inputs are
//! unconstrained.

use intervalsets_core::bound::Side;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Split;
use intervalsets_core::sets::FiniteInterval;

#[kani::proof]
fn try_split_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let at: i64 = kani::any();
    let side: bool = kani::any();

    let interval = FiniteInterval::<i64>::closed(lmin, lmax);
    let closed = if side { Side::Left } else { Side::Right };

    let _ = interval.try_split(at, closed);
}

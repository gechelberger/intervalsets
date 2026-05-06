//! Phase 2 pilot — `ConvexHull::try_hull` for `FiniteInterval<i64>`.
//!
//! Iterator-consuming impl. We use a fixed-size array (3 elements)
//! rather than a Vec to keep the harness's loop bound static and
//! within the workspace `default-unwind = 4`. Each element is
//! independently nondeterministic, so the proof covers all
//! min/max orderings on a 3-element input.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::ConvexHull;
use intervalsets_core::sets::FiniteInterval;

#[kani::proof]
fn try_hull_finite_i64_array3_no_panic() {
    let a: i64 = kani::any();
    let b: i64 = kani::any();
    let c: i64 = kani::any();

    let elems = [a, b, c];

    let _ = <FiniteInterval<i64> as ConvexHull<i64>>::try_hull(elems);
}

//! Tier 3 panic-free canary — `ConvexHull::try_hull`, FiniteInterval-item impls.
//!
//! Hull has 10 impls (4 macro × 1 + 6 hand). Empirically up to 8 fit
//! per binary; the EnumInterval-item hand impls are split into
//! `panic_free_tier3_hull_enums`.
//!
//!     cargo build --example panic_free_tier3_hull --features panic-free-check --release

extern crate std;

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::ConvexHull;
use intervalsets_core::sets::{EnumInterval, FiniteInterval};

fn main() {
    let elems: std::vec::Vec<i64> = std::vec![1, 2, 3];
    let finites = std::vec![FiniteInterval::<i64>::closed(0, 5)];

    // 4 macro-generated impls: FiniteInterval / EnumInterval × T / &T.
    let _ = black_box(<FiniteInterval<i64> as ConvexHull<i64>>::try_hull(
        elems.clone(),
    ));
    let _ = black_box(<FiniteInterval<i64> as ConvexHull<&i64>>::try_hull(
        elems.iter(),
    ));
    let _ = black_box(<EnumInterval<i64> as ConvexHull<i64>>::try_hull(
        elems.clone(),
    ));
    let _ = black_box(<EnumInterval<i64> as ConvexHull<&i64>>::try_hull(
        elems.iter(),
    ));

    // 4 hand impls over FiniteInterval items.
    let _ = black_box(
        <FiniteInterval<i64> as ConvexHull<FiniteInterval<i64>>>::try_hull(finites.clone()),
    );
    let _ = black_box(
        <FiniteInterval<i64> as ConvexHull<&FiniteInterval<i64>>>::try_hull(finites.iter()),
    );
    let _ = black_box(
        <EnumInterval<i64> as ConvexHull<FiniteInterval<i64>>>::try_hull(finites.clone()),
    );
    let _ = black_box(
        <EnumInterval<i64> as ConvexHull<&FiniteInterval<i64>>>::try_hull(finites.iter()),
    );
}

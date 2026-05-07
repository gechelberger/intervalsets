//! Tier 3 panic-free canary — `ConvexHull::try_hull`, EnumInterval-item hand impls.
//!
//! Companion to `panic_free_tier3_hull` (which covers FiniteInterval-item impls).
//!
//!     cargo build --example panic_free_tier3_hull_enums --features panic-free-check --release

extern crate std;

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::ConvexHull;
use intervalsets_core::sets::EnumInterval;

fn main() {
    let enums = std::vec![EnumInterval::<i64>::closed(0, 5)];

    let _ =
        black_box(<EnumInterval<i64> as ConvexHull<EnumInterval<i64>>>::try_hull(enums.clone()));
    let _ =
        black_box(<EnumInterval<i64> as ConvexHull<&EnumInterval<i64>>>::try_hull(enums.iter()));
}

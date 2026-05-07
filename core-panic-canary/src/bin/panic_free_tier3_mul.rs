//! Tier 3 panic-free canary — `TryMul::try_mul`, hand impls A.
//!
//! TryMul has 9 hand impls but only ~3 fit per binary under the
//! optimizer's panic-edge-elimination budget. Coverage is split
//! across `panic_free_tier3_mul`, `_mul_b`, `_mul_c`.
//!
//!     cargo build --example panic_free_tier3_mul --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryMul;
use intervalsets_core::sets::{FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);

    let _ = black_box(f.try_mul(f)); // FiniteInterval × Self
    let _ = black_box(h_l.try_mul(h_l)); // HalfInterval × Self
    let _ = black_box(f.try_mul(h_l)); // FiniteInterval × HalfInterval
}

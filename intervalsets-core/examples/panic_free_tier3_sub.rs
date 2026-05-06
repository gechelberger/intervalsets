//! Tier 3 panic-free canary — `TrySub::try_sub` only.
//!
//!     cargo build --example panic_free_tier3_sub --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TrySub;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);
    let e = EnumInterval::<i64>::closed(0, 10);

    // Hand impls.
    let _ = black_box(f.try_sub(f));
    let _ = black_box(h_l.try_sub(h_l));
    let _ = black_box(f.try_sub(h_l));
    let _ = black_box(h_l.try_sub(f));

    // Dispatch_lhs macro: EnumInterval × {Finite, Half, Enum}.
    let _ = black_box(e.try_sub(f));
    let _ = black_box(e.try_sub(h_l));
    let _ = black_box(e.try_sub(e));

    // Dispatch_rhs macro: {Finite, Half} × EnumInterval.
    let _ = black_box(f.try_sub(e));
    let _ = black_box(h_l.try_sub(e));

    // Underflow path:
    let f_min = FiniteInterval::<i64>::closed(i64::MIN, i64::MIN + 5);
    let _ = black_box(f_min.try_sub(f));
}

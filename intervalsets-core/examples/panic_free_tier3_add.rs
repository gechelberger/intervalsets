//! Tier 3 panic-free canary — `TryAdd::try_add` only.
//!
//! Covers all 9 type-pair monomorphizations (3 hand + 3 dispatch + 3 commutative).
//!
//!     cargo build --example panic_free_tier3_add --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryAdd;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);
    let e = EnumInterval::<i64>::closed(0, 10);

    // Hand impls: FF, HH, HF.
    let _ = black_box(f.try_add(f));
    let _ = black_box(h_l.try_add(h_l));
    let _ = black_box(h_l.try_add(f));

    // Dispatch macro impls: EnumInterval × {Finite, Half, Enum}.
    let _ = black_box(e.try_add(f));
    let _ = black_box(e.try_add(h_l));
    let _ = black_box(e.try_add(e));

    // Commutative macro impls: {Finite, Half} × {Half, Enum}.
    let _ = black_box(f.try_add(h_l));
    let _ = black_box(f.try_add(e));
    let _ = black_box(h_l.try_add(e));

    // Overflow path (release wraps silently — must not panic):
    let f_max = FiniteInterval::<i64>::closed(i64::MAX - 5, i64::MAX);
    let _ = black_box(f_max.try_add(f));
}

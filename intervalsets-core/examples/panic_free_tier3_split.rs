//! Tier 3 panic-free canary — `Split::try_split` only.
//!
//! Per-trait isolation: comprehensive Tier 3 verification across
//! multiple `try_*` trait families in one binary triggers cascade
//! no_panic link failures (LLVM optimizer budget exhaustion). One
//! canary per Tier 3 trait family avoids the cascade.
//!
//!     cargo build --example panic_free_tier3_split --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::bound::Side;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Split;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);
    let h_r = HalfInterval::<i64>::unbound_closed(10);
    let e = EnumInterval::<i64>::closed(0, 10);
    let e_unbounded = EnumInterval::<i64>::unbounded();
    let e_empty = EnumInterval::<i64>::empty();

    // FiniteInterval: at inside, at outside upper, at outside lower, both Side variants.
    let _ = black_box(f.try_split(5, Side::Left));
    let _ = black_box(f.try_split(5, Side::Right));
    let _ = black_box(f.try_split(100, Side::Left));
    let _ = black_box(f.try_split(-100, Side::Right));

    // HalfInterval: both side flavors, at inside and outside.
    let _ = black_box(h_l.try_split(5, Side::Left));
    let _ = black_box(h_l.try_split(5, Side::Right));
    let _ = black_box(h_l.try_split(-5, Side::Left));
    let _ = black_box(h_r.try_split(5, Side::Left));
    let _ = black_box(h_r.try_split(50, Side::Right));

    // EnumInterval: covers Finite, Half, Unbounded variants.
    let _ = black_box(e.try_split(5, Side::Left));
    let _ = black_box(e.try_split(100, Side::Right));
    let _ = black_box(e_unbounded.try_split(0, Side::Left));
    let _ = black_box(e_unbounded.try_split(0, Side::Right));
    let _ = black_box(e_empty.try_split(0, Side::Left));
}

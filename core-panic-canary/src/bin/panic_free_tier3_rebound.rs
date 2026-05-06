//! Tier 3 panic-free canary — `Rebound::try_with_left` / `try_with_right` only.
//!
//!     cargo build --example panic_free_tier3_rebound --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Rebound;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);
    let h_r = HalfInterval::<i64>::unbound_closed(10);
    let e = EnumInterval::<i64>::closed(0, 10);
    let e_unbounded = EnumInterval::<i64>::unbounded();

    // FiniteInterval: None / normal / crossing-bound (silent empty).
    let _ = black_box(f.try_with_left(None));
    let _ = black_box(f.try_with_left(Some(FiniteBound::closed(-5))));
    let _ = black_box(f.try_with_left(Some(FiniteBound::open(-5))));
    let _ = black_box(f.try_with_left(Some(FiniteBound::closed(100))));
    let _ = black_box(f.try_with_right(None));
    let _ = black_box(f.try_with_right(Some(FiniteBound::closed(20))));
    let _ = black_box(f.try_with_right(Some(FiniteBound::open(20))));
    let _ = black_box(f.try_with_right(Some(FiniteBound::closed(-100))));

    // HalfInterval: both side flavors, None / normal.
    let _ = black_box(h_l.try_with_left(None));
    let _ = black_box(h_l.try_with_left(Some(FiniteBound::closed(5))));
    let _ = black_box(h_l.try_with_right(None));
    let _ = black_box(h_l.try_with_right(Some(FiniteBound::closed(50))));
    let _ = black_box(h_r.try_with_left(None));
    let _ = black_box(h_r.try_with_left(Some(FiniteBound::closed(5))));
    let _ = black_box(h_r.try_with_right(None));
    let _ = black_box(h_r.try_with_right(Some(FiniteBound::closed(50))));

    // EnumInterval: covers Finite/Half/Unbounded variants.
    let _ = black_box(e.try_with_left(None));
    let _ = black_box(e.try_with_left(Some(FiniteBound::closed(-5))));
    let _ = black_box(e.try_with_right(None));
    let _ = black_box(e.try_with_right(Some(FiniteBound::closed(20))));
    let _ = black_box(e_unbounded.try_with_left(None));
    let _ = black_box(e_unbounded.try_with_left(Some(FiniteBound::closed(0))));
    let _ = black_box(e_unbounded.try_with_right(None));
    let _ = black_box(e_unbounded.try_with_right(Some(FiniteBound::closed(0))));
}

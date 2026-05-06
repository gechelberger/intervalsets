//! Tier 3 panic-free canary — `TryMul::try_mul`, partial EnumInterval coverage.
//!
//! TryMul's EnumInterval-LHS impls (`E×F`, `E×H`, `E×E`, `F×E`, `H×E`)
//! internally dispatch through the hand impls; they're heavy enough
//! that two of them in the same binary exceed the budget. This
//! canary covers `H×F` plus one EnumInterval dispatch as a
//! representative sample. Comprehensive coverage of the remaining
//! Enum-dispatch impls is currently out of reach.
//!
//!     cargo build --example panic_free_tier3_mul_b --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryMul;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let h_l = HalfInterval::<i64>::closed_unbound(0);
    let e = EnumInterval::<i64>::closed(0, 10);

    let _ = black_box(h_l.try_mul(f));          // HalfInterval × FiniteInterval (hand)
    let _ = black_box(e.try_mul(f));            // EnumInterval × FiniteInterval (dispatch)
    let _ = (h_l, e);
}

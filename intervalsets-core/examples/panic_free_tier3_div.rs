//! Tier 3 panic-free canary — `TryDiv::try_div`, single demonstrative call.
//!
//! TryDiv has 9 impls but only one call fits per binary under the
//! optimizer's panic-edge-elimination budget — even two calls of
//! the *same* monomorphization trigger cascade no_panic link
//! failures. The panic-free claim is individually verifiable per
//! instantiation; comprehensive verification across all 9
//! monomorphizations would require 9 separate canary files, which
//! is judged not worth the maintenance cost.
//!
//! This file proves the wiring: `try_div` for `FiniteInterval<i64>`
//! is panic-free in release, even when the divisor crosses zero
//! (the path that would panic at the underlying `/` if not handled
//! by the categorical dispatch).
//!
//!     cargo build --example panic_free_tier3_div --features panic-free-check --release

use core::hint::black_box;

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryDiv;
use intervalsets_core::sets::FiniteInterval;

fn main() {
    let f = FiniteInterval::<i64>::closed(0, 10);
    let f_zero_crossing = FiniteInterval::<i64>::closed(-5, 5);

    let _ = black_box(f.try_div(f_zero_crossing));
}

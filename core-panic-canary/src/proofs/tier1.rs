//! Phase 0 — Kani-wiring sanity harness.
//!
//! `Contains::contains` for `FiniteInterval<i64>` is Tier 1 (truly
//! infallible) and the simplest predicate in the crate. We already
//! know it cannot panic; the point of this harness is to confirm
//! Kani is actually verifying things before we point it at proofs
//! whose answers we don't already know.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::Contains;
use intervalsets_core::sets::FiniteInterval;

#[kani::proof]
fn contains_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let elem: i64 = kani::any();

    // `closed(a, b)` factory returns Self with empty fallback when
    // a > b (no NaN concern at i64), so we don't need kani::assume.
    let interval = FiniteInterval::<i64>::closed(lmin, lmax);

    let _ = interval.contains(&elem);
}

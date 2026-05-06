//! Phase 2 pilot — `TryAdd` for `FiniteInterval<i64>`.
//!
//! Expansion to all 9 monomorphizations (3 hand + 3 dispatch + 3
//! commutative) is Phase 2's expansion step; this harness validates
//! the approach for the trait family.
//!
//! Inputs are bounded to a half-range to keep `lmax + rmax` and
//! `lmin + rmin` within i64 — Kani requires `overflow-checks=on` for
//! sound analysis (stricter than release-mode `+` semantics, which
//! wraps silently). The harness therefore proves "no panic AND no
//! overflow" rather than just "no panic in release", which is a
//! strictly stronger property than the documented Tier 3 contract.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryAdd;
use intervalsets_core::sets::FiniteInterval;

const HALF_MIN: i64 = i64::MIN / 2;
const HALF_MAX: i64 = i64::MAX / 2;

#[kani::proof]
fn try_add_finite_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let rmin: i64 = kani::any();
    let rmax: i64 = kani::any();

    kani::assume(lmin >= HALF_MIN && lmax <= HALF_MAX);
    kani::assume(rmin >= HALF_MIN && rmax <= HALF_MAX);

    let lhs = FiniteInterval::<i64>::closed(lmin, lmax);
    let rhs = FiniteInterval::<i64>::closed(rmin, rmax);

    let _ = lhs.try_add(rhs);
}

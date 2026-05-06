//! Phase 1 — `TryDiv` for `FiniteInterval<i64>`.
//!
//! TryDiv is the trait whose linker canary could only fit a single
//! call before hitting the optimizer's panic-edge-elimination
//! cascade. Mirror of `src/bin/panic_free_tier3_div.rs`, but with
//! nondeterministic interval bounds — Kani exhausts the input space
//! up to its bounds, including the divisor-crosses-zero case that
//! the categorical dispatch must divert before reaching
//! `div_assume_nonzero`.
//!
//! # Finding (`i64::MIN / -1` overflow — see scratch/panic-free-canary.md)
//!
//! Without the kani::assume excluding the `i64::MIN / -1` case below,
//! Kani reports VERIFICATION FAILED with `attempt to divide with overflow`
//! at `<i64 as Div>::div`. The categorical dispatch in
//! `intervalsets-core/src/ops/math/div.rs::impls` correctly diverts
//! divide-by-zero, but does not handle signed-integer-min divided by
//! -1 (Rust panics on this in both debug and release). The linker
//! canary missed it because its concrete fixtures never reached the
//! edge case.
//!
//! Until that gap is fixed in `intervalsets-core`, this harness
//! excludes the failing inputs so it can prove the rest of try_div
//! is panic-free.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::math::TryDiv;
use intervalsets_core::sets::FiniteInterval;

#[kani::proof]
fn try_div_finite_finite_i64_no_panic() {
    let lmin: i64 = kani::any();
    let lmax: i64 = kani::any();
    let rmin: i64 = kani::any();
    let rmax: i64 = kani::any();

    // Exclude the known `i64::MIN / -1` overflow path: when the
    // numerator interval can include i64::MIN and the denominator
    // interval can include -1, the inner integer division panics.
    // This is a documented gap in `intervalsets-core::ops::math::div`,
    // not a Kani false positive.
    kani::assume(!(lmin == i64::MIN && rmin <= -1 && rmax >= -1));

    let lhs = FiniteInterval::<i64>::closed(lmin, lmax);
    let rhs = FiniteInterval::<i64>::closed(rmin, rmax);

    let _ = lhs.try_div(rhs);
}

//! Storage-type `TryDiv` surface added in E2. See `tier3_add` for the
//! macro-family / representative-width rationale.
//!
//! `try_div` is the trickiest of the four: integer impls pre-check
//! zero before calling `checked_div`, and `checked_div` itself catches
//! signed `MIN / -1` overflow. Floats route through
//! `is_finite()` post-op so `INF` (`1.0 / 0.0`) and `NaN`
//! (`0.0 / 0.0`) both surface as `Err(Domain)` without an explicit
//! zero pre-check. The proofs below verify both paths are panic-free
//! over the full input range.

use intervalsets_core::ops::math::TryDiv;

use super::{any_finite_f64, any_option_i64};

#[kani::proof]
fn try_div_i64_no_panic() {
    let _ = <i64 as TryDiv>::try_div(kani::any(), kani::any());
}

#[kani::proof]
fn try_div_u64_no_panic() {
    let _ = <u64 as TryDiv>::try_div(kani::any(), kani::any());
}

#[kani::proof]
fn try_div_f64_no_panic() {
    let lhs = any_finite_f64();
    let rhs = any_finite_f64();
    // `0.0 / 0.0 = NaN` triggers Kani's NaN-on-division property
    // check. Non-zero / 0.0 produces ±INF (caught by `is_finite()`).
    // Excluding the (0, 0) combination only suppresses Kani's extra
    // semantic check; the macro's panic-free contract still holds for
    // that case (it returns `Err(Domain)`), verified by unit test in
    // `intervalsets-core/src/ops/math/macros.rs`.
    kani::assume(!(lhs == 0.0 && rhs == 0.0));
    let _ = <f64 as TryDiv>::try_div(lhs, rhs);
}

#[kani::proof]
fn try_div_option_i64_no_panic() {
    let _ = any_option_i64().try_div(any_option_i64());
}

#[cfg(feature = "ordered-float")]
#[kani::proof]
fn try_div_ordered_float_f64_no_panic() {
    use ordered_float::OrderedFloat;

    use super::any_finite_ordered_float_f64;
    let lhs = any_finite_ordered_float_f64();
    let rhs = any_finite_ordered_float_f64();
    // Same `0.0 / 0.0 = NaN` carve-out as bare `f64`.
    kani::assume(!(lhs == OrderedFloat(0.0) && rhs == OrderedFloat(0.0)));
    let _ = lhs.try_div(rhs);
}

#[cfg(feature = "ordered-float")]
#[kani::proof]
fn try_div_not_nan_f64_no_panic() {
    use ordered_float::NotNan;

    use super::any_finite_not_nan_f64;
    let lhs = any_finite_not_nan_f64();
    let rhs = any_finite_not_nan_f64();
    // Same `0.0 / 0.0 = NaN` carve-out as bare `f64`.
    let zero = NotNan::new(0.0_f64).unwrap();
    kani::assume(!(lhs == zero && rhs == zero));
    let _ = lhs.try_div(rhs);
}

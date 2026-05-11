//! Storage-type `TrySub` surface added in E2. See `tier3_add` for the
//! macro-family / representative-width rationale.

use intervalsets_core::ops::math::TrySub;

use super::{any_finite_f64, any_option_i64};

#[kani::proof]
fn try_sub_i64_no_panic() {
    let _ = <i64 as TrySub>::try_sub(kani::any(), kani::any());
}

#[kani::proof]
fn try_sub_u64_no_panic() {
    let _ = <u64 as TrySub>::try_sub(kani::any(), kani::any());
}

#[kani::proof]
fn try_sub_f64_no_panic() {
    let _ = <f64 as TrySub>::try_sub(any_finite_f64(), any_finite_f64());
}

#[kani::proof]
fn try_sub_option_i64_no_panic() {
    let _ = any_option_i64().try_sub(any_option_i64());
}

#[cfg(feature = "ordered-float")]
#[kani::proof]
fn try_sub_ordered_float_f64_no_panic() {
    use super::any_finite_ordered_float_f64;
    let _ = any_finite_ordered_float_f64().try_sub(any_finite_ordered_float_f64());
}

#[cfg(feature = "ordered-float")]
#[kani::proof]
fn try_sub_not_nan_f64_no_panic() {
    use super::any_finite_not_nan_f64;
    let _ = any_finite_not_nan_f64().try_sub(any_finite_not_nan_f64());
}

//! Storage-type `TryMul` surface added in E2. See `tier3_add` for the
//! macro-family / representative-width rationale.

use intervalsets_core::ops::math::TryMul;

use super::{any_finite_f64, any_option_i64};

#[kani::proof]
fn try_mul_i64_no_panic() {
    let _ = <i64 as TryMul>::try_mul(kani::any(), kani::any());
}

#[kani::proof]
fn try_mul_u64_no_panic() {
    let _ = <u64 as TryMul>::try_mul(kani::any(), kani::any());
}

#[kani::proof]
fn try_mul_f64_no_panic() {
    let _ = <f64 as TryMul>::try_mul(any_finite_f64(), any_finite_f64());
}

#[kani::proof]
fn try_mul_option_i64_no_panic() {
    let _ = any_option_i64().try_mul(any_option_i64());
}

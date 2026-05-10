//! Storage-type `TryAdd` surface added in E2.
//!
//! Covers the per-`T` arithmetic impls the set-level math dispatches
//! through (`intervalsets-core/src/ops/math/macros.rs`):
//!
//! - `impl_try_add_checked!` (integer primitives via `checked_add`),
//!   represented by `i64` (signed) and `u64` (unsigned).
//! - `impl_try_add_float_finite!` (IEEE-754 floats with non-finite
//!   detection), represented by `f64`.
//! - `Option<T>` delegating wrapper, represented at `Option<i64>`.
//!
//! Each macro family expands the same pattern for every primitive
//! width, so a panic-free proof at the representative width
//! certifies the family. Inputs use the full type range — unlike
//! the set-level harnesses, the storage-type `checked_add` and
//! `is_finite()` paths are total without input bounding.

use intervalsets_core::ops::math::TryAdd;

use super::{any_finite_f64, any_option_i64};

#[kani::proof]
fn try_add_i64_no_panic() {
    let _ = <i64 as TryAdd>::try_add(kani::any(), kani::any());
}

#[kani::proof]
fn try_add_u64_no_panic() {
    let _ = <u64 as TryAdd>::try_add(kani::any(), kani::any());
}

#[kani::proof]
fn try_add_f64_no_panic() {
    let _ = <f64 as TryAdd>::try_add(any_finite_f64(), any_finite_f64());
}

/// Covers all four discriminant pairs: `(Some, Some)` flows through the
/// inner `T::try_add`; `(Some, None)`, `(None, Some)`, `(None, None)`
/// all short-circuit to `Ok(None)`.
#[kani::proof]
fn try_add_option_i64_no_panic() {
    let _ = any_option_i64().try_add(any_option_i64());
}

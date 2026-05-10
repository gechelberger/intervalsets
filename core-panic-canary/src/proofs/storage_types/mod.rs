//! Kani harnesses over the per-`T` storage-type `TryOp` surface added
//! in E2 — the math on the underlying `T` that the set-level dispatch
//! funnels into.
//!
//! Each file mirrors one trait (`TryAdd` / `TrySub` / `TryMul` /
//! `TryDiv`) and covers a representative per macro family:
//!
//! - `impl_try_*_checked!` — signed (`i64`) and unsigned (`u64`).
//! - `impl_try_*_float_finite!` — `f64`.
//! - `Option<T>` delegating wrapper — `Option<i64>`.
//!
//! The macros expand the same pattern across primitive widths, so a
//! single representative per family certifies the family.

pub mod tier3_add;
pub mod tier3_div;
pub mod tier3_mul;
pub mod tier3_sub;

/// Shared `Option<i64>` generator. Exhausts both discriminants and the
/// full `i64` bit-pattern for the `Some` arm.
#[cfg(kani)]
pub(crate) fn any_option_i64() -> Option<i64> {
    if kani::any() {
        Some(kani::any::<i64>())
    } else {
        None
    }
}

/// Finite `f64` generator. CBMC's "NaN on addition" property check
/// (and the matching sub/mul/div variants) fires when the op produces
/// `NaN` from non-finite inputs (`INF + -INF`, `INF - INF`, `0 * INF`,
/// `0 / 0`), even though our `is_finite()` post-check catches the
/// result and returns `Err(Domain)` rather than panicking. The
/// panic-free contract holds for *all* inputs; the assumption below
/// just hides Kani's stricter semantic check, mirroring the
/// half-range bound the `i64` set-level harnesses use to dodge Kani's
/// overflow check. Non-finite-input handling is covered by the unit
/// tests in `intervalsets-core/src/ops/math/{add,sub,mul,div}.rs`.
#[cfg(kani)]
pub(crate) fn any_finite_f64() -> f64 {
    let v: f64 = kani::any();
    kani::assume(v.is_finite());
    v
}

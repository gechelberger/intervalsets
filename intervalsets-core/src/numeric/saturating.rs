//! Storage-type support for [`core::num::Saturating<T>`].
//!
//! `Saturating<T>` wraps a primitive integer and redefines `+`/`-`/`*`
//! (and unary `-` for signed) with saturating semantics: overflow at
//! `T::MIN`/`T::MAX` is clamped to the extremum rather than panicking
//! or wrapping. Division and remainder retain their bare-`T` semantics
//! (panic on `_/0`; on signed types, panic on `MIN/-1`).
//!
//! # Why this is a valid storage type
//!
//! Saturating add/sub/mul are **monotonic and ordering-preserving** on
//! the bounded integer lattice. They are closed operations that preserve
//! the bound-pair invariant `lo ≤ hi`. Saturation is therefore *defined
//! behavior*, not overflow:
//!
//! - [`crate::ops::math::TryAdd`] / [`crate::ops::math::TrySub`]
//!   / [`crate::ops::math::TryMul`] for `Saturating<T>` return
//!   `Ok(saturated_result)` with `Error = Infallible`. For example
//!   `[Saturating(100i8), Saturating(120)] + [Saturating(50), Saturating(60)]`
//!   correctly yields `[Saturating(127), Saturating(127)]` — that *is*
//!   the set-theoretic answer, since every element of the lhs plus every
//!   element of the rhs saturates to `127`.
//! - [`crate::ops::math::TryDiv`] is likewise `Error = Infallible`.
//!   Every overflow path — including `_/0` — saturates by the sign
//!   of the true result: positive overflow clamps to `MAX`, negative
//!   overflow clamps to `MIN`. `+n/0` (limit `+∞`) → `MAX`, `-n/0`
//!   (limit `-∞`) → `MIN`. The integer-only `MIN/-1` case → `MAX`,
//!   matching stdlib's `core::num::Saturating<iN>` Div. For inner
//!   types where `|rhs|` can be less than 1 (fixed-point, decimal,
//!   etc.) division can grow magnitude in either direction; the
//!   sign-of-result rule handles them uniformly. `0/0` is genuinely
//!   indeterminate and falls out as `MAX` from the sign rule — a
//!   convention, not a limit, but the only `Try*` op input that's
//!   mathematically undefined at any precision.
//!
//! # Contrast with `core::num::Wrapping<T>`
//!
//! [`core::num::Wrapping<T>`] is **not** a valid storage type and will
//! not be added. Wrapping overflow flips ordering (`i8::MAX + 1 = i8::MIN`),
//! violating the bound-pair invariant `lo ≤ hi`. There is no
//! semantically coherent path to making `Wrapping<T>` a valid storage
//! type for this crate's interval/set types.
//!
//! # Bounds-driven surface
//!
//! Each trait impl below is bounded on the corresponding `num_traits`
//! capability trait on the inner `T`. For the 12 primitive integer
//! types (`i8..i128`, `u8..u128`, `isize`, `usize`) every impl fires
//! and `Saturating<T>` is fully usable as a storage type. For inner
//! types lacking certain capabilities (e.g. `BigInt` is unbounded so
//! has no `Bounded` impl, `f32` has no `SaturatingAdd`), the
//! corresponding pieces are simply not available — the rest still
//! works.
//!
//! # Note on `num_traits::{Bounded, Zero, One}`
//!
//! Rust's orphan rule blocks this crate from blanket-impl'ing
//! `num_traits::{Bounded, Zero, One}` for `Saturating<T>` — both the
//! trait and the type are foreign. For the natural usage pattern this
//! is a non-issue: saturating arithmetic is closed over the bounded
//! integer lattice, so a `FiniteInterval<Saturating<T>>` never needs
//! to escape its own type to stay representable. Overflow paths just
//! clamp at the storage type's `MIN`/`MAX` and the result is still a
//! valid `FiniteInterval<Saturating<T>>`.
//!
//! The limitation only surfaces when a user *intentionally* composes
//! `Saturating<T>` with an unbounded variant
//! ([`EnumInterval`](crate::EnumInterval)`::Unbounded` /
//! [`HalfInterval`](crate::HalfInterval)) and then calls
//! [`IntoFiniteInterval`](crate::ops::IntoFiniteInterval) to flatten
//! it back to a `FiniteInterval`. That path bounds on `T: Bounded`
//! and is currently unavailable; the workaround is to construct the
//! `FiniteInterval` directly with `Saturating(T::MIN)` /
//! `Saturating(T::MAX)`. The missing `Zero`/`One` impls on the set
//! types only matter if the additive/multiplicative identity
//! intervals are needed by name rather than constructed inline.
//!
//! The principled fix — upstream `num_traits` blankets or a
//! crate-local capability trait — is deferred until a real use case
//! makes it worth doing.

use core::convert::Infallible;
use core::num::Saturating;

use num_traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedSub, One, SaturatingAdd, SaturatingMul, SaturatingSub,
    Zero,
};

use crate::bound::Side;
use crate::cast::{LossyCastElement, TryCastElement};
use crate::measure::{Countable, Widthable};
use crate::numeric::{Element, Midpoint};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

// ===== Element ======================================================

impl<T> Element for Saturating<T>
where
    T: Element + CheckedAdd + CheckedSub + One,
{
    #[inline]
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Right => self.0.checked_add(&T::one()).map(Saturating),
            Side::Left => self.0.checked_sub(&T::one()).map(Saturating),
        }
    }
    // `validate` defaults: delegates to `partial_cmp(&self).is_some()`,
    // which is correct for integer inner types (no NaN). For an inner
    // type with intrinsic infinities the override on `T` is not
    // inherited — `Saturating<T>` would need its own override. None of
    // the supported inner types fall into that case.
}

// ===== Midpoint =====================================================

impl<T> Midpoint for Saturating<T>
where
    T: Midpoint,
{
    type Error = T::Error;

    #[inline]
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        self.0.midpoint(other.0).map(Saturating)
    }
}

// ===== Widthable / Countable ========================================
//
// `num_traits::{Bounded, Zero, One}` are intentionally not impl'd for
// `Saturating<T>` here — both the trait and the type are foreign, so
// the orphan rule blocks it. See the module-level docs for the
// consequences and the deferred-follow-up plan.

impl<T> Widthable for Saturating<T>
where
    T: Widthable,
{
    type Output = T::Output;

    #[inline]
    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        T::width_between(&left.0, &right.0)
    }
}

impl<T> Countable for Saturating<T>
where
    T: Countable,
    Self: Element,
{
    type Output = T::Output;

    #[inline]
    fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
        T::count_inclusive(&left.0, &right.0)
    }
}

// ===== Arithmetic: TryAdd / TrySub / TryMul / TryDiv ================

impl<T> TryAdd for Saturating<T>
where
    T: SaturatingAdd<Output = T>,
{
    type Output = Self;
    type Error = Infallible;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(Saturating(self.0.saturating_add(&rhs.0)))
    }
}

impl<T> TrySub for Saturating<T>
where
    T: SaturatingSub<Output = T>,
{
    type Output = Self;
    type Error = Infallible;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(Saturating(self.0.saturating_sub(&rhs.0)))
    }
}

impl<T> TryMul for Saturating<T>
where
    T: SaturatingMul<Output = T>,
{
    type Output = Self;
    type Error = Infallible;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(Saturating(self.0.saturating_mul(&rhs.0)))
    }
}

impl<T> TryDiv for Saturating<T>
where
    T: Zero + Bounded + PartialOrd + CheckedDiv<Output = T>,
{
    type Output = Self;
    type Error = Infallible;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        // Saturating arithmetic never errors. `CheckedDiv` can fail
        // for two reasons:
        //   1. Overflow — `MIN/-1` for signed integers, or
        //      `|self| / |rhs|` exceeding the type's range for inner
        //      types admitting `|rhs| < 1` (fixed-point, decimal).
        //   2. Division by zero — `+n/0` (limit `+∞`), `-n/0` (limit
        //      `-∞`), or the indeterminate `0/0`.
        //
        // Both cases saturate by the sign of the true result: positive
        // → `MAX`, negative → `MIN`. The sign is the XOR of the
        // operands' signs. `0/0` is mathematically indeterminate; the
        // rule lands it on `MAX` as the saturation convention.
        Ok(Saturating(self.0.checked_div(&rhs.0).unwrap_or_else(
            || {
                let zero = T::zero();
                let result_negative = (self.0 < zero) ^ (rhs.0 < zero);
                if result_negative {
                    T::min_value()
                } else {
                    T::max_value()
                }
            },
        )))
    }
}

// ===== Cast surface =================================================
//
// Only `TryCast` and `LossyCast` are exposed; Tier-1 `Cast` is
// intentionally omitted. See the module docs and the PR plan for
// rationale: lifting `T → Saturating<T>` doesn't change values but
// does change downstream arithmetic semantics, so it should be an
// explicit opt-in.
//
// All six impls are total — no error path is reachable — but routing
// through `TryCast`/`LossyCast` forces the caller to spell out the
// semantic shift.
//
// TODO(saturating-cast-surface): expand to cross-Saturating
// (`Saturating<T> ↔ Saturating<U>`) and cross-with-primitive
// (`Saturating<T> ↔ U`) pairs once a real call site needs them.

// Reflexive: Saturating<T> → Saturating<T>
impl<T> TryCastElement<Saturating<T>> for Saturating<T> {
    #[inline]
    fn try_cast_element(self) -> Option<Saturating<T>> {
        Some(self)
    }
}

impl<T> LossyCastElement<Saturating<T>> for Saturating<T> {
    #[inline]
    fn lossy_cast_element(self) -> Saturating<T> {
        self
    }
}

// Lift: T → Saturating<T>
impl<T> TryCastElement<Saturating<T>> for T
where
    T: crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<Saturating<T>> {
        Some(Saturating(self))
    }
}

impl<T> LossyCastElement<Saturating<T>> for T
where
    T: crate::cast::Primitive,
{
    #[inline]
    fn lossy_cast_element(self) -> Saturating<T> {
        Saturating(self)
    }
}

// Unwrap: Saturating<T> → T
impl<T> TryCastElement<T> for Saturating<T>
where
    T: crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<T> {
        Some(self.0)
    }
}

impl<T> LossyCastElement<T> for Saturating<T>
where
    T: crate::cast::Primitive,
{
    #[inline]
    fn lossy_cast_element(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    // ----- Construction / normalization ---------------------------------

    #[test]
    fn construct_closed_interval() {
        let x = EnumInterval::closed(Saturating(0_i32), Saturating(10));
        assert!(x.contains(&Saturating(5_i32)));
    }

    #[test]
    fn open_open_normalizes_to_closed() {
        // Discrete normalization: (0, 10) for an integer-like storage
        // collapses to [1, 9].
        let open = EnumInterval::open(Saturating(0_i32), Saturating(10));
        let closed = EnumInterval::closed(Saturating(1_i32), Saturating(9));
        assert_eq!(open, closed);
    }

    #[test]
    fn singleton_at_min_is_valid() {
        let x = FiniteInterval::closed(Saturating(i8::MIN), Saturating(i8::MIN));
        assert!(x.contains(&Saturating(i8::MIN)));
    }

    // ----- Measure ------------------------------------------------------

    #[test]
    fn count_matches_inner() {
        let x = EnumInterval::closed(Saturating(0_i32), Saturating(10));
        assert_eq!(x.count().finite(), 11u128);
    }

    #[test]
    fn width_matches_inner() {
        let x = EnumInterval::closed(Saturating(0_i32), Saturating(10));
        assert_eq!(x.width().finite(), 10u128);
    }

    // ----- Midpoint -----------------------------------------------------

    #[test]
    fn midpoint_basic() {
        let m = Saturating(10_i32).midpoint(Saturating(20)).unwrap();
        assert_eq!(m, Saturating(15_i32));
    }

    #[test]
    fn midpoint_at_extremes_does_not_overflow() {
        // Delegates to `i8::midpoint`, which is overflow-free.
        let m = Saturating(i8::MIN).midpoint(Saturating(i8::MAX)).unwrap();
        // i8::MIN + i8::MAX = -1; / 2 toward zero = 0.
        assert_eq!(m, Saturating(0_i8));
    }

    // ----- Element-level arithmetic: saturating, total ------------------

    #[test]
    fn try_add_saturates_at_max() {
        let r = Saturating(120_i8).try_add(Saturating(50)).unwrap();
        assert_eq!(r, Saturating(i8::MAX));
    }

    #[test]
    fn try_sub_saturates_at_min() {
        let r = Saturating(-120_i8).try_sub(Saturating(50)).unwrap();
        assert_eq!(r, Saturating(i8::MIN));
    }

    #[test]
    fn try_mul_saturates_at_max() {
        let r = Saturating(64_i8).try_mul(Saturating(4)).unwrap();
        assert_eq!(r, Saturating(i8::MAX));
    }

    #[test]
    fn try_div_normal() {
        let r = Saturating(10_i32).try_div(Saturating(3)).unwrap();
        assert_eq!(r, Saturating(3));
    }

    #[test]
    fn try_div_positive_by_zero_saturates_to_max() {
        // `+n / 0` → limit `+∞` → `MAX`.
        let r = Saturating(1_i32).try_div(Saturating(0)).unwrap();
        assert_eq!(r, Saturating(i32::MAX));
    }

    #[test]
    fn try_div_negative_by_zero_saturates_to_min() {
        // `-n / 0` → limit `-∞` → `MIN`.
        let r = Saturating(-1_i32).try_div(Saturating(0)).unwrap();
        assert_eq!(r, Saturating(i32::MIN));
    }

    #[test]
    fn try_div_zero_by_zero_saturates_to_max() {
        // `0/0` is mathematically indeterminate; the saturation
        // convention is `MAX`.
        let r = Saturating(0_i32).try_div(Saturating(0)).unwrap();
        assert_eq!(r, Saturating(i32::MAX));
    }

    #[test]
    fn try_div_signed_min_neg_one_saturates() {
        // True result is `+|i8::MIN| = 128`, which exceeds `i8::MAX`.
        // Saturate to `MAX`, matching stdlib's `Saturating<i8>::Div`.
        let r = Saturating(i8::MIN).try_div(Saturating(-1)).unwrap();
        assert_eq!(r, Saturating(i8::MAX));
    }

    // ----- Set-level arithmetic at boundaries ---------------------------

    #[test]
    fn set_add_saturates_at_high_boundary() {
        let a = FiniteInterval::closed(Saturating(100_i8), Saturating(120));
        let b = FiniteInterval::closed(Saturating(50_i8), Saturating(60));
        let sum = a.try_add(b).unwrap();
        let expected = FiniteInterval::closed(Saturating(i8::MAX), Saturating(i8::MAX));
        assert_eq!(sum, expected);
    }

    #[test]
    fn set_sub_saturates_at_low_boundary() {
        let a = FiniteInterval::closed(Saturating(-120_i8), Saturating(-100));
        let b = FiniteInterval::closed(Saturating(50_i8), Saturating(60));
        let diff = a.try_sub(b).unwrap();
        let expected = FiniteInterval::closed(Saturating(i8::MIN), Saturating(i8::MIN));
        assert_eq!(diff, expected);
    }

    // `IntoFiniteInterval` requires `T: num_traits::Bounded`. The
    // orphan rule blocks `impl Bounded for Saturating<T>` from this
    // crate, so `EnumInterval<Saturating<T>>::into_finite_interval()`
    // is not currently available. See module-level docs for the
    // workaround and the deferred-follow-up plan.

    // ----- Casts --------------------------------------------------------

    #[test]
    fn try_cast_lift_t_to_saturating() {
        let x = FiniteInterval::closed(0_i32, 10);
        let y: FiniteInterval<Saturating<i32>> = x.try_cast().unwrap();
        assert_eq!(y, FiniteInterval::closed(Saturating(0_i32), Saturating(10)));
    }

    // `LossyCast` for `FiniteInterval<T> → FiniteInterval<Saturating<T>>`
    // requires `Saturating<T>: Bounded`. The orphan rule blocks that
    // blanket impl from this crate, so the set-level LossyCast lift is
    // currently unavailable. The element-level
    // `LossyCastElement<Saturating<T>> for T` impl below is still
    // correct and usable directly; only the set-level path is blocked.

    #[test]
    fn try_cast_unwrap_saturating_to_t() {
        let x = FiniteInterval::closed(Saturating(0_i32), Saturating(10));
        let y: FiniteInterval<i32> = x.try_cast().unwrap();
        assert_eq!(y, FiniteInterval::closed(0_i32, 10));
    }

    #[test]
    fn lossy_cast_unwrap_saturating_to_t() {
        let x = FiniteInterval::closed(Saturating(0_i32), Saturating(10));
        let y: FiniteInterval<i32> = x.lossy_cast();
        assert_eq!(y, FiniteInterval::closed(0_i32, 10));
    }

    #[test]
    fn try_cast_reflexive_saturating() {
        let x = FiniteInterval::closed(Saturating(0_i32), Saturating(10));
        let y: FiniteInterval<Saturating<i32>> = x.try_cast().unwrap();
        assert_eq!(y, FiniteInterval::closed(Saturating(0_i32), Saturating(10)));
    }

    /// Tier-1 `Cast` is intentionally **not** provided for any
    /// `Saturating<T>` pair; lifting `T → Saturating<T>` changes
    /// downstream arithmetic semantics and must be an explicit opt-in
    /// via `TryCast`/`LossyCast`. This compile-fail doctest guards the
    /// decision from regressing.
    ///
    /// ```compile_fail
    /// use intervalsets_core::cast::Cast;
    /// use intervalsets_core::prelude::*;
    /// use core::num::Saturating;
    ///
    /// let x = FiniteInterval::closed(0_i32, 10);
    /// let _y: FiniteInterval<Saturating<i32>> = x.cast();
    /// ```
    #[allow(dead_code)]
    fn _cast_lift_must_not_compile() {}
}

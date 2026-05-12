use super::Measurement;
use crate::numeric::Zero;
use crate::ops::math::TryAdd;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// The width of a set cannot be represented in
/// [`Width::Output`] (e.g. width of `[i128::MIN, i128::MAX]` overflows
/// `u128`, or `f64::MAX - f64::MIN` overflows `f64` to `±INF`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("width overflows the Width Output type or is non-finite")]
pub struct WidthOverflowError;

impl From<core::convert::Infallible> for WidthOverflowError {
    fn from(x: core::convert::Infallible) -> Self {
        match x {}
    }
}

impl From<crate::error::MathError> for WidthOverflowError {
    /// Lifts a value-level overflow during width summation into the
    /// width-overflow umbrella. Used by `IntervalSet::try_width` to
    /// surface mid-fold `TryAdd` overflow as a width-side failure.
    fn from(_: crate::error::MathError) -> Self {
        WidthOverflowError
    }
}

/// Defines the [width measure](https://en.wikipedia.org/wiki/Lebesgue_measure) of a set in R1.
///
/// The width is defined as the absolute difference between the
/// greatest and least elements within the interval set. If one or
/// more sides is unbounded, the width is infinite.
///
/// > Mathematically speaking, the width of any Countable set is 0.
/// > We *do* allow calculating the width over the Reals between two
/// > integer bounds, however unexpected results may occur due to
/// > discrete normalization. For discrete `T`, prefer
/// > [`Count`](crate::measure::Count).
///
/// Mirrors [`Count`](crate::measure::Count) in shape: every impl
/// provides a fallible [`try_width`](Width::try_width) that surfaces
/// representation overflow, and a default [`width`](Width::width)
/// that panics on overflow for convenience.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// let interval = EnumInterval::closed(10.0, 100.0);
/// assert_eq!(interval.width().finite(), 90.0);
/// ```
///
/// # Normalization problem
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let a = EnumInterval::closed(0.0, 10.0);
/// assert_eq!(a.width().finite(), 10.0);
///
/// let b = EnumInterval::open(0, 10);
/// assert_eq!(b.width().finite(), 8u128);
/// ```
pub trait Width {
    /// The type produced by a successful width computation.
    type Output;
    /// The error returned when the width cannot be represented in [`Self::Output`].
    type Error: core::error::Error;

    /// Compute the width measure of this set.
    ///
    /// # Panics
    ///
    /// Panics if the width cannot be represented in [`Self::Output`]
    /// (e.g. width of `[i128::MIN, i128::MAX]` overflows `u128`, or a
    /// float diff overflows to `±INF`). For panic-free width
    /// computation, use [`try_width`](Width::try_width).
    fn width(&self) -> Measurement<Self::Output> {
        self.try_width()
            .expect("Width::width: representation overflow; use try_width for panic-free")
    }

    /// Compute the width measure of this set, returning `Err` if the
    /// width cannot be represented in [`Self::Output`].
    fn try_width(&self) -> Result<Measurement<Self::Output>, Self::Error>;
}

/// Defines whether a set of type `T` has a width measure, and the
/// type used to represent that width.
///
/// [`Width`] delegates to the underlying type that implements
/// [`Widthable`]. Library impls cover primitive integers (widening to
/// `u128`), floats, `BigInt`/`BigUint`/`BigDecimal`, `Decimal`, and
/// the `fixed::Fixed*` family. Downstream users extend via
/// [`default_width_impl!`](crate::default_width_impl) for
/// arbitrary-precision types or a bespoke impl for types with
/// representation overflow.
pub trait Widthable {
    /// The type used to represent a width. May be wider than `Self`
    /// (e.g. primitive integers widen to `u128` so `[i32::MIN, i32::MAX]`
    /// fits without overflow).
    type Output;

    /// Compute `right - left` as a width. The interval invariant
    /// guarantees `right >= left`. Returns `None` when the width
    /// cannot be represented in [`Self::Output`] (e.g. extreme
    /// `Decimal`, `f64::MIN..f64::MAX` overflowing to `INF`,
    /// `i128::MIN..i128::MAX` overflowing `u128`).
    fn width_between(left: &Self, right: &Self) -> Option<Self::Output>;
}

/// Implements [`Widthable`] for arbitrary-precision types where
/// `&T - &T = T` cannot fail.
#[macro_export]
macro_rules! default_width_impl {
    ($t_in_out:ty) => {
        impl $crate::measure::Widthable for $t_in_out {
            type Output = $t_in_out;

            fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
                Some(right - left)
            }
        }
    };
}

/// Implements [`Widthable`] for native primitive integer types
/// narrower than 128 bits. `Output` is always [`u128`]; the input is
/// widened to [`i128`] before subtraction, so no intermediate overflow
/// is possible. Always returns `Some`.
macro_rules! primitive_width_impl {
    ($($t:ty),+ $(,)?) => {
        $(
            impl $crate::measure::Widthable for $t {
                type Output = u128;

                fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
                    // Interval invariant: right >= left, so the i128 diff is non-negative.
                    let diff = (*right as i128) - (*left as i128);
                    Some(diff as u128)
                }
            }
        )+
    };
}

primitive_width_impl!(u8, u16, u32, u64, usize);
primitive_width_impl!(i8, i16, i32, i64, isize);

// 128-bit types need bespoke handling.
impl Widthable for u128 {
    type Output = u128;

    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        // Interval invariant: right >= left, so checked_sub never underflows.
        right.checked_sub(*left)
    }
}

impl Widthable for i128 {
    type Output = u128;

    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        // Interval invariant: right >= left. The wrapping i128 difference
        // reinterpreted as u128 yields the true unsigned distance, up to
        // 2^128 - 1 (e.g. `[i128::MIN, i128::MAX]` ⇒ u128::MAX).
        Some(right.wrapping_sub(*left) as u128)
    }
}

impl Widthable for f32 {
    type Output = f32;

    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        let d = right - left;
        // Element::validate already rejects ±INF/NaN bounds at construction;
        // a non-finite diff means the diff itself overflowed.
        d.is_finite().then_some(d)
    }
}

impl Widthable for f64 {
    type Output = f64;

    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        let d = right - left;
        d.is_finite().then_some(d)
    }
}

impl<T> Width for FiniteInterval<T>
where
    T: Widthable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = WidthOverflowError;

    fn try_width(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Measurement::Finite(<Self::Output as Zero>::zero())),
            Some((left, right)) => match T::width_between(left.value(), right.value()) {
                Some(w) => Ok(Measurement::Finite(w)),
                None => Err(WidthOverflowError),
            },
        }
    }
}

impl<T> Width for HalfInterval<T>
where
    T: Widthable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = WidthOverflowError;

    fn try_width(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        Ok(Measurement::Infinite)
    }
}

impl<T> Width for EnumInterval<T>
where
    T: Widthable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = WidthOverflowError;

    fn try_width(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        match self {
            Self::Finite(inner) => inner.try_width(),
            Self::Half(inner) => inner.try_width(),
            Self::Unbounded => Ok(Measurement::Infinite),
        }
    }
}

/// Width of a [`MaybeDisjoint`] is the sum of its pieces' widths.
/// `Connected(iv)` delegates; `Disjoint(a, b)` sums per-piece widths
/// via [`TryAdd`] so an overflowing total surfaces as
/// `WidthOverflowError` rather than wrapping. Infinite from either
/// piece propagates to an infinite total.
///
/// Adds `TryAdd` to the bound chain (matching `IntervalSet`'s
/// per-piece-summing impl in the outer crate).
impl<T, Out> Width for MaybeDisjoint<T>
where
    T: Widthable<Output = Out>,
    Out: Zero + TryAdd<Out, Output = Out>,
    <Out as TryAdd>::Error: Into<WidthOverflowError>,
{
    type Output = Out;
    type Error = WidthOverflowError;

    fn try_width(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        match self {
            Self::Connected(iv) => iv.try_width(),
            Self::Disjoint(a, b) => {
                let aw = a.try_width()?;
                let bw = b.try_width()?;
                aw.try_binop_map(bw, |x, y| x.try_add(y).map_err(Into::into))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};

    #[test]
    fn finite_integer_width_widens_to_u128() {
        let x = EnumInterval::closed(0_i32, 10);
        assert_eq!(x.width().finite(), 10_u128);
    }

    #[test]
    fn finite_signed_full_range_no_overflow() {
        // [i32::MIN, i32::MAX] has width 2^32 - 1, fits in u128.
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        assert_eq!(x.width().finite(), u32::MAX as u128);
    }

    #[test]
    fn finite_unsigned_full_range_no_overflow() {
        let x = EnumInterval::closed(0_u64, u64::MAX);
        assert_eq!(x.width().finite(), u64::MAX as u128);
    }

    #[test]
    fn finite_i128_full_range_at_u128_max() {
        // [i128::MIN, i128::MAX] has width 2^128 - 1, exactly u128::MAX.
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        assert_eq!(x.try_width().unwrap().finite(), u128::MAX);
    }

    #[test]
    fn finite_float_width() {
        let x = EnumInterval::closed(0.0_f64, 10.0);
        assert_eq!(x.width().finite(), 10.0);
    }

    #[test]
    fn float_extreme_range_overflows_to_err() {
        // f64::MIN..f64::MAX has true width ≈ 3.6e308, overflows f64 to INF.
        let x = EnumInterval::closed(f64::MIN, f64::MAX);
        assert!(matches!(x.try_width(), Err(WidthOverflowError)));
    }

    #[test]
    #[should_panic]
    fn float_extreme_range_panics_via_width() {
        // The panicking sibling is documented to panic on overflow.
        let x = EnumInterval::closed(f64::MIN, f64::MAX);
        let _ = x.width();
    }

    #[test]
    fn half_interval_width_is_infinite() {
        use crate::bound::FiniteBound;
        let x = HalfInterval::<i32>::left(FiniteBound::closed(0));
        assert!(x.try_width().unwrap().is_infinite());
    }

    #[test]
    fn unbounded_width_is_infinite() {
        let x: EnumInterval<i32> = EnumInterval::Unbounded;
        assert!(x.try_width().unwrap().is_infinite());
    }

    // ===== MaybeDisjoint =====

    #[test]
    fn md_empty_width_is_zero() {
        let x = MaybeDisjoint::<i32>::empty();
        assert_eq!(x.width().finite(), 0_u128);
    }

    #[test]
    fn md_connected_delegates_to_inner_width() {
        let x = MaybeDisjoint::from_interval(EnumInterval::closed(0, 10));
        assert_eq!(x.width().finite(), 10_u128);
    }

    #[test]
    fn md_disjoint_width_sums_pieces() {
        // [0, 5] ∪ [10, 20] → 5 + 10 = 15
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 20));
        assert_eq!(x.width().finite(), 15_u128);
    }

    #[test]
    fn md_disjoint_with_half_interval_is_infinite() {
        // Disjoint(finite, half) — half-piece makes total width infinite.
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i32, 5),
            EnumInterval::closed_unbound(10),
        );
        assert!(x.try_width().unwrap().is_infinite());
    }

    #[test]
    fn md_disjoint_width_sum_overflow_surfaces_err() {
        // f64::MIN..0 and 1..f64::MAX each have finite per-piece width,
        // but their sum overflows f64 to INF and surfaces as Err.
        let a = EnumInterval::closed(f64::MIN, 0.0_f64);
        let b = EnumInterval::closed(1.0, f64::MAX);
        let x = MaybeDisjoint::from_pair(a, b);
        assert!(matches!(x.try_width(), Err(WidthOverflowError)));
    }
}

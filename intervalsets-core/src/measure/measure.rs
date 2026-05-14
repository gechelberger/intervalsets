use super::Extent;
use crate::error::MathError;
use crate::numeric::{Element, Zero};
use crate::ops::math::TryAdd;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// The natural additive measure of a set.
///
/// `Measure` returns whatever measure is natural for the underlying
/// element type `T`:
///
/// - **Discrete `T`** (e.g. `i32`, `u128`, `BigInt`, fixed-point):
///   counting cardinality. `[0, 10]: i32` → `Finite(11_u64)`.
/// - **Continuous `T`** (e.g. `f64`, `Decimal`, `BigDecimal`,
///   `OrderedFloat<f64>`): Lebesgue width. `[0.0, 10.0]: f64` →
///   `Finite(10.0)`. A singleton `[5.0, 5.0]` has measure `0.0` (the
///   natural measure on continuous T is width, not counting).
///
/// The kind-projection happens at the [`Element`] layer:
/// `Output = T::Measure`. The integer-primitive impls use stepwise
/// widening (`i32::Measure = u64`, etc.); see [`Element::Measure`].
///
/// For users who want diameter `sup − inf` on any T (e.g. `b − a` on
/// an integer interval, in its native type), see
/// [`Span`](crate::ops::Span).
///
/// # Three-state contract
///
/// Every call returns `Result<Extent<Self::Output>, Self::Error>`:
///
/// | Outcome | Return |
/// |---|---|
/// | Finite-bounded, arithmetic succeeded | `Ok(Extent::Finite(_))` |
/// | Half-bounded or fully unbounded set | `Ok(Extent::Infinite)` |
/// | Finite-bounded, representation overflow (e.g. `[i128::MIN, i128::MAX]`) | `Err(_)` |
///
/// `Extent::Infinite` is **never** used as a fallback for overflow —
/// that surfaces as `Err`.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// // Discrete: cardinality.
/// let a = EnumInterval::closed(0_i32, 10);
/// assert_eq!(a.measure().finite(), 11_u64);
///
/// // Continuous: Lebesgue width.
/// let b = EnumInterval::closed(0.0_f64, 10.0);
/// assert_eq!(b.measure().finite(), 10.0);
///
/// // Half-bounded → Infinite.
/// let c = EnumInterval::closed_unbound(0_i32);
/// assert!(c.measure().is_infinite());
/// ```
pub trait Measure {
    /// The natural measure value type for the underlying element.
    type Output;
    /// The error returned when the measure cannot be represented in
    /// `Self::Output`.
    type Error: core::error::Error;

    /// Compute the measure of this set.
    ///
    /// # Panics
    ///
    /// Panics if the measure cannot be represented in `Self::Output`
    /// (e.g. `[i128::MIN, i128::MAX].measure()` overflows `u128`). For
    /// panic-free use, call [`try_measure`](Measure::try_measure).
    fn measure(&self) -> Extent<Self::Output> {
        self.try_measure()
            .expect("Measure::measure: representation overflow; use try_measure for panic-free")
    }

    /// Compute the measure of this set, returning `Err` if it
    /// overflows `Self::Output`.
    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

impl<T> Measure for FiniteInterval<T>
where
    T: Element,
{
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Extent::Finite(<T::Measure as Zero>::zero())),
            Some((left, right)) => match T::try_measure_finite(left.value(), right.value()) {
                Some(m) => Ok(Extent::Finite(m)),
                None => Err(MathError::Range),
            },
        }
    }
}

impl<T> Measure for HalfInterval<T>
where
    T: Element,
{
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        Ok(Extent::Infinite)
    }
}

impl<T> Measure for EnumInterval<T>
where
    T: Element,
{
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Finite(inner) => inner.try_measure(),
            Self::Half(inner) => inner.try_measure(),
            Self::Unbounded => Ok(Extent::Infinite),
        }
    }
}

/// Measure of a [`MaybeDisjoint`] is the sum of its pieces' measures.
/// `Connected(iv)` delegates; `Disjoint(a, b)` sums via [`TryAdd`] so
/// an overflowing total surfaces as [`MathError`] rather than
/// wrapping. `Infinite` from either piece propagates.
impl<T> Measure for MaybeDisjoint<T>
where
    T: Element,
    <T::Measure as TryAdd>::Error: Into<MathError>,
{
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Connected(iv) => iv.try_measure(),
            Self::Disjoint(a, b) => {
                let am = a.try_measure()?;
                let bm = b.try_measure()?;
                am.try_binop_map(bm, |x, y| x.try_add(y).map_err(Into::into))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

    // ===== Discrete: cardinality semantics =====

    #[test]
    fn discrete_finite_cardinality() {
        let x = EnumInterval::closed(0_i32, 10);
        assert_eq!(x.measure().finite(), 11_u64);
    }

    #[test]
    fn discrete_full_i32_range_fits_u64() {
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        // 2^32 elements; fits in u64.
        assert_eq!(x.measure().finite(), 1u64 << 32);
    }

    #[test]
    fn discrete_full_u64_range_fits_u128() {
        let x = EnumInterval::closed(0_u64, u64::MAX);
        assert_eq!(x.measure().finite(), 1u128 << 64);
    }

    #[test]
    fn discrete_i128_min_max_overflows() {
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        assert!(matches!(x.try_measure(), Err(MathError::Range)));
    }

    #[test]
    fn discrete_u128_full_range_overflows() {
        let x = EnumInterval::closed(0_u128, u128::MAX);
        assert!(matches!(x.try_measure(), Err(MathError::Range)));
    }

    #[test]
    #[should_panic]
    fn measure_overflow_panics() {
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        let _ = x.measure();
    }

    // ===== Continuous: Lebesgue width semantics =====

    #[test]
    fn continuous_finite_width() {
        let x = EnumInterval::closed(0.0_f64, 10.0);
        assert_eq!(x.measure().finite(), 10.0);
    }

    #[test]
    fn continuous_singleton_is_zero_width() {
        // Note: pre-unification `cardinality([5.0, 5.0])` returned `Finite(1)`.
        // Under unification the natural measure on continuous T is
        // Lebesgue width, so a singleton has measure 0.0.
        let x = EnumInterval::closed(5.0_f64, 5.0);
        assert_eq!(x.measure().finite(), 0.0);
    }

    #[test]
    fn continuous_empty_is_zero() {
        let x = FiniteInterval::<f64>::empty();
        assert_eq!(x.measure().finite(), 0.0);
    }

    #[test]
    fn continuous_float_overflow_surfaces_err() {
        // f64::MIN..f64::MAX has true width ≈ 3.6e308; overflows f64.
        let x = EnumInterval::closed(f64::MIN, f64::MAX);
        assert!(matches!(x.try_measure(), Err(MathError::Range)));
    }

    // ===== Structural Infinite =====

    #[test]
    fn half_interval_is_infinite() {
        let x = HalfInterval::<i32>::closed_unbound(0);
        assert!(x.try_measure().unwrap().is_infinite());
    }

    #[test]
    fn unbounded_is_infinite() {
        let x = EnumInterval::<i32>::unbounded();
        assert!(x.try_measure().unwrap().is_infinite());
    }

    // ===== MaybeDisjoint =====

    #[test]
    fn md_empty_is_zero() {
        let x = MaybeDisjoint::<i32>::empty();
        assert_eq!(x.measure().finite(), 0_u64);
    }

    #[test]
    fn md_connected_delegates() {
        let x = MaybeDisjoint::from_interval(EnumInterval::closed(0_i32, 10));
        assert_eq!(x.measure().finite(), 11_u64);
    }

    #[test]
    fn md_disjoint_sums_pieces() {
        // [0, 5] (6 elements) ∪ [10, 20] (11 elements) → 17
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 20));
        assert_eq!(x.measure().finite(), 17_u64);
    }

    #[test]
    fn md_disjoint_with_half_is_infinite() {
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i32, 5),
            EnumInterval::closed_unbound(10),
        );
        assert!(x.try_measure().unwrap().is_infinite());
    }

    #[test]
    fn md_per_piece_overflow_propagates() {
        let inner = EnumInterval::closed(i128::MIN, i128::MAX);
        let md = MaybeDisjoint::from_interval(inner);
        assert!(md.try_measure().is_err());
    }

    #[test]
    fn md_disjoint_continuous_widths_sum() {
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 1.0),
            EnumInterval::closed(2.0, 5.0),
        );
        // widths 1.0 + 3.0 = 4.0
        assert_eq!(x.measure().finite(), 4.0);
    }
}

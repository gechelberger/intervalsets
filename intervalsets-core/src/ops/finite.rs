//! `IntoFiniteInterval` itruncates a `Set` to the smallest FiniteInterval that
//! covers the elements of the original `Set` which can be represented by the
//! storage-type T: Bounded + Element.
//!
//! # Notes
//!
//! todo: require min/max of T, some elements (BigDecimal) inherently do not have this, so
//! for truncation to FiniteInterval, instead need to define some Subset on T that defines
//! the desired universe and intersect with that.

use crate::bound::{BoundType, FiniteBound, Side};
use crate::numeric::Element;
use crate::{EnumInterval, FiniteInterval, HalfInterval};

/// Truncates a set to the universe of elements representable by the generic data type.
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). Cannot panic
/// or error given inputs satisfying their type invariants; no
/// `try_*` variant because the operation introduces no logical
/// violation of its own. See [`crate::ops`] for the full tier model.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let positive = EnumInterval::closed_unbound(1i8);
/// assert_eq!(positive.count(), Measurement::Infinite);
/// let as_finite = positive.into_finite();
/// assert_eq!(as_finite.count(), Measurement::Finite(127));
/// ```
pub trait IntoFinite {
    /// The type of set to create.
    type Output;

    /// Converts to a finite set.
    fn into_finite(self) -> Self::Output;
}

impl<T> IntoFinite for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn into_finite(self) -> Self::Output {
        self
    }
}

impl<T: Element + num_traits::Bounded> IntoFinite for HalfInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn into_finite(self) -> Self::Output {
        // An open bound at the type's saturating extreme (e.g.
        // `(255, ->)` for u8) describes an empty set after truncation.
        // The half-bounded `bound` came from a validated interval, so
        // I2 + I4 hold; the Tier-3 helper evaluates the pair
        // satisfiability against the saturating closed bound.
        let (side, bound) = self.into_raw();
        match side {
            Side::Left => super::intersection::from_normed_pair(
                bound,
                FiniteBound::new_assume_valid(BoundType::Closed, T::max_value()),
            ),
            Side::Right => super::intersection::from_normed_pair(
                FiniteBound::new_assume_valid(BoundType::Closed, T::min_value()),
                bound,
            ),
        }
    }
}

impl<T: Element + num_traits::Bounded> IntoFinite for EnumInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn into_finite(self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.into_finite(),
            Self::Half(inner) => inner.into_finite(),
            Self::Unbounded => FiniteInterval::new_assume_valid(
                FiniteBound::new_assume_valid(BoundType::Closed, T::min_value()),
                FiniteBound::new_assume_valid(BoundType::Closed, T::max_value()),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_into_finite() {
        assert_eq!(
            EnumInterval::<i8>::closed_unbound(0).into_finite(),
            FiniteInterval::closed(0, 127)
        );

        assert_eq!(
            EnumInterval::<i32>::unbound_open(0).into_finite(),
            FiniteInterval::closed(-2147483648, -1)
        );

        assert_eq!(
            EnumInterval::<u8>::unbounded().into_finite(),
            FiniteInterval::<u8>::closed(0, 255)
        );

        assert_eq!(
            EnumInterval::<u8>::open_unbound(255).into_finite(),
            FiniteInterval::empty()
        );

        assert_eq!(
            HalfInterval::<u8>::unbound_open(0).into_finite(),
            FiniteInterval::empty()
        );
    }
}

use crate::bound::{FiniteBound, Side};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

/// Truncates a set to the universe of elements representable by the generic data type.
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

impl<T: num_traits::Bounded + PartialOrd> IntoFinite for HalfInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn into_finite(self) -> Self::Output {
        let (side, bound) = self.into_raw();
        match side {
            Side::Left => unsafe {
                FiniteInterval::new_assume_normed(bound, FiniteBound::closed(T::max_value()))
            },
            Side::Right => unsafe {
                FiniteInterval::new_assume_normed(FiniteBound::closed(T::min_value()), bound)
            },
        }
    }
}

impl<T: num_traits::Bounded + PartialOrd> IntoFinite for EnumInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn into_finite(self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.into_finite(),
            Self::Half(inner) => inner.into_finite(),
            Self::Unbounded => unsafe {
                FiniteInterval::new_unchecked(
                    FiniteBound::closed(T::min_value()),
                    FiniteBound::closed(T::max_value()),
                )
            },
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

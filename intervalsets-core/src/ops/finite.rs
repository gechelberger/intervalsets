use crate::bound::{FiniteBound, Side};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

pub trait IntoFinite {
    type Output;

    fn into_finite(self) -> Self::Output;
}

impl<T> IntoFinite for FiniteInterval<T> {
    type Output = Self;

    fn into_finite(self) -> Self::Output {
        self
    }
}

impl<T: num_traits::Bounded + PartialOrd> IntoFinite for HalfInterval<T> {
    type Output = FiniteInterval<T>;

    fn into_finite(self) -> Self::Output {
        match self.side {
            Side::Left => unsafe {
                FiniteInterval::new_norm(self.bound, FiniteBound::closed(T::max_value()))
            },
            Side::Right => unsafe {
                FiniteInterval::new_norm(FiniteBound::closed(T::min_value()), self.bound)
            },
        }
    }
}

impl<T: num_traits::Bounded + PartialOrd> IntoFinite for EnumInterval<T> {
    type Output = FiniteInterval<T>;

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

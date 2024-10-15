use core::ops::Sub;
use num_traits::Zero;

use crate::{FiniteInterval, HalfInterval, ISize, Interval, IntervalSet};

/// A measure of the size of the set S in R1.
///
/// The Lebesgue measure for R^1.
///
/// The lebesgue measure for any countable set should be 0. We assume
/// that if the width of a set is being used, it is over the reals even
/// if the underlying datatype are integers.
pub trait Width {
    type Output;

    fn width(&self) -> Self::Output;
}

impl<T> Width for FiniteInterval<T>
where
    T: Clone + Sub<T, Output = T> + Zero,
{
    type Output = T;

    fn width(&self) -> Self::Output {
        match self {
            Self::Empty => T::zero(),
            Self::NonZero(left, right) => right.value.clone() - left.value.clone(),
        }
    }
}

impl<T> Width for HalfInterval<T> {
    type Output = ISize<T>;

    fn width(&self) -> Self::Output {
        ISize::Infinite
    }
}

impl<T> Width for Interval<T>
where
    T: Clone + Sub<T, Output = T> + Zero,
{
    type Output = ISize<T>;

    fn width(&self) -> Self::Output {
        match self {
            Self::Finite(inner) => ISize::Finite(inner.width()),
            Self::Half(inner) => inner.width(),
            Self::Infinite => ISize::Infinite,
        }
    }
}

impl<T> Width for IntervalSet<T>
where
    T: Clone + Sub<T, Output = T> + Zero,
{
    type Output = ISize<T>;

    fn width(&self) -> Self::Output {
        self.intervals
            .iter()
            .fold(ISize::Finite(T::zero()), |accum, subset| {
                accum + subset.width()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_width() {
        assert_eq!(FiniteInterval::<i32>::Empty.width(), 0);
        assert_eq!(FiniteInterval::closed(0, 10).width(), 10);
    }
}

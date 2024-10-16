use core::ops::Sub;
use num_traits::Zero;

use super::Measurement as M;
use crate::{FiniteInterval, HalfBounded, EBounds, IntervalSet};

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

impl<T, Out> Width for FiniteInterval<T>
where
    Out: Zero,
    T: Clone + Sub<T, Output = Out>,
{
    type Output = M<Out>;

    fn width(&self) -> Self::Output {
        match self {
            Self::Empty => M::Finite(Out::zero()),
            Self::FullyBounded(left, right) => M::Finite(right.value.clone() - left.value.clone()),
        }
    }
}
impl<T, Out> Width for HalfBounded<T>
where
    Out: Zero,
    T: Clone + Sub<T, Output = Out>,
{
    type Output = M<Out>;

    fn width(&self) -> Self::Output {
        M::Infinite
    }
}

impl<T, Out> Width for EBounds<T>
where
    Out: Zero,
    T: Clone + Sub<T, Output = Out>,
{
    type Output = M<Out>;

    fn width(&self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.width(),
            Self::Half(inner) => inner.width(),
            Self::Unbounded => M::Infinite,
        }
    }
}

impl<T, Out> Width for IntervalSet<T>
where
    Out: Clone + Zero + core::ops::Add<Out, Output = Out>,
    T: Clone + core::ops::Sub<T, Output = Out>,
{
    type Output = M<Out>;

    fn width(&self) -> Self::Output {
        self.intervals
            .iter()
            .fold(M::Finite(Out::zero()), |accum, subset| {
                accum + subset.width()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_width() {
        assert_eq!(FiniteInterval::<i32>::Empty.width().finite(), 0);
        assert_eq!(FiniteInterval::closed(0, 10).width().finite(), 10);
    }
}

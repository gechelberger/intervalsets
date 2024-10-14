use std::ops::{Add, Sub};

use num_traits::Zero;

use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum ISize<T> {
    Finite(T),
    Infinite,
}

/// Required by Zero trait for some reason
impl<T: Add<Output = T>> Add for ISize<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ISize::Infinite, _) => ISize::Infinite,
            (_, ISize::Infinite) => ISize::Infinite,
            (ISize::Finite(lhs), ISize::Finite(rhs)) => ISize::Finite(lhs + rhs),
        }
    }
}

/// Implement Zero for ISize
impl<T: Zero + Eq + Add<Output = T>> Zero for ISize<T> {
    fn zero() -> Self {
        Self::Finite(T::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Infinite => false,
            Self::Finite(wrapped) => *wrapped == T::zero(),
        }
    }
}

pub trait Sizable {
    type Output: Zero + Eq;

    fn size(&self) -> Self::Output;

    fn is_empty(&self) -> bool {
        self.size() == Self::Output::zero()
    }
}

impl<T: Zero + Sub<Output = T> + Copy + Eq> Sizable for Interval<T> {
    type Output = ISize<T>;

    fn size(&self) -> Self::Output {
        match self {
            Self::Infinite => ISize::Infinite,
            Self::Half(_) => ISize::Infinite,
            Self::Finite(finite) => ISize::Finite(finite.size()),
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::Finite(FiniteInterval::Empty)
    }
}

impl<T: Zero + Sub<Output = T> + Copy + Eq> Sizable for FiniteInterval<T> {
    type Output = T;

    fn size(&self) -> Self::Output {
        match self {
            Self::Empty => T::zero(),
            Self::NonZero(left, right) => right.value - left.value,
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::Empty
    }
}

impl<T: Eq + Zero> Sizable for HalfInterval<T> {
    type Output = ISize<T>;

    fn size(&self) -> Self::Output {
        ISize::Infinite
    }
}

impl<T: Copy + Eq + Zero + Sub<Output = T>> Sizable for IntervalSet<T> {
    type Output = ISize<T>;

    fn size(&self) -> Self::Output {
        self.intervals
            .iter()
            .map(|itv| itv.size())
            .fold(ISize::zero(), ISize::add)
    }
}

#[cfg(test)]
mod test {
    use crate::Normalize;

    use super::*;

    #[test]
    fn test_finite_size() {
        let interval = FiniteInterval::open(0, 20).normalized();
        assert_eq!(interval.size(), 18);

        let interval = Interval::open(0, 20).normalized();
        assert_eq!(interval.size(), ISize::Finite(18));
    }

    #[test]
    fn test_interval_set_size() {
        let set = IntervalSet {
            intervals: vec![Interval::closed(0, 20), Interval::closed(30, 50)],
        };
        assert_eq!(set.size(), ISize::Finite(40));
    }
}

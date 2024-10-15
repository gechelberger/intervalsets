use num_traits::Zero;

use crate::{Domain, FiniteInterval, ISize, Interval, Side};

pub trait Count {
    type Output;

    fn count(&self) -> Self::Output;
}

impl<T: Countable + Zero> Count for FiniteInterval<T> {
    type Output = ISize<T>;

    fn count(&self) -> Self::Output {
        match self {
            Self::Empty => ISize::Finite(T::zero()),
            Self::NonZero(left, right) => T::count_between(&left.value, &right.value)
                .map_or(ISize::Infinite, |c| ISize::Finite(c)),
        }
    }
}

impl<T: Countable + Zero> Count for Interval<T> {
    type Output = ISize<T>;

    fn count(&self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.count(),
            _ => ISize::Infinite,
        }
    }
}

/// This trait allows delegation to the underlying type
/// for how their elements should be counted.
///
/// The default implementation assumes integer like behavior,
/// meaning an even distribution of elements 1 apart.
pub trait Countable: Domain + core::ops::Sub<Self, Output = Self> {
    fn count_between(left: &Self, right: &Self) -> Option<Self> {
        // this has a bug at max value...
        right
            .try_adjacent(Side::Right)
            .map(|adjacent| adjacent.clone() - left.clone())
    }
}

impl Countable for i8 {}
impl Countable for i16 {}
impl Countable for i32 {}
impl Countable for i64 {}
impl Countable for isize {}

impl Countable for u8 {}
impl Countable for u16 {}
impl Countable for u32 {}
impl Countable for u64 {}
impl Countable for usize {}

#[cfg(test)]
mod tests {
    use crate::FiniteInterval;

    use super::*;

    #[test]
    fn test_finit_count() {
        assert_eq!(FiniteInterval::closedopen(0, 10).count(), ISize::Finite(10));

        assert_eq!(Interval::closed(0, 5).count(), ISize::Finite(6));

        assert_eq!(Interval::unbound_closed(0).count(), ISize::Infinite);
    }
}

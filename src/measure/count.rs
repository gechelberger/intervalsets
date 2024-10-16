use num_traits::Zero;

use super::Measurement as M;
use crate::{Domain, FiniteInterval, HalfBounded, EBounds, Side};

/// Defines the measure of a Countable Interval/Set.
///
///
pub trait Count {
    type Output;

    fn count(&self) -> Self::Output;
}

impl<T: Countable + Zero> Count for FiniteInterval<T> {
    type Output = M<T>;

    fn count(&self) -> Self::Output {
        match self {
            Self::Empty => M::Finite(T::zero()),
            Self::FullyBounded(left, right) => {
                T::count_inclusive(&left.value, &right.value).map_or(M::Infinite, |c| M::Finite(c))
            }
        }
    }
}

impl<T> Count for HalfBounded<T> {
    type Output = M<T>;

    fn count(&self) -> Self::Output {
        M::Infinite
    }
}

impl<T: Countable + Zero> Count for EBounds<T> {
    type Output = M<T>;

    fn count(&self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.count(),
            _ => M::Infinite,
        }
    }
}

/// The `Countable` trait delegates to the underlying type for how their
/// elements should be counted.
///
/// The default implementation assumes an integer-like underlying type.
pub trait Countable<Out = Self>: Domain + core::ops::Sub<Self, Output = Out> {
    fn count_inclusive(left: &Self, right: &Self) -> Option<Out> {
        // TODO: this has a bug at max value...
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
        assert_eq!(FiniteInterval::closed_open(0, 10).count().finite(), 10);

        assert_eq!(EBounds::closed(0, 5).count().finite(), 6);

        assert_eq!(EBounds::unbound_closed(0).count(), M::Infinite);
    }
}

use core::ops::{Add, Sub};
use num_traits::Zero;

use crate::{Domain, Interval, IntervalSet, Side};

use super::Measurement;

/// Defines the counting measure of a [`Countable`] Interval/Set.
///
///
pub trait Count {
    type Output;

    fn count(&self) -> Measurement<Self::Output>;
}

/// Defines whether a set of type T is countable.
///
/// [`Count`] delegates to the underlying type that implements [`Countable`].
///
/// The default implementation relies on Domain implementations for **discrete**
/// data types.
pub trait Countable: Domain + Sub<Self> {
    fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
        // TODO: this has a bug at max value...
        right
            .try_adjacent(Side::Right)
            .map(|adjacent| adjacent.clone() - left.clone())
    }
}

impl<T, Out> Count for Interval<T>
where
    Out: Zero,
    T: Countable<Output = Out>,
{
    type Output = <T as core::ops::Sub>::Output;

    fn count(&self) -> Measurement<Self::Output> {
        self.0.count()
    }
}

impl<T, Out> Count for IntervalSet<T>
where
    Out: Zero + Clone,
    T: Countable<Output = Out> + Add<Out, Output = Out>,
{
    type Output = Out;

    fn count(&self) -> Measurement<Self::Output> {
        self.intervals()
            .iter()
            .map(|subset| subset.count())
            .fold(Measurement::Finite(Out::zero()), |accum, item| accum + item)
    }
}

#[cfg(test)]
mod tests {}

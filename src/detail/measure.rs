use core::ops::Sub;

use num_traits::Zero;

use crate::measure::count::{Count, Countable};
use crate::measure::width::Width;
use crate::measure::Measurement;
use crate::Domain;

use super::{BoundCase, Finite, HalfBounded};

impl<T, Out> Width for Finite<T>
where
    Out: num_traits::Zero,
    T: Domain + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        match self {
            Self::Empty => Measurement::Finite(Out::zero()),
            Self::FullyBounded(left, right) => {
                Measurement::Finite(right.value().clone() - left.value().clone())
            }
        }
    }
}

impl<T, Out> Width for HalfBounded<T>
where
    Out: num_traits::Zero,
    T: Domain + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        Measurement::Infinite
    }
}

impl<T, Out> Width for BoundCase<T>
where
    Out: num_traits::Zero,
    T: Domain + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Finite(inner) => inner.width(),
            Self::Half(inner) => inner.width(),
            Self::Unbounded => Measurement::Infinite,
        }
    }
}

impl<T, Out> Count for Finite<T>
where
    Out: Zero,
    T: Countable<Output = Out>,
{
    type Output = Out;

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::FullyBounded(left, right) => {
                let count = T::count_inclusive(left.value(), right.value())
                    .expect("Count should be Some since interval is FullyBounded");
                Measurement::Finite(count)
            }
            Self::Empty => Measurement::Finite(Self::Output::zero()),
        }
    }
}

impl<T, Out> Count for BoundCase<T>
where
    Out: Zero,
    T: Countable<Output = Out>,
{
    type Output = <T as Sub>::Output;

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Finite(inner) => inner.count(),
            _ => Measurement::Infinite,
        }
    }
}

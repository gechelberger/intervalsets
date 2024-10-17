use core::ops::Sub;

use crate::numeric::LibZero;

use crate::measure::{Count, Countable, Measurement, Width};
use crate::Domain;

use super::{BoundCase, Finite, HalfBounded};

impl<T, Out> Width for Finite<T>
where
    Out: LibZero,
    T: Domain + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        match self {
            Self::Empty => Measurement::Finite(Out::new_zero()),
            Self::FullyBounded(left, right) => {
                Measurement::Finite(right.value().clone() - left.value().clone())
            }
        }
    }
}

impl<T, Out> Width for HalfBounded<T>
where
    Out: LibZero,
    T: Domain + Sub<T, Output = Out>,
{
    type Output = Out;

    fn width(&self) -> Measurement<Self::Output> {
        Measurement::Infinite
    }
}

impl<T, Out> Width for BoundCase<T>
where
    Out: LibZero,
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

impl<T> Count for Finite<T>
where
    T: Countable,
    T::Output: LibZero,
{
    type Output = T::Output;

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::FullyBounded(left, right) => {
                let count = T::count_inclusive(left.value(), right.value())
                    .expect("Count should be Some since interval is FullyBounded");
                Measurement::Finite(count)
            }
            Self::Empty => Measurement::Finite(Self::Output::new_zero()),
        }
    }
}

impl<T> Count for BoundCase<T>
where
    T: Countable,
    T::Output: LibZero,
{
    type Output = T::Output;

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Finite(inner) => inner.count(),
            _ => Measurement::Infinite,
        }
    }
}

use crate::{Domain, Interval};

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> From<Finite<T>> for BoundCase<T> {
    fn from(value: Finite<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T: Domain> From<HalfBounded<T>> for BoundCase<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self::Half(value)
    }
}

impl<T: Domain> From<BoundCase<T>> for Interval<T> {
    fn from(value: BoundCase<T>) -> Self {
        Self(value)
    }
}

impl<T: Domain> From<Finite<T>> for Interval<T> {
    fn from(value: Finite<T>) -> Self {
        Self::from(BoundCase::from(value))
    }
}

impl<T: Domain> From<HalfBounded<T>> for Interval<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self::from(BoundCase::from(value))
    }
}

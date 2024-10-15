use crate::normalize::Normalize;
use crate::numeric::Domain;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

impl<T: Domain> From<FiniteInterval<T>> for Interval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value).normalized()
    }
}

impl<T: Domain> From<HalfInterval<T>> for Interval<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::Half(value).normalized()
    }
}

impl<T: Domain> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        match value {
            Interval::Finite(inner) => Self::from(inner),
            _ => Self::new_unchecked(vec![value]),
        }
    }
}

impl<T: Domain> From<FiniteInterval<T>> for IntervalSet<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        match value {
            FiniteInterval::Empty => Self::empty(),
            _ => Self::new_unchecked(vec![value.into()]),
        }
    }
}

impl<T: Domain> From<HalfInterval<T>> for IntervalSet<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::new_unchecked(vec![value.into()])
    }
}

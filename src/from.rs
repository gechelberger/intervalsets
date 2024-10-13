use crate::{half::HalfInterval, infinite::IntervalSet, FiniteInterval, Interval};

impl<T> From<FiniteInterval<T>> for Interval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T> From<HalfInterval<T>> for Interval<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::Half(value)
    }
}

impl<T> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        Self {
            intervals: vec![value]
        }
    }
}

impl<T> From<FiniteInterval<T>> for IntervalSet<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self {
            intervals: vec![value.into()]
        }
    }
}

impl<T> From<HalfInterval<T>> for IntervalSet<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self {
            intervals: vec![value.into()]
        }
    }
}
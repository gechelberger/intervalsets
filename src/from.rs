use crate::normalize::Normalize;
use crate::numeric::Numeric;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

impl<T: Numeric> From<FiniteInterval<T>> for Interval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value).normalized()
    }
}

impl<T: Numeric> From<HalfInterval<T>> for Interval<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::Half(value).normalized()
    }
}

impl<T: Numeric> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        match value {
            Interval::Finite(inner) => Self::from(inner),
            _ => Self::new_unchecked(vec![value]),
        }
    }
}

impl<T: Numeric> From<FiniteInterval<T>> for IntervalSet<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        match value {
            FiniteInterval::Empty => Self::empty(),
            _ => Self::new_unchecked(vec![value.into()]),
        }
    }
}

impl<T: Numeric> From<HalfInterval<T>> for IntervalSet<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::new_unchecked(vec![value.into()])
    }
}

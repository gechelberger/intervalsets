use crate::numeric::Domain;
use crate::{EBounds, FiniteInterval, HalfBounded, Interval, IntervalSet, MaybeEmpty};

impl<T: Domain> From<FiniteInterval<T>> for EBounds<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T: Domain> From<HalfBounded<T>> for EBounds<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self::Half(value)
    }
}

impl<T: Domain> From<FiniteInterval<T>> for Interval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self(value.into())
    }
}

impl<T: Domain> From<HalfBounded<T>> for Interval<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self(value.into())
    }
}

impl<T: Domain> From<EBounds<T>> for Interval<T> {
    fn from(value: EBounds<T>) -> Self {
        Self(value)
    }
}

impl<T: Domain> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        if value.is_empty() {
            return Self::new_unchecked(vec![]);
        }
        Self::new_unchecked(vec![value.into()])
    }
}

impl<T: Domain> From<FiniteInterval<T>> for IntervalSet<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        let value: Interval<T> = value.into();
        value.into()
    }
}

impl<T: Domain> From<HalfBounded<T>> for IntervalSet<T> {
    fn from(value: HalfBounded<T>) -> Self {
        let value: Interval<T> = value.into();
        value.into()
    }
}

impl<T: Domain> From<EBounds<T>> for IntervalSet<T> {
    fn from(value: EBounds<T>) -> Self {
        let value: Interval<T> = value.into();
        value.into()
    }
}
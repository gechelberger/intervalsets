use crate::{half::HalfInterval, infinite::IntervalSet, FiniteInterval, Interval};


impl<T> Into<Interval<T>> for FiniteInterval<T> {
    fn into(self) -> Interval<T> {
        Interval::Finite(self)
    }
}

impl<T> Into<Interval<T>> for HalfInterval<T> {
    fn into(self) -> Interval<T> {
        Interval::Half(self)
    }
}

impl<T> Into<IntervalSet<T>> for Interval<T> {
    fn into(self) -> IntervalSet<T> {
        IntervalSet { intervals: vec![self] }
    }
}

impl<T> Into<IntervalSet<T>> for HalfInterval<T> {
    fn into(self) -> IntervalSet<T> {
        let interval: Interval<T> = self.into();
        interval.into()
    }
}

impl<T> Into<IntervalSet<T>> for FiniteInterval<T> {
    fn into(self) -> IntervalSet<T> {
        let interval: Interval<T> = self.into();
        interval.into()
    }
}
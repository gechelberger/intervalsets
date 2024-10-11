use crate::{half::HalfInterval, FiniteInterval, Interval};


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
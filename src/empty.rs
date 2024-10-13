/// This trait is intended to fix degeneracy of bounds so we can
/// differentiate between a lack of a bound because the interval
/// is empty or has a bound at infinity.
/// 
/// HalfInterval can not be empty. Do we need to implement this for symmetry?

use crate::{infinite::IntervalSet, FiniteInterval, Interval};

pub(crate) trait MaybeEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> MaybeEmpty for FiniteInterval<T> {
    fn is_empty(&self) -> bool {
        *self == FiniteInterval::Empty
    }
}

impl<T> MaybeEmpty for Interval<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Finite(finite) => finite.is_empty(),
            _ => false
        }
    }
}

impl<T> MaybeEmpty for IntervalSet<T> {
    fn is_empty(&self) -> bool {
        self.intervals.len() == 0
    }
}
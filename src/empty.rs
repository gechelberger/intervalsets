/// This trait is intended to fix degeneracy of bounds so we can
/// differentiate between a lack of a bound because the interval
/// is empty or has a bound at infinity.
///
/// HalfInterval can not be empty so we shouldn't need it and can
/// just skip strait to determinite functions for it.
use crate::{FiniteInterval, Interval, IntervalSet};

pub trait MaybeEmpty {
    fn is_empty(&self) -> bool;
}

impl<T: PartialEq> MaybeEmpty for FiniteInterval<T> {
    fn is_empty(&self) -> bool {
        *self == FiniteInterval::Empty
    }
}

impl<T: PartialEq> MaybeEmpty for Interval<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Finite(finite) => finite.is_empty(),
            _ => false,
        }
    }
}

impl<T> MaybeEmpty for IntervalSet<T> {
    fn is_empty(&self) -> bool {
        self.intervals.len() == 0
    }
}

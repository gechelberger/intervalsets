use crate::{Interval, IntervalSet};

///
///
/// This trait is intended to fix degeneracy of bounds so we can
/// differentiate between a lack of a bound because the interval
/// is empty or has a bound at infinity.
///
/// HalfInterval can not be empty so we shouldn't need it and can
/// just skip strait to determinite functions for it.
pub trait MaybeEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> MaybeEmpty for Interval<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> MaybeEmpty for IntervalSet<T> {
    fn is_empty(&self) -> bool {
        self.intervals().len() == 0
    }
}
use crate::numeric::Domain;
use crate::{Interval, IntervalSet};

/// Defines an item that may be empty and a way to query it.
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

impl<T: Domain> MaybeEmpty for Interval<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Domain> MaybeEmpty for &Interval<T> {
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

impl<T: Domain> MaybeEmpty for IntervalSet<T> {
    fn is_empty(&self) -> bool {
        self.slice().len() == 0
    }
}

impl<T: Domain> MaybeEmpty for &IntervalSet<T> {
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

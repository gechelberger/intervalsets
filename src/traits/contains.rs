use crate::sets::IntervalSet;
use crate::{Domain, Interval};
/// Defines whether a set fully contains another.
///
/// For our purposes a point is the singleton set [T].
///
/// A contains B if and only if
/// for every element x of B,
/// x is also an element of A.
///
/// Contains is not commutative.
///
/// # Example
/// ```
/// use intervalsets::Interval;
/// use intervalsets::Contains;
///
/// let x = Interval::open(0, 10);
/// assert_eq!(x.contains(&5), true);
/// assert_eq!(x.contains(&10), false);
/// assert_eq!(x.contains(&Interval::open(0, 10)), true);
/// ```
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T: Domain> Contains<T> for Interval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.0.contains(rhs)
    }
}

impl<T: Domain> Contains<Self> for Interval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.0.contains(&rhs.0)
    }
}

impl<T: Domain> Contains<IntervalSet<T>> for Interval<T> {
    fn contains(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.intervals().iter().all(|subset| self.contains(subset))
    }
}

impl<T: Domain> Contains<T> for IntervalSet<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.intervals().iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: Domain> Contains<Interval<T>> for IntervalSet<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        todo!()
    }
}

impl<T: Domain> Contains<Self> for IntervalSet<T> {
    fn contains(&self, rhs: &Self) -> bool {
        todo!()
    }
}

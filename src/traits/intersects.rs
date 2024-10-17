use crate::{commutative_predicate_impl, Domain, Interval, IntervalSet};

/// Defines whether two sets intersect.
///
/// For these two sets is there at least one
/// element which is contained in each?
///
/// Intersects is commutative.
///
/// # Example
///
/// ```
/// use intervalsets::Interval;
/// use intervalsets::Intersects;
///
/// let interval = Interval::closed(10, 20);
/// if interval.intersects(&Interval::closed_unbound(15)) {
///     // true: do something
/// }
/// ```
pub trait Intersects<Rhs = Self> {
    fn intersects(&self, rhs: &Rhs) -> bool;

    fn is_disjoint_from(&self, rhs: &Rhs) -> bool {
        !self.intersects(rhs)
    }
}

impl<T: Domain> Intersects<Self> for Interval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.0.intersects(&rhs.0)
    }
}

impl<T: Domain> Intersects<Interval<T>> for IntervalSet<T> {
    fn intersects(&self, rhs: &Interval<T>) -> bool {
        // binary search for
        self.intervals().iter().any(|subset| subset.intersects(rhs))
    }
}
commutative_predicate_impl!(Intersects, intersects, Interval<T>, IntervalSet<T>);

impl<T: Domain> Intersects<Self> for IntervalSet<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.intervals().iter().any(|subset| rhs.intersects(subset))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_intersects() {
        assert!(Interval::open(0, 10).intersects(&Interval::open(5, 15)));

        assert!(!Interval::open(0, 10).intersects(&Interval::closed(10, 20)));
    }

    #[test]
    fn test_set_set_intersects() {
        let a = IntervalSet::new_unchecked(vec![
            Interval::unbound_open(0.0),
            Interval::closed(100.0, 110.0),
            Interval::open(1000.0, 1100.0),
        ]);
        let b = IntervalSet::new_unchecked(vec![
            Interval::open(10.0, 20.0),     // no
            Interval::closed(110.0, 120.0), // [110.0, 110.0]
        ]);

        assert!(a.intersects(&b));
    }
}

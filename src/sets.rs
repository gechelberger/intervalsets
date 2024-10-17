use crate::bound::Bound;
use crate::detail::{BoundCase, Finite, HalfBounded};
use crate::{Domain, Intersects, MaybeEmpty, Merged, Side};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Interval<T: Domain>(pub(crate) BoundCase<T>);

impl<T: Domain> Interval<T> {
    /// Returns a new Empty [`Interval`]
    ///
    /// {} = {x | x not in T }
    pub fn empty() -> Self {
        Self(BoundCase::Finite(Finite::Empty))
    }

    /// Returns a new closed finite [`Interval`] or Empty
    ///
    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        Finite::new(Bound::Closed(left), Bound::Closed(right)).into()
    }

    /// Returns a new open finite [`Interval`] or Empty
    ///
    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        Finite::new(Bound::Open(left), Bound::Open(right)).into()
    }

    /// Returns a new left open finite [`Interval`] or Empty
    ///
    ///  (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        Finite::new(Bound::Open(left), Bound::Closed(right)).into()
    }

    /// Returns a new right open finite [`Interval`] or Empty
    ///
    ///  [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        Finite::new(Bound::Closed(left), Bound::Open(right)).into()
    }

    /// Returns a new open, right-unbound [`Interval`]
    ///
    ///  (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        HalfBounded::new(Side::Left, Bound::Open(left)).into()
    }

    /// Returns a new closed, right-unbound [`Interval`]
    ///
    ///  [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        HalfBounded::new(Side::Left, Bound::Closed(left)).into()
    }

    /// Returns a new open, left-unbound [`Interval`]
    ///
    /// (a, ->) = { x in T | a < x }
    pub fn unbound_open(right: T) -> Self {
        HalfBounded::new(Side::Right, Bound::Open(right)).into()
    }

    /// Returns a new closed, left-unbound [`Interval`]
    ///
    ///  [a, ->) = { x in T | a <= x }
    pub fn unbound_closed(right: T) -> Self {
        HalfBounded::new(Side::Right, Bound::Closed(right)).into()
    }

    /// Returns a new unbounded [`Interval`]
    ///
    /// (<-, ->) = { x in T }
    pub fn unbounded() -> Self {
        BoundCase::Unbounded.into()
    }

    pub fn new_finite(left: Bound<T>, right: Bound<T>) -> Self {
        Finite::new(left, right).into()
    }

    pub fn new_half_bounded(side: Side, bound: Bound<T>) -> Self {
        HalfBounded::new(side, bound).into()
    }

    pub fn is_fully_bounded(&self) -> bool {
        match &self.0 {
            BoundCase::Finite(inner) => matches!(inner, Finite::FullyBounded(_, _)),
            _ => false,
        }
    }

    pub fn is_half_bounded(&self) -> bool {
        matches!(&self.0, BoundCase::Half(_))
    }

    pub fn is_unbounded(&self) -> bool {
        matches!(&self.0, BoundCase::Unbounded)
    }
}

#[derive(Debug, Clone, PartialEq)] // PartialOrd
pub struct IntervalSet<T: Domain> {
    intervals: Vec<Interval<T>>,
}

impl<T: Domain> IntervalSet<T> {
    
    /// Create a new empty IntervalSet
    pub fn empty() -> Self {
        Self { intervals: vec![] }
    }

    /// Create a new Set of intervals and enforce invariants.
    pub fn new(intervals: Vec<Interval<T>>) -> Self {
        
        // O(n)
        if Self::satisfies_invariants(&intervals) {
            return Self::new_unchecked(intervals);
        }

        let mut intervals: Vec<Interval<T>> =
            intervals.into_iter().filter(|iv| !iv.is_empty()).collect();

        if intervals.is_empty() {
            return Self::new_unchecked(intervals);
        }

        // most of the time intervals should already by sorted
        // O(n)
        //if !intervals.is_sorted() {
        // O(n*log(n))
        intervals.sort_by(|a, b| {
                a.partial_cmp(b)
                    .expect("Could not sort intervals in IntervalSet because partial_cmp returned None. Likely float NaN")
            });
        //}

        Self {
            intervals: Self::merge_sorted(intervals),
        }
    }

    /// Merge overlapping intervals assuming that they are already sorted
    pub(crate) fn merge_sorted(intervals: Vec<Interval<T>>) -> Vec<Interval<T>> {
        let mut merged_sets: Vec<Interval<T>> = Vec::with_capacity(intervals.len());
        let mut it = intervals.into_iter();

        // empty already checked so there is at least one subset.
        let mut current = it.next().unwrap();
        for rhs in it {
            match current.merged(&rhs) {
                Some(merged) => {
                    current = merged;
                }
                None => {
                    merged_sets.push(current);
                    current = rhs;
                }
            }
        }
        merged_sets.push(current);
        merged_sets
    }

    pub fn satisfies_invariants(intervals: &Vec<Interval<T>>) -> bool {
        let mut current = &Interval::empty();
        for interval in intervals {
            if interval.is_empty() || current > interval || current.intersects(interval) {
                // current starts as empty which always compares false and intersects false
                // so we should only reach this branch on the first element if it is empty.
                return false;
            }

            current = interval;
        }

        true
    }

    pub fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
    }

    pub fn intervals(&self) -> &Vec<Interval<T>> {
        &self.intervals
    }
}

impl<T: Domain> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        if value.is_empty() {
            return IntervalSet::new_unchecked(vec![]);
        }
        IntervalSet::new_unchecked(vec![value])
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_normalization() {
        let interval = Interval::open(0, 10);
        assert_eq!(interval, Interval::closed(1, 9));
    }


    fn assert_lt<T: Domain>(itv1: Interval<T>, itv2: Interval<T>) {
        assert!(itv1 < itv2);
        assert!(!(itv1 >= itv2)); // antisymmetry

        assert!(itv2 > itv1); // duality
        assert!(!(itv2 <= itv1)); // antisymmetry
    }

    #[test]
    fn test_interval_cmp() {
        // (0, _) < (200, _)
        assert_lt(Interval::open(0.0, 100.0), Interval::open(200.0, 300.0));

        // [0, A] < (0.0, A)
        assert_lt(Interval::closed(0.0, 100.0), Interval::open(0.0, 100.0));

        // [0, 50] < [0, 100]
        assert_lt(Interval::closed(0.0, 50.0), Interval::closed(0.0, 100.0));

        // (0, 50) < (0, ->)
        assert_lt(Interval::open(0.0, 50.0), Interval::open_unbound(0.0));

        // (<-, _) < (0.0, _)
        assert_lt(Interval::unbound_open(5.0), Interval::open(0.0, 3.0));

        // (0, 50) < (<-, ->)
        assert_lt(Interval::unbound_open(50.0), Interval::unbounded());

        // (<-, ->) < (0, 50)
        assert_lt(Interval::unbounded(), Interval::open(0.0, 50.0));

        // (<-, ->) < (0, ->)
        assert_lt(Interval::unbounded(), Interval::open_unbound(0.0));

        // Empty Set should not compare
        assert_eq!(Interval::<u8>::empty() <= Interval::<u8>::unbounded(), false);
        assert_eq!(Interval::<u8>::empty() >= Interval::<u8>::unbounded(), false);
    }
}
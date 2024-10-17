use crate::bound::Bound;
use crate::detail::{BoundCase, Finite, HalfBounded};
use crate::{Bounding, Domain, Intersects, MaybeEmpty, Merged, Side};

/// TODODOODODODODODO
///
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Interval<T: Domain>(pub(crate) BoundCase<T>);

impl<T: Domain> Interval<T> {
    /// Returns a new Empty [`Interval`]
    ///
    /// {} = {x | x not in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Contains};
    ///
    /// let x = Interval::<i32>::empty();
    /// assert_eq!(x.contains(&10), false);
    /// ```
    pub fn empty() -> Self {
        Self(BoundCase::Finite(Finite::Empty))
    }

    /// Returns a new closed finite [`Interval`] or Empty
    ///
    /// [a, b] = { x in T | a <= x <= b }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Contains};
    ///
    /// let x = Interval::closed(10, 20);
    /// assert_eq!(x.contains(&10), true);
    /// assert_eq!(x.contains(&15), true);
    /// assert_eq!(x.contains(&20), true);
    /// assert_eq!(x.contains(&0), false);
    /// ```
    pub fn closed(left: T, right: T) -> Self {
        Finite::new(Bound::Closed(left), Bound::Closed(right)).into()
    }

    /// Returns a new open finite [`Interval`] or Empty
    ///
    /// For discrete data types T, open bounds are **normalized** to closed form.
    /// Continuous(ish) types (like f32, or chrono::DateTime) are left as is.
    ///
    /// (a, b) = { x in T | a < x < b }
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Contains};
    ///
    /// let x = Interval::open(0.0, 10.0);
    /// assert_eq!(x.contains(&0.0), false);
    /// assert_eq!(x.contains(&5.0), true);
    ///
    /// let y = Interval::open(0, 10);
    /// assert_eq!(y.contains(&0), false);
    /// assert_eq!(y.contains(&5), true);
    /// assert_eq!(y, Interval::closed(1, 9));
    /// ```
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

    /// Returns a new unbounded [`Interval`].
    ///
    /// An unbounded interval contains every element in T,
    /// as well as every set of T except the `Empty` set.
    ///
    /// (<-, ->) = { x in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Contains};
    ///
    /// let x = Interval::<f32>::unbounded();
    /// assert_eq!(x.contains(&10.0), true);
    /// assert_eq!(x.contains(&Interval::empty()), false);
    /// ```
    pub fn unbounded() -> Self {
        BoundCase::Unbounded.into()
    }

    /// Returns a new finite [`Interval`].
    ///
    /// If there are no elements that satisfy both left and right bounds
    /// then an `Empty` interval is returned. Otherwise the result will
    /// be fully bounded.
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Bound, Interval, Bounding};
    ///
    /// let x = Interval::open(0, 100);
    /// let y = Interval::new_finite(x.right().unwrap().flip(), Bound::Closed(200));
    /// assert_eq!(y, Interval::closed(100, 200));
    ///
    /// let x = Interval::open(10, 10);
    /// assert_eq!(x, Interval::empty());
    /// ```
    pub fn new_finite(left: Bound<T>, right: Bound<T>) -> Self {
        Finite::new(left, right).into()
    }

    /// Returns a ew half bounded [`Interval`].
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Bound, Complement, Bounding, Side};
    ///
    /// let x = Interval::unbound_open(0);
    /// let y = Interval::new_half_bounded(Side::Left, x.right().unwrap().flip());
    /// assert_eq!(x.complement(), y.into());
    /// ```
    pub fn new_half_bounded(side: Side, bound: Bound<T>) -> Self {
        HalfBounded::new(side, bound).into()
    }

    /// Returns `true` if the interval is either fully bounded or empty.
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval};
    ///
    /// assert_eq!(Interval::<i32>::empty().is_finite(), true);
    /// assert_eq!(Interval::closed(0, 10).is_finite(), true);
    ///
    /// assert_eq!(Interval::unbound_open(10).is_finite(), false);
    /// assert_eq!(Interval::<i32>::unbounded().is_finite(), false);
    /// ```
    pub fn is_finite(&self) -> bool {
        matches!(self.0, BoundCase::Finite(_))
    }

    /// Returns `true` if the interval approaches infinity on either side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval};
    ///
    /// assert_eq!(Interval::<i32>::empty().is_infinite(), false);
    /// assert_eq!(Interval::<i32>::closed(0, 10).is_infinite(), false);
    ///
    /// assert_eq!(Interval::unbound_open(10).is_infinite(), true);
    /// assert_eq!(Interval::<i32>::unbounded().is_infinite(), true);
    /// ```
    pub fn is_infinite(&self) -> bool {
        !self.is_finite()
    }

    /// Return `true` if the interval is finite **and** not empty.
    ///
    /// # Example
    /// ```
    /// use intervalsets::Interval;
    ///
    /// assert_eq!(Interval::closed(0, 10).is_fully_bounded(), true);
    ///
    /// assert_eq!(Interval::<i32>::empty().is_fully_bounded(), false);
    /// assert_eq!(Interval::<i32>::unbounded().is_fully_bounded(), false);
    /// ```
    pub fn is_fully_bounded(&self) -> bool {
        match &self.0 {
            BoundCase::Finite(inner) => matches!(inner, Finite::FullyBounded(_, _)),
            _ => false,
        }
    }

    /// Return `true` if the interval is unbounded on exactly one side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::Interval;
    ///
    /// assert_eq!(Interval::closed_unbound(10).is_half_bounded(), true);
    /// assert_eq!(Interval::<i32>::unbounded().is_half_bounded(), false);
    ///
    /// ```
    pub fn is_half_bounded(&self) -> bool {
        matches!(&self.0, BoundCase::Half(_))
    }

    /// Returns `true` if the interval is unbounded on the expected side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Side};
    ///
    /// let x = Interval::unbound_open(10);
    /// assert_eq!(x.is_half_bounded_on(Side::Right), true);
    /// assert_eq!(x.is_half_bounded_on(Side::Left), false);
    ///
    /// let x = Interval::closed_unbound(10);
    /// assert_eq!(x.is_half_bounded_on(Side::Right), false);
    /// assert_eq!(x.is_half_bounded_on(Side::Left), true);
    /// ```
    pub fn is_half_bounded_on(&self, side: Side) -> bool {
        match &self.0 {
            BoundCase::Half(inner) => inner.bound(side).is_some(),
            _ => false,
        }
    }

    /// Returns `true` if the interval is unbounded on both sides.
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Merged};
    ///
    /// let x = Interval::unbound_closed(10)
    ///             .merged(&Interval::closed_unbound(-10))
    ///             .unwrap();
    ///
    /// assert_eq!(x.is_unbounded(), true);
    /// ```
    pub fn is_unbounded(&self) -> bool {
        matches!(&self.0, BoundCase::Unbounded)
    }
}


/// A Set in Z or R consisting of disjoint contiguous intervals.
///
/// # Invariants
///
/// * All stored intervals are normalized.
///     * We do not enforce this here because it should be
///       an invariant of Interval<T> already.
/// * No stored interval may be the empty set.
///     * Emptiness is represented by storing no intervals.
///     * Normalized Interval<T> should have a total ordering w/o empty sets.
/// * All intervals are stored in ascending order.
/// * All stored intervals are disjoint subsets of T.
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

    pub fn accum_map<F>(&self, func: F) -> Self
    where
        F: Fn(&mut Vec<Interval<T>>, &Interval<T>),
    {
        let mut accum: Vec<Interval<T>> = Vec::with_capacity(self.intervals.len());
        for subset in self.intervals.iter() {
            func(&mut accum, subset);
        }

        Self::new(accum)
    }

    pub fn map<F>(&self, func: F) -> Self
    where
        F: Fn(&Interval<T>) -> Self,
    {
        self.accum_map(|accum, interval| {
            let mut result = func(interval);
            accum.append(&mut result.intervals)
        })
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
        assert_eq!(
            Interval::<u8>::empty() <= Interval::<u8>::unbounded(),
            false
        );
        assert_eq!(
            Interval::<u8>::empty() >= Interval::<u8>::unbounded(),
            false
        );
    }
}

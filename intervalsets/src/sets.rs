use crate::bound::Bound;
use crate::detail::{BoundCase, Finite};
use crate::numeric::Domain;
use crate::ops::{Intersects, Merged};
use crate::{Bounding, MaybeEmpty, Side};

use crate::factory::Factory;

/// A Set representation of a contiguous interval on N, Z, or R.
///
/// Discrete types (integers) are normalized to closed form.
///
/// Most operations are supported through
/// [trait implementations](#trait-implementations).
///
/// Creation is managed through the [`Factory`](crate::factory::Factory)
/// trait.
///
/// ```
/// use intervalsets::prelude::*;
/// let x = Interval::closed(0, 10);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Interval<T: Domain>(pub(crate) BoundCase<T>);

impl<T: Domain> Interval<T> {
    /// Returns `true` if the interval is either fully bounded or empty.
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Factory};
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
    /// use intervalsets::{Interval, Factory};
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
    /// use intervalsets::{Interval, Factory};
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
    /// use intervalsets::{Interval, Factory};
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
    /// use intervalsets::{Interval, Side, Factory};
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
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::Merged;
    ///
    /// let x = Interval::unbound_closed(10)
    ///             .merged(Interval::closed_unbound(-10))
    ///             .unwrap();
    ///
    /// assert_eq!(x.is_unbounded(), true);
    /// ```
    pub fn is_unbounded(&self) -> bool {
        matches!(&self.0, BoundCase::Unbounded)
    }

    /// Map the bounds of this interval to a new one or else `Empty`.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    /// use intervalsets::{Bound, Side};
    /// use intervalsets::numeric::Domain;
    ///
    /// fn shift<T: Domain>(interval: Interval<T>, amount: T) -> Interval<T>
    /// where
    ///     T: Domain + core::ops::Add<T, Output=T>
    /// {
    ///     let shift_bound = |bound: &Bound<T>| {
    ///         bound.clone().map(|v| v + amount.clone())
    ///     };
    ///
    ///     interval.flat_map(|left, right| {
    ///         match (left, right) {
    ///             (None, None) => Interval::unbounded(),
    ///             (None, Some(right)) => {
    ///                 Interval::half_bounded(Side::Right, shift_bound(right))
    ///             },
    ///             (Some(left), None) => {
    ///                 Interval::half_bounded(Side::Left, shift_bound(left))
    ///             },
    ///             (Some(left), Some(right)) => {
    ///                 Interval::finite(shift_bound(left), shift_bound(right))
    ///             }
    ///         }
    ///     })
    /// }
    ///
    /// assert_eq!(shift(Interval::empty(), 10), Interval::empty());
    /// assert_eq!(shift(Interval::closed(0, 10), 10), Interval::closed(10, 20));
    /// assert_eq!(shift(Interval::unbound_closed(0), 10), Interval::unbound_closed(10));
    /// assert_eq!(shift(Interval::closed_unbound(0), 10), Interval::closed_unbound(10));
    /// ```
    pub fn flat_map<F>(&self, func: F) -> Self
    where
        F: FnOnce(Option<&Bound<T>>, Option<&Bound<T>>) -> Self,
    {
        self.map_or_else(Self::empty, func)
    }

    /// Map the bounds of this interval or return default if `Empty`.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// fn is_finite(interval: Interval<i32>) -> bool {
    ///     interval.map_or(true, |left, right| {
    ///         matches!((left, right), (Some(_), Some(_)))
    ///     })
    /// }
    ///
    /// assert_eq!(is_finite(Interval::empty()), true);
    /// assert_eq!(is_finite(Interval::closed(0, 10)), true);
    /// assert_eq!(is_finite(Interval::unbound_closed(0)), false);
    /// assert_eq!(is_finite(Interval::closed_unbound(0)), false);
    /// assert_eq!(is_finite(Interval::unbounded()), false);
    /// ```
    pub fn map_or<F, U>(&self, default: U, func: F) -> U
    where
        F: FnOnce(Option<&Bound<T>>, Option<&Bound<T>>) -> U,
    {
        if self.is_empty() {
            return default;
        }
        func(self.left(), self.right())
    }

    /// Map the bounds of this interval or result of default fn if `Empty`.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    /// use intervalsets::Side;
    ///
    /// fn my_complement(interval: &Interval<i32>) -> Interval<i32> {
    ///     interval.map_or_else(Interval::unbounded, |left, right| {
    ///         match (left, right) {
    ///             (None, None) => Interval::empty(),
    ///             (None, Some(right)) => Interval::half_bounded(Side::Left, right.clone().flip()),
    ///             (Some(left), None) => Interval::half_bounded(Side::Right, left.clone().flip()),
    ///             (Some(left), Some(right)) => panic!("... elided ...")
    ///         }
    ///     })
    /// }
    ///
    /// let x = Interval::closed_unbound(0);
    /// assert_eq!(my_complement(&x), x.complement().expect_interval());
    ///
    /// let x = Interval::unbound_closed(0);
    /// assert_eq!(my_complement(&x), x.complement().expect_interval());
    ///
    /// let x = Interval::empty();
    /// assert_eq!(my_complement(&x), x.complement().expect_interval());
    ///
    /// let x = Interval::unbounded();
    /// assert_eq!(my_complement(&x), x.complement().expect_interval());
    /// ```
    pub fn map_or_else<F, D, U>(&self, default: D, func: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(Option<&Bound<T>>, Option<&Bound<T>>) -> U,
    {
        if self.is_empty() {
            default()
        } else {
            func(self.left(), self.right())
        }
    }

    /// Map bounds to a new Interval if and only if fully bounded else `Empty`.
    ///
    /// # Panics
    ///
    /// This method panics if called on an unbounded interval.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// fn shift(interval: Interval<i32>, amount: i32) -> Interval<i32> {
    ///     interval.flat_map_finite(|left, right| {
    ///         Interval::finite(
    ///             left.clone().map(|v| v + amount), right.clone().map(|v| v + amount)
    ///         )
    ///     })
    /// }
    /// assert_eq!(shift(Interval::empty(), 10), Interval::empty());
    /// assert_eq!(shift(Interval::closed(0, 10), 10), Interval::closed(10, 20));
    /// ```
    /// ```should_panic
    /// use intervalsets::prelude::*;
    ///
    /// fn shift(interval: Interval<i32>, amount: i32) -> Interval<i32> {
    ///     interval.flat_map_finite(|left, right| Interval::empty())
    /// }
    ///
    /// // any of these should panic:
    /// assert_eq!(shift(Interval::unbound_closed(0), 10), Interval::empty());
    /// assert_eq!(shift(Interval::closed_unbound(0), 10), Interval::empty());
    /// assert_eq!(shift(Interval::unbounded(), 10), Interval::empty());
    /// ```
    pub fn flat_map_finite<F>(&self, func: F) -> Self
    where
        F: FnOnce(&Bound<T>, &Bound<T>) -> Self,
    {
        match &self.0 {
            BoundCase::Finite(inner) => inner.ref_map_or_else(Self::empty, func),
            _ => panic!("Expected finite interval"),
        }
    }
}

impl<T: Domain + Eq> Eq for Interval<T> {}

impl<T: Domain + PartialOrd> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Domain + Ord> Ord for Interval<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

//impl<T: Domain + Copy> Copy for Interval<T> {}

/// A Set in N, Z, or R consisting of disjoint contiguous intervals.
///
/// # Invariants
///
/// * All stored intervals are normalized.
///     * We do not enforce this here because it should be
///       an invariant of `Interval<T>` already.
/// * No stored interval may be the empty set.
///     * Emptiness is represented by storing no intervals.
///     * Normalized `Interval<T>` should have a total ordering w/o empty sets.
/// * All intervals are stored in ascending order.
/// * All stored intervals are disjoint subsets of T.
///     * Stored intervals *should* not be adjacent.
///         * This can only be assured for `T: Eq + Ord`
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalSet<T: Domain> {
    intervals: Vec<Interval<T>>,
}

impl<T: Domain> IntervalSet<T> {
    /// Create a new empty IntervalSet
    pub fn empty() -> Self {
        Self { intervals: vec![] }
    }

    /// Create a new Set of intervals and enforce invariants.
    pub fn new<I>(intervals: I) -> Self
    where
        I: IntoIterator<Item = Interval<T>>,
    {
        let mut intervals = intervals.into_iter().filter(|iv| !iv.is_empty()).collect();

        if Self::satisfies_invariants(&intervals) {
            // includes empty case
            return Self::new_unchecked(intervals);
        }

        // most of the time intervals should already by sorted
        intervals.sort_by(|a, b| {
            a.partial_cmp(b)
                .expect("Could not sort intervals in IntervalSet because partial_cmp returned None. Likely float NaN")
        });

        Self::merge_sorted(intervals)
    }

    /// Merge overlapping intervals assuming all other invariants are
    /// already satisfied.
    pub(crate) fn merge_sorted<I>(intervals: I) -> Self
    where
        I: IntoIterator<Item = Interval<T>>,
    {
        let mut it = intervals.into_iter();
        let mut current = match it.next() {
            None => return Self::empty(),
            Some(head) => head,
        };

        let mut merged_sets: Vec<Interval<T>> = Vec::new();
        for rhs in it {
            match current.clone().merged(rhs.clone()) {
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
        Self {
            intervals: merged_sets,
        }
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

    /// Creates a new IntervalSet without checking invariants.
    ///
    /// The invariants check and enforcement step can be expensive, O(nlog(n)),
    /// since it sorts all elements. If an operation is certain
    /// that it has left the invariants in tact it can create a new IntervalSet
    /// directly.
    ///
    /// Behavior is **undefined** if invariants are violated!
    pub fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
    }

    /// Creates an [`Interval`] that forms a convex hull for this Set.
    ///
    /// This should be equivalent to using [`ConvexHull`](crate::ConvexHull),
    /// but much more efficient and convenient.
    ///
    /// > This function call relies on invariants.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let set = IntervalSet::from_iter([
    ///     Interval::closed(100, 110),
    ///     Interval::closed(0, 10),
    /// ]);
    /// assert_eq!(set.convex_hull(), Interval::closed(0, 110));
    ///
    /// // ConvexHull trait equivalent
    /// assert_eq!(Interval::convex_hull([set]), Interval::closed(0, 110));
    /// ```
    ///
    pub fn convex_hull(&self) -> Interval<T> {
        if self.is_empty() {
            return Interval::<T>::empty();
        }

        let first = self.intervals.first().unwrap();
        let last = self.intervals.last().unwrap();

        Interval::finite(first.left().unwrap().clone(), last.right().unwrap().clone())
    }

    /// Take the only [`Interval`] in this Set. `self` is consumed.
    ///
    /// This is useful for operations that *could* return
    /// multiple intervals such as [`Union`](crate::ops::Union).
    ///
    /// # Panics
    ///
    /// This method panics if there is not **exactly** one subset.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let interval = Interval::closed(0, 10);
    /// let iset = IntervalSet::from(interval.clone());
    /// let unwrapped = iset.expect_interval(); // iset moved
    /// assert_eq!(interval, unwrapped);
    ///
    /// let a = Interval::closed(0, 10)
    ///     .union(Interval::closed(5, 15))
    ///     .expect_interval();
    /// assert_eq!(a, Interval::closed(0, 15));
    ///
    /// let a = IntervalSet::from_iter::<[(i32, i32); 0]>([]);
    /// assert_eq!(a.expect_interval(), Interval::<i32>::empty());
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets::prelude::*;
    ///
    /// let a = Interval::closed(0, 10)
    ///     .union(Interval::closed(100, 110))
    ///     .expect_interval();
    /// ```
    pub fn expect_interval(mut self) -> Interval<T> {
        match self.intervals.len() {
            0 => Interval::<T>::empty(),
            1 => self.intervals.remove(0),
            _ => panic!("Set should have exactly one subset."), //panic!("{} should have exactly one subset.", self);
        }
    }

    /// Returns a slice of the [`Interval`].
    pub fn slice(&self) -> &[Interval<T>] {
        &self.intervals
    }

    /// Returns a new iterator over the subsets in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = &Interval<T>> {
        self.intervals.iter()
    }

    /// Returns the underlying vector of intervals; `self` is consumed.
    ///
    /// # Examples
    ///
    /// ```compile_fail
    /// let set = IntervalSet::from((0, 10));
    /// let intervals = set.into_raw();
    /// let q = set.contains(5) // set is moved
    /// ```
    pub fn into_raw(self) -> Vec<Interval<T>> {
        self.intervals
    }

    /// Returns a new IntervalSet mapped from this Set's subsets.
    ///
    /// The mapping function is given a mutable vector in which to
    /// collect as many or as few new intervals as desired regardless
    /// of the intermediate types in question.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let x = Interval::closed(0, 10)
    ///     .union(Interval::closed(20, 40))
    ///     .union(Interval::closed(50, 100));
    ///
    /// let mapped = x.collect_map(|mut collect, subset| {
    ///     if subset.count().finite() > 30 {
    ///         collect.push(subset.clone())
    ///     }
    /// });
    ///
    /// assert_eq!(mapped, IntervalSet::from(Interval::closed(50, 100)));
    ///
    /// let mask = Interval::closed(5, 25);
    /// let mapped = x.collect_map(|mut collect, subset| {
    ///     collect.push(mask.ref_intersection(subset));
    /// });
    /// assert_eq!(mapped, IntervalSet::from_iter([
    ///     Interval::closed(5, 10),
    ///     Interval::closed(20, 25)
    /// ]));
    ///
    /// let mask_set = IntervalSet::from_iter([
    ///     Interval::closed(20, 30),
    ///     Interval::closed(50, 60),
    /// ]);
    /// let mapped = x.collect_map(|mut collect, subset| {
    ///     for interval in subset.ref_difference(&mask_set) {
    ///         collect.push(interval)
    ///     }
    /// });
    /// assert_eq!(mapped, IntervalSet::from_iter([
    ///     Interval::closed(0, 10),
    ///     Interval::closed(31, 40),
    ///     Interval::closed(61, 100),
    /// ]));
    /// ```
    pub fn collect_map<F>(&self, func: F) -> Self
    where
        F: Fn(&mut Vec<Interval<T>>, &Interval<T>),
    {
        let mut accum: Vec<Interval<T>> = Vec::new();
        for subset in self.intervals.iter() {
            func(&mut accum, subset);
        }

        Self::new(accum)
    }

    /// Returns a new IntervalSet mapped from this Set`s subsets.
    ///
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let x = Interval::closed(0, 10)
    ///     .union(Interval::closed(20, 40))
    ///     .union(Interval::closed(50, 100));
    ///
    /// let mapped = x.flat_map(|subset| {
    ///     if subset.count().finite() > 30 {
    ///         subset.clone().into()
    ///     } else {
    ///         Interval::empty().into()
    ///     }
    /// });
    ///
    /// assert_eq!(mapped, IntervalSet::from(Interval::closed(50, 100)));
    /// ```
    pub fn flat_map<F>(&self, func: F) -> Self
    where
        F: Fn(&Interval<T>) -> Self,
    {
        self.collect_map(|accum, interval| {
            let mut result = func(interval);
            accum.append(&mut result.intervals)
        })
    }
}

impl<T, I> FromIterator<I> for IntervalSet<T>
where
    T: Domain,
    I: Into<Interval<T>>,
{
    fn from_iter<U: IntoIterator<Item = I>>(iter: U) -> Self {
        Self::new(iter.into_iter().map(|x| x.into()))
    }
}

impl<T: Domain> IntoIterator for IntervalSet<T> {
    type Item = Interval<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter()
    }
}

impl<T: Domain + Eq> Eq for IntervalSet<T> {}

impl<T: Domain + PartialOrd> PartialOrd for IntervalSet<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.intervals.partial_cmp(&other.intervals)
    }
}

impl<T: Domain + Ord> Ord for IntervalSet<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.intervals.cmp(&other.intervals)
    }
}

#[cfg(test)]
mod tests {
    //use core::hash::Hash;

    use super::*;
    use crate::ops::{Complement, Difference};

    #[test]
    fn test_interval_normalization() {
        let interval = Interval::open(0, 10);
        assert_eq!(interval, Interval::closed(1, 9));
    }

    #[test]
    fn test_interval_set_fold() {
        let x = IntervalSet::from_iter([Interval::closed(0, 10), Interval::closed(100, 110)]);

        assert_eq!(
            x.iter().fold(
                IntervalSet::from(Interval::unbounded()),
                |left: IntervalSet<_>, item: &Interval<_>| { left.difference(item.clone()) }
            ),
            x.complement()
        );
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

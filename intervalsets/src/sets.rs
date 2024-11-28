use intervalsets_core::ops::MergeSortedByValue;
use intervalsets_core::sets::EnumInterval;

use crate::bound::ord::{OrdBoundPair, OrdBounded};
use crate::bound::{FiniteBound, SetBounds, Side};
use crate::numeric::Element;
use crate::ops::Connects;
use crate::MaybeEmpty;

/// A Set representation of a contiguous interval in N, Z, or R.
///
/// Discrete types (integers) are normalized to closed form.
///
/// Most operations are supported through
/// [trait implementations](#trait-implementations).
///
/// See [`factory`](crate::factory) for helpers.
///
/// ```
/// use intervalsets::prelude::*;
/// let x = Interval::closed(0, 10);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Interval<T>(pub(crate) EnumInterval<T>);

impl<T> Interval<T> {
    pub const fn empty() -> Self {
        Self(EnumInterval::empty())
    }

    /// Returns `true` if the interval is either fully bounded or empty.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// assert_eq!(Interval::<i32>::empty().is_finite(), true);
    /// assert_eq!(Interval::closed(0, 10).is_finite(), true);
    ///
    /// assert_eq!(Interval::unbound_open(10).is_finite(), false);
    /// assert_eq!(Interval::<i32>::unbounded().is_finite(), false);
    /// ```
    pub fn is_finite(&self) -> bool {
        matches!(self.0, EnumInterval::Finite(_))
    }

    /// Returns `true` if the interval approaches infinity on either side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
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
    /// use intervalsets::prelude::*;
    ///
    /// assert_eq!(Interval::closed(0, 10).is_fully_bounded(), true);
    ///
    /// assert_eq!(Interval::<i32>::empty().is_fully_bounded(), false);
    /// assert_eq!(Interval::<i32>::unbounded().is_fully_bounded(), false);
    /// ```
    pub fn is_fully_bounded(&self) -> bool {
        self.0.is_fully_bounded()
    }

    /// Return `true` if the interval is unbounded on exactly one side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// assert_eq!(Interval::closed_unbound(10).is_half_bounded(), true);
    /// assert_eq!(Interval::<i32>::unbounded().is_half_bounded(), false);
    ///
    /// ```
    pub fn is_half_bounded(&self) -> bool {
        matches!(&self.0, EnumInterval::Half(_))
    }

    /// Returns `true` if the interval is unbounded on the expected side.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
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
        match self.0 {
            EnumInterval::Half(ref inner) => inner.side() == side,
            _ => false,
        }
    }

    /// Returns `true` if the interval is unbounded on both sides.
    ///
    /// # Example
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let x = Interval::unbound_closed(10)
    ///             .try_merge(Interval::closed_unbound(-10))
    ///             .unwrap();
    ///
    /// assert_eq!(x.is_unbounded(), true);
    /// ```
    pub fn is_unbounded(&self) -> bool {
        matches!(self.0, EnumInterval::Unbounded)
    }
}

impl<T: Element> Default for Interval<T> {
    fn default() -> Self {
        Interval::empty()
    }
}

impl<T> SetBounds<T> for Interval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        self.0.bound(side)
    }
}

impl<T> MaybeEmpty for Interval<T> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> OrdBounded<T> for Interval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        OrdBoundPair::from(self)
    }
}

/// A subset of Z, or R consisting of unconnected intervals.
///
/// # Invariants
///
/// * All stored intervals are normalized.
///     * This is not checked here because it should be
///       an invariant of `Interval<T>` already.
/// * No stored interval may be the empty set.
///     * Emptiness is represented by storing no intervals.
/// * All intervals are stored in ascending order.
/// * All stored intervals are unconnected subsets of T.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct IntervalSet<T> {
    intervals: Vec<Interval<T>>,
}

impl<T: Element> IntervalSet<T> {
    /// Create a new Set of intervals and enforce invariants.
    pub fn new<I>(intervals: I) -> Self
    where
        I: IntoIterator<Item = Interval<T>>,
    {
        let mut intervals: Vec<_> = intervals.into_iter().filter(|iv| !iv.is_empty()).collect();

        if Self::satisfies_invariants(&intervals) {
            return Self { intervals };
        }

        intervals.sort_unstable_by(|a, b| {
            a.partial_cmp(b)
                .expect("Could not sort intervals in IntervalSet because partial_cmp returned None. Likely float NaN")
        });

        let intervals: Vec<_> = MergeSortedByValue::new(intervals).collect();
        Self { intervals }
    }
}

impl<T> IntervalSet<T> {
    /// Create a new empty IntervalSet
    pub fn empty() -> Self {
        Self { intervals: vec![] }
    }
}

impl<T: Element> IntervalSet<T> {
    pub fn satisfies_invariants(intervals: &[Interval<T>]) -> bool {
        let mut prev = &Interval::<T>::empty();
        for interval in intervals {
            if prev >= interval || (prev.is_inhabited() && prev.connects(interval)) {
                return false;
            }

            prev = interval;
        }
        true
    }
}

impl<T> IntervalSet<T> {
    /// Creates a new IntervalSet without checking invariants.
    ///
    /// # Safety
    ///
    /// User is responsible for enforcing invariants:
    /// 1. provided intervals do not include the empty set.
    /// 2. provided intervals are sorted ascendingly.
    /// 3. provided intervals are not connected to each other.
    pub unsafe fn new_unchecked<I>(intervals: I) -> Self
    where
        I: IntoIterator<Item = Interval<T>>,
    {
        Self {
            intervals: Vec::from_iter(intervals),
        }
    }
}

impl<T: Clone + Element> IntervalSet<T> {
    /// Creates an [`Interval`] that forms a convex hull for this Set.
    ///
    /// This should be equivalent to using [`ConvexHull`](crate::ops::ConvexHull),
    /// but more efficient and convenient.
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::prelude::*;
    ///
    /// let set = IntervalSet::new([
    ///     Interval::closed(100, 110),
    ///     Interval::closed(0, 10),
    /// ]);
    /// assert_eq!(set.hull(), Interval::closed(0, 110));
    ///
    /// // ConvexHull trait equivalent
    /// assert_eq!(Interval::hull([set]), Interval::closed(0, 110));
    /// ```
    ///
    pub fn hull(&self) -> Interval<T> {
        match self.intervals.len() {
            0 => Interval::empty(),
            1 => self.intervals.first().unwrap().clone(),
            _ => {
                let first = self.intervals.first().unwrap();
                let last = self.intervals.last().unwrap();
                let (min, _) = first.ord_bound_pair().into_raw();
                let (_, max) = last.ord_bound_pair().into_raw();
                let hull = OrdBoundPair::new(min.cloned(), max.cloned());
                hull.try_into().expect("intervalset invariants violated")
            }
        }
    }
}

impl<T> IntervalSet<T> {
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
}

impl<T, I> FromIterator<I> for IntervalSet<T>
where
    T: Element,
    I: Into<Interval<T>>,
{
    fn from_iter<U: IntoIterator<Item = I>>(iter: U) -> Self {
        Self::new(iter.into_iter().map(|x| x.into()))
    }
}

impl<T: Element> IntoIterator for IntervalSet<T> {
    type Item = Interval<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter()
    }
}

impl<T> OrdBounded<T> for IntervalSet<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        OrdBoundPair::<&T>::from(self)
    }
}

impl<T> MaybeEmpty for IntervalSet<T> {
    fn is_empty(&self) -> bool {
        self.intervals.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use core::hash::{Hash, Hasher};

    use siphasher::sip::SipHasher13;

    use super::*;
    use crate::factory::traits::*;
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

    fn assert_lt<T: Element>(itv1: Interval<T>, itv2: Interval<T>) {
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

        // Empty Set < everything else
        assert_eq!(Interval::<u8>::empty() < Interval::<u8>::unbounded(), true);
        assert_eq!(
            Interval::<u8>::empty() >= Interval::<u8>::unbounded(),
            false
        );
    }

    fn do_hash<T: Hash>(item: T) -> u64 {
        let key: &[u8; 16] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let mut hasher = SipHasher13::new_with_key(key);
        item.hash(&mut hasher);
        hasher.finish()
    }

    pub(super) fn check_hash<T: Hash + PartialEq>(a: &T, b: &T) {
        if a == b {
            assert_eq!(do_hash(a), do_hash(b));
        } else {
            // hash collissions are allowed, but highly unlikely
            assert_ne!(do_hash(a), do_hash(b));
        }
    }

    #[test]
    fn test_hash_stable_interval() {
        check_hash(&Interval::<i8>::empty(), &Interval::<i8>::empty());
        check_hash(&Interval::<i8>::unbounded(), &Interval::<i8>::unbounded());
        check_hash(
            &Interval::<i8>::closed(0, 10),
            &Interval::<i8>::closed(0, 10),
        );

        // f32 & f64 are not Hash
        //check_hash(
        //    &Interval::<f64>::open(0.0, 10.0),
        //    &Interval::<f64>::open(0.0, 10.0),
        //)
    }

    #[test]
    fn test_hash_stable_set() {
        check_hash(&IntervalSet::<i8>::empty(), &Interval::<i8>::empty().into());
    }

    #[quickcheck]
    fn check_hash_interval_set(a: i8, b: i8) {
        let set = IntervalSet::from_iter([Interval::closed(-50, 50)]);

        let other: IntervalSet<_> = Interval::closed(a, b).into();
        check_hash(&set, &other);
    }

    #[quickcheck]
    fn check_hash_stable_interval(a: i8, b: i8) {
        let interval = Interval::closed(-50, 50);
        check_hash(&interval, &Interval::closed(a, b));
    }
}

/*
#[cfg(feature = "rust_decimal")]
#[cfg(test)]
mod decimal_tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::Factory;

    #[quickcheck]
    fn check_hash_decimal_interval(a: f32, b: f32) {
        let a = Decimal::from_f32_retain(a);
        let b = Decimal::from_f32_retain(b);
        if a.is_none() || b.is_none() {
            return;
        }
        let a = a.unwrap();
        let b = b.unwrap();

        let interval = Interval::open(a, b);
        super::tests::check_hash(&interval, &Interval::open(a, b));
        super::tests::check_hash(&interval, &Interval::closed(a, b));
        super::tests::check_hash(&interval, &Interval::open_closed(a, b));
        super::tests::check_hash(&interval, &Interval::closed_open(a, b));
    }
}
*/

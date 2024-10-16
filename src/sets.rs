use crate::ival::{Bound, IVal, Side};
use crate::numeric::Domain;

use crate::empty::MaybeEmpty;
use crate::op::merged::Merged;
use crate::pred::intersects::Intersects;

/// A fully bounded interval in N, Z, or R.
///
/// (a, a) = (a, a] = [a, a) = Empty { x not in T }
/// [a, a] = NonZero { x in T |    x = a    }
/// (a, b) = NonZero { x in T | a <  x <  b }
/// (a, b] = NonZero { x in T | a <  x <= b }
/// [a, b) = NonZero { x in T | a <= x <  b }
/// [a, b] = NonZero { x in T | a <= x <= b }
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FiniteInterval<T> {
    Empty,
    FullyBounded(IVal<T>, IVal<T>),
}

impl<T: Domain> FiniteInterval<T> {
    pub fn new(left: IVal<T>, right: IVal<T>) -> Self {
        if left.value > right.value {
            Self::Empty
        } else if left.value == right.value {
            if left.bound == Bound::Open || right.bound == Bound::Open {
                Self::Empty
            } else {
                // singleton set
                Self::new_unchecked(left, right)
            }
        } else {
            Self::new_unchecked(left.normalized(Side::Left), right.normalized(Side::Right))
        }
    }

    pub fn new_unchecked(left: IVal<T>, right: IVal<T>) -> Self {
        Self::FullyBounded(left, right)
    }

    pub fn singleton(item: T) -> Self {
        Self::new_unchecked(IVal::closed(item.clone()), IVal::closed(item))
    }

    pub fn open(left: T, right: T) -> Self {
        Self::new(IVal::open(left), IVal::open(right))
    }

    pub fn closed(left: T, right: T) -> Self {
        Self::new(IVal::closed(left), IVal::closed(right))
    }

    pub fn open_closed(left: T, right: T) -> Self {
        Self::new(IVal::open(left), IVal::closed(right))
    }

    pub fn closed_open(left: T, right: T) -> Self {
        Self::new(IVal::closed(left), IVal::open(right))
    }
}

impl<T> FiniteInterval<T> {
    pub fn lval_unchecked(&self) -> &T {
        match self {
            Self::Empty => panic!("Empty interval has no left bound"),
            Self::FullyBounded(left, _) => &left.value,
        }
    }

    pub fn rval_unchecked(&self) -> &T {
        match self {
            Self::Empty => panic!("Empty interval has no right bound"),
            Self::FullyBounded(_, right) => &right.value,
        }
    }

    pub fn map_bounds(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> Self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::FullyBounded(left, right) => func(left, right),
        }
    }

    #[allow(dead_code)]
    pub fn map<U>(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> U) -> Option<U> {
        match self {
            Self::Empty => None,
            Self::FullyBounded(left, right) => Some(func(left, right)),
        }
    }

    pub fn map_or<U>(&self, default: U, func: impl Fn(&IVal<T>, &IVal<T>) -> U) -> U {
        match self {
            Self::Empty => default,
            Self::FullyBounded(left, right) => func(left, right),
        }
    }
}

///
/// 
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HalfBounded<T> {
    pub(crate) side: Side,
    pub(crate) ival: IVal<T>,
}

impl<T: Domain> HalfBounded<T> {
    pub fn new(side: Side, ival: IVal<T>) -> Self {
        Self {
            side,
            ival: ival.normalized(side),
        }
    }

    /// (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        Self::new(Side::Right, IVal::open(right))
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        Self::new(Side::Right, IVal::closed(right))
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        Self::new(Side::Left, IVal::open(left))
    }

    /// [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        Self::new(Side::Left, IVal::closed(left))
    }

    pub fn lval_unchecked(&self) -> &T {
        match self.side {
            Side::Left => &self.ival.value,
            Side::Right => panic!("right half interval has no left bound"),
        }
    }

    pub fn rval_unchecked(&self) -> &T {
        match self.side {
            Side::Left => panic!("left half interval has no right bound"),
            Side::Right => &self.ival.value,
        }
    }
}

///
/// 
/// 
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EBounds<T> {
    /// (a, a) = (a, a] = [a, a) = Empty { x not in T }
    /// [a, a] = FullyBounded { x in T |    x = a    }
    /// (a, b) = FullyBounded { x in T | a <  x <  b }
    /// (a, b] = FullyBounded { x in T | a <  x <= b }
    /// [a, b) = FullyBounded { x in T | a <= x <  b }
    /// [a, b] = FullyBounded { x in T | a <= x <= b }
    Finite(FiniteInterval<T>),

    /// (a, ->) = Left  { x in T | a <  x      }
    /// [a, ->) = Left  { x in T | a <= x      }
    /// (<-, b) = Right { x in T |      x < b  }
    /// (<-, b] = Right { x in T |      x <= b }
    Half(HalfBounded<T>),

    /// {<-, ->) = { x in T }
    Unbounded,
}

impl<T: Domain> EBounds<T> {
    /// {} = {x | x not in T }
    pub fn empty() -> Self {
        FiniteInterval::Empty.into()
    }

    /// [a, a] = {x in T | a <= x <= a }
    pub fn singleton(item: T) -> Self {
        FiniteInterval::singleton(item).into()
    }

    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        FiniteInterval::open(left, right).into()
    }

    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        FiniteInterval::closed(left, right).into()
    }

    /// (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        FiniteInterval::open_closed(left, right).into()
    }

    /// [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        FiniteInterval::closed_open(left, right).into()
    }

    // (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        HalfBounded::unbound_open(right).into()
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        HalfBounded::unbound_closed(right).into()
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        HalfBounded::open_unbound(left).into()
    }

    /// [a, ->) = { x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        HalfBounded::closed_unbound(left).into()
    }

    /// (<-, ->) = { x in T }
    pub fn unbound() -> Self {
        Self::Unbounded
    }

    pub fn lval_unchecked(&self) -> &T {
        match self {
            Self::Finite(interval) => interval.lval_unchecked(),
            Self::Half(interval) => interval.lval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }

    pub fn rval_unchecked(&self) -> &T {
        match self {
            Self::Finite(interval) => interval.rval_unchecked(),
            Self::Half(interval) => interval.rval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Interval<T: Domain> (pub(crate) EBounds<T>);

impl<T: Domain> Interval<T> {

    pub fn empty() -> Self {
        FiniteInterval::Empty.into()
    }

    pub fn closed(left: T, right: T) -> Self {
        FiniteInterval::closed(left, right).into()
    }

    pub fn open(left: T, right: T) -> Self {
        FiniteInterval::open(left, right).into()
    }

    pub fn open_closed(left: T, right: T) -> Self {
        FiniteInterval::open_closed(left, right).into()
    }

    pub fn closed_open(left: T, right: T) -> Self {
        FiniteInterval::closed_open(left, right).into()
    }

    pub fn unbound_open(right: T) -> Self {
        HalfBounded::unbound_open(right).into()
    }

    pub fn unbound_closed(right: T) -> Self {
        HalfBounded::unbound_closed(right).into()
    }

    pub fn open_unbound(left: T) -> Self {
        HalfBounded::open_unbound(left).into()
    }

    pub fn closed_unbound(left: T) -> Self {
        HalfBounded::closed_unbound(left).into()
    }

    pub fn unbounded() -> Self {
        EBounds::unbound().into()
    }
}



#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct IntervalSet<T: Domain> {
    pub(crate) intervals: Vec<Interval<T>>,
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
#[allow(dead_code)]
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

    /// Create a new IntervalSet directly from a Vec<Interval<_>>.
    ///
    /// If invariants are not maintained, behavior is undefined.
    pub fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
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

            //TODO: check is_normalized()

            current = interval;
        }

        true
    }

    pub fn map(&self, func: impl Fn(&Interval<T>) -> IntervalSet<T>) -> Self {
        let mut intervals = Vec::with_capacity(self.intervals.len());
        for subset in self.intervals.iter() {
            let mut mapped = func(subset);
            intervals.append(&mut mapped.intervals);
        }
        Self::new(intervals)
    }
}

impl<T: Domain> FromIterator<Interval<T>> for IntervalSet<T> {
    fn from_iter<U: IntoIterator<Item = Interval<T>>>(iter: U) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariants() {
        // has empty member
        assert!(!IntervalSet::satisfies_invariants(&vec![
            EBounds::<usize>::empty()
        ]));

        // not disjoint
        assert!(!IntervalSet::satisfies_invariants(&vec![
            EBounds::closed(5, 10),
            EBounds::closed(8, 12)
        ]));
    }
}

use crate::{empty::MaybeEmpty, intersects::Intersects, numeric::Numeric, Normalize};

use super::interval::Interval;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntervalSet<T> {
    pub intervals: Vec<Interval<T>>,
}

/// A Set in Z or R consisting of disjoint contiguous intervals.
///
/// # Invariants
///
/// * All stored intervals are normalized.
/// * No stored interval may be the empty set.
///     * Emptiness is represented by storing no intervals.
///     * Normalized Interval<T> should have a total ordering w/o empty sets.
/// * All intervals are stored in ascending order.
/// * All stored intervals are disjoint subsets of T.
#[allow(dead_code)]
impl<T: Copy + PartialOrd + Numeric> IntervalSet<T> {
    fn new(intervals: Vec<Interval<T>>) -> Self {
        // O(n)
        if Self::satisfies_invariants(&intervals) {
            return Self::new_unchecked(intervals);
        }

        // O(n*log(n))
        let intervals = intervals
            .into_iter()
            .filter_map(|iv| {
                if iv.is_empty() {
                    None
                } else {
                    Some(iv.normalized())
                }
            })
            .collect();

        // todo: sort
        // todo: union

        Self { intervals }
    }

    fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
    }

    fn satisfies_invariants(intervals: &Vec<Interval<T>>) -> bool {
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
}

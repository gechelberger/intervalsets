use crate::{empty::MaybeEmpty, intersects::Intersects, merged::Merged};

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
///     * We do not enforce this here because it should be
///       an invariant of Interval<T> already.
/// * No stored interval may be the empty set.
///     * Emptiness is represented by storing no intervals.
///     * Normalized Interval<T> should have a total ordering w/o empty sets.
/// * All intervals are stored in ascending order.
/// * All stored intervals are disjoint subsets of T.
#[allow(dead_code)]
impl<T: Copy + PartialOrd> IntervalSet<T> {
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
        if !intervals.is_sorted() {
            // O(n*log(n))
            intervals.sort_by(|a, b| {
                a.partial_cmp(b)
                    .expect("Could not sort intervals in IntervalSet because partial_cmp returned None. Likely float NaN")
            });
        }

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

    pub fn new_unchecked(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals }
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

    /// The number of distinct intervals/subsets in this set.
    pub fn count_subsets(&self) -> usize {
        self.intervals.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariants() {
        // has empty member
        assert!(!IntervalSet::satisfies_invariants(&vec![
            Interval::<usize>::empty()
        ]));

        // not disjoint
        assert!(!IntervalSet::satisfies_invariants(&vec![
            Interval::closed(5, 10),
            Interval::closed(8, 12)
        ]));
    }
}

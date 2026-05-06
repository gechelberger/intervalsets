//! Element-wise iteration for [`Interval`] and [`IntervalSet`].
//!
//! Re-exports the [`IntoElementIterator`] trait and [`Elements`] iterator
//! from [`intervalsets_core::ops::elem_iter`], and adds outer-crate impls.
//!
//! `IntervalSet` already implements [`IntoIterator`] for **interval-wise**
//! reading (yielding `Interval<T>` pieces). [`IntoElementIterator`] is
//! the separate, named entry point for **element-wise** reading
//! (yielding `T`), so the two don't compete.

pub use intervalsets_core::ops::{DisjointElements, Elements, IntoElementIterator};

use crate::numeric::Element;
use crate::{Interval, IntervalSet};

// ---- Interval (newtype around EnumInterval) ----

impl<T: Element + Ord> IntoElementIterator for Interval<T> {
    type Item = T;
    type IntoIter = Elements<T>;

    fn into_elements(self) -> Elements<T> {
        self.0.into_elements()
    }
}

impl<T: Element + Ord + Clone> Interval<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    pub fn elements(&self) -> Elements<T> {
        self.0.elements()
    }
}

// ---- IntervalSet ----

/// Iterator over the discrete element values of an [`IntervalSet`].
///
/// Walks each interval piece in order via [`Elements`], advancing through
/// pieces from either end. Implements [`Iterator`],
/// [`DoubleEndedIterator`], and [`core::iter::FusedIterator`], so
/// `.rev()`, mixed forward/backward walks, and the usual combinators all
/// work.
///
/// Constructed via [`IntoElementIterator::into_elements`] (consume) or
/// [`IntervalSet::elements`] (borrow).
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let set = IntervalSet::new([Interval::closed(0, 2), Interval::closed(10, 12)]);
/// let xs: Vec<i32> = set.clone().into_elements().collect();
/// assert_eq!(xs, vec![0, 1, 2, 10, 11, 12]);
///
/// let xs: Vec<i32> = set.into_elements().rev().collect();
/// assert_eq!(xs, vec![12, 11, 10, 2, 1, 0]);
/// ```
pub struct SetElements<T> {
    intervals: std::vec::IntoIter<Interval<T>>,
    front: Option<Elements<T>>,
    back: Option<Elements<T>>,
}

impl<T: Element + Ord> Iterator for SetElements<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            if let Some(it) = self.front.as_mut() {
                if let Some(v) = it.next() {
                    return Some(v);
                }
                self.front = None;
            }
            if let Some(piece) = self.intervals.next() {
                self.front = Some(piece.into_elements());
                continue;
            }
            // No pieces left between cursors; the back walker is in the
            // final remaining piece. Elements<T> is DoubleEndedIterator,
            // so it handles meeting-in-the-middle for the shared piece.
            if let Some(it) = self.back.as_mut() {
                if let Some(v) = it.next() {
                    return Some(v);
                }
                self.back = None;
            }
            return None;
        }
    }
}

impl<T: Element + Ord> DoubleEndedIterator for SetElements<T> {
    fn next_back(&mut self) -> Option<T> {
        loop {
            if let Some(it) = self.back.as_mut() {
                if let Some(v) = it.next_back() {
                    return Some(v);
                }
                self.back = None;
            }
            if let Some(piece) = self.intervals.next_back() {
                self.back = Some(piece.into_elements());
                continue;
            }
            if let Some(it) = self.front.as_mut() {
                if let Some(v) = it.next_back() {
                    return Some(v);
                }
                self.front = None;
            }
            return None;
        }
    }
}

impl<T: Element + Ord> core::iter::FusedIterator for SetElements<T> {}

impl<T: Element + Ord> IntoElementIterator for IntervalSet<T> {
    type Item = T;
    type IntoIter = SetElements<T>;

    fn into_elements(self) -> SetElements<T> {
        SetElements {
            intervals: self.into_raw().into_iter(),
            front: None,
            back: None,
        }
    }
}

impl<T: Element + Ord + Clone> IntervalSet<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    ///
    /// Walks each interval piece in order, yielding every element along
    /// the way. The returned iterator is double-ended.
    pub fn elements(&self) -> SetElements<T> {
        // Clone the slice of intervals so the iterator is owned (no
        // lifetime parameter). Per-interval clones are O(1) bounds; the
        // element walk that follows dominates this cost.
        SetElements {
            intervals: self.slice().to_vec().into_iter(),
            front: None,
            back: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn interval_into_elements() {
        assert!(Interval::closed(0, 4).into_elements().eq([0, 1, 2, 3, 4]));
    }

    #[test]
    fn interval_elements_borrow() {
        let iv = Interval::closed(2u8, 5);
        assert!(iv.elements().eq([2u8, 3, 4, 5]));
        // Still usable.
        assert_eq!(iv.elements().count(), 4);
    }

    #[test]
    fn interval_set_into_elements_multi_piece() {
        let set = IntervalSet::new([
            Interval::closed(0, 2),
            Interval::closed(10, 12),
        ]);
        let collected: Vec<i32> = set.into_elements().collect();
        assert_eq!(collected, vec![0, 1, 2, 10, 11, 12]);
    }

    #[test]
    fn interval_set_elements_borrow() {
        let set = IntervalSet::new([
            Interval::closed(0u8, 1),
            Interval::closed(253, 255),
        ]);
        let collected: Vec<u8> = set.elements().collect();
        assert_eq!(collected, vec![0, 1, 253, 254, 255]);
        // Still usable.
        let again: Vec<u8> = set.elements().collect();
        assert_eq!(again, vec![0, 1, 253, 254, 255]);
    }

    #[test]
    fn interval_set_empty_yields_nothing() {
        let set = IntervalSet::<i32>::empty();
        assert_eq!(set.into_elements().next(), None);
    }

    #[test]
    fn interval_set_singleton_piece_yields_once() {
        let set = IntervalSet::new([Interval::closed(7, 7)]);
        let collected: Vec<i32> = set.into_elements().collect();
        assert_eq!(collected, vec![7]);
    }

    #[test]
    fn interval_set_with_half_bounded_walks_until_max() {
        // Right-most piece is half-bounded → continues to type MAX.
        let set = IntervalSet::new([
            Interval::closed(0u8, 1),
            Interval::closed_unbound(254),
        ]);
        let collected: Vec<u8> = set.into_elements().collect();
        assert_eq!(collected, vec![0, 1, 254, 255]);
    }

    // ---- DoubleEndedIterator coverage ----

    #[test]
    fn interval_set_into_elements_reversed() {
        let set = IntervalSet::new([
            Interval::closed(0, 2),
            Interval::closed(10, 12),
        ]);
        let collected: Vec<i32> = set.into_elements().rev().collect();
        assert_eq!(collected, vec![12, 11, 10, 2, 1, 0]);
    }

    #[test]
    fn interval_set_next_back_only() {
        let set = IntervalSet::new([
            Interval::closed(0, 1),
            Interval::closed(10, 11),
        ]);
        let mut it = set.into_elements();
        assert_eq!(it.next_back(), Some(11));
        assert_eq!(it.next_back(), Some(10));
        assert_eq!(it.next_back(), Some(1));
        assert_eq!(it.next_back(), Some(0));
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn interval_set_meeting_across_three_singletons() {
        // Three single-element pieces; mixed direction walks should yield
        // every element exactly once.
        let set = IntervalSet::new([
            Interval::closed(0, 0),
            Interval::closed(5, 5),
            Interval::closed(10, 10),
        ]);
        let mut it = set.into_elements();
        assert_eq!(it.next_back(), Some(10));
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next(), Some(5));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn interval_set_meeting_inside_one_piece() {
        // Two pieces; consume one from front, then one from back, then
        // both walkers converge on the remaining elements of one piece.
        let set = IntervalSet::new([
            Interval::closed(0, 1),
            Interval::closed(10, 14),
        ]);
        let mut it = set.into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next_back(), Some(14));
        assert_eq!(it.next(), Some(10));
        assert_eq!(it.next_back(), Some(13));
        assert_eq!(it.next(), Some(11));
        assert_eq!(it.next_back(), Some(12));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn interval_set_single_piece_mixed_walk() {
        // One piece, mixed directions inside it.
        let set = IntervalSet::new([Interval::closed(0, 4)]);
        let mut it = set.into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next_back(), Some(4));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next_back(), Some(3));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn interval_set_empty_yields_nothing_in_reverse() {
        let set = IntervalSet::<i32>::empty();
        assert_eq!(set.into_elements().next_back(), None);
    }

    #[test]
    fn interval_set_left_half_bounded_in_reverse_walks_to_min() {
        // Leftmost piece is `(-∞, 2]`; reverse iteration should walk it
        // down to type MIN.
        let set = IntervalSet::new([
            Interval::unbound_closed(2u8),
            Interval::closed(100, 101),
        ]);
        let collected: Vec<u8> =
            set.into_elements().rev().take(5).collect();
        assert_eq!(collected, vec![101, 100, 2, 1, 0]);
    }

    #[test]
    fn interval_set_borrow_reverse() {
        let set = IntervalSet::new([
            Interval::closed(0u8, 2),
            Interval::closed(10, 11),
        ]);
        let collected: Vec<u8> = set.elements().rev().collect();
        assert_eq!(collected, vec![11, 10, 2, 1, 0]);
        // Still usable.
        let again: Vec<u8> = set.elements().collect();
        assert_eq!(again, vec![0, 1, 2, 10, 11]);
    }
}

//! Element-wise iteration for [`Interval`] and [`IntervalSet`].
//!
//! Re-exports the [`IntoElementIterator`] trait and [`Elements`] iterator
//! from [`intervalsets_core::ops::elem_iter`], and adds outer-crate impls.
//!
//! `IntervalSet` already implements [`IntoIterator`] for **interval-wise**
//! reading (yielding `Interval<T>` pieces). [`IntoElementIterator`] is
//! the separate, named entry point for **element-wise** reading
//! (yielding `T`), so the two don't compete.

pub use intervalsets_core::ops::{Elements, IntoElementIterator};

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
/// Walks each interval piece in ascending order via [`Elements`], then
/// advances to the next piece. Implements [`Iterator`] and
/// [`core::iter::FusedIterator`].
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
/// let xs: Vec<i32> = set.into_elements().collect();
/// assert_eq!(xs, vec![0, 1, 2, 10, 11, 12]);
/// ```
pub struct SetElements<T> {
    intervals: std::vec::IntoIter<Interval<T>>,
    current: Option<Elements<T>>,
}

impl<T: Element + Ord> Iterator for SetElements<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            if let Some(it) = self.current.as_mut() {
                if let Some(v) = it.next() {
                    return Some(v);
                }
            }
            // Current piece exhausted; advance to the next one.
            let next_interval = self.intervals.next()?;
            self.current = Some(next_interval.into_elements());
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
            current: None,
        }
    }
}

impl<T: Element + Ord + Clone> IntervalSet<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    ///
    /// Walks each interval piece in ascending order, yielding every
    /// element along the way.
    pub fn elements(&self) -> SetElements<T> {
        // Clone the slice of intervals so the iterator is owned (no
        // lifetime parameter). Per-interval clones are O(1) bounds; the
        // element walk that follows dominates this cost.
        SetElements {
            intervals: self.slice().to_vec().into_iter(),
            current: None,
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
}

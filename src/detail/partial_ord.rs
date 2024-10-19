use std::cmp::Ordering;

use crate::numeric::Domain;
use crate::{Bound, Bounding, MaybeEmpty, Side};

use super::{BoundCase, Finite, HalfBounded};

/// Partial compare of two boundary conditions
/// when both are the same side of each interval.
///
/// The IVals are assumed to be the left or right
/// bounds of two *non-empty* intervals and values
/// of `None` are assumed to indicate an infinite
/// bound.
///
/// # Examples:
///
/// 1) left case:  (a, _) partial_cmp to (b, _)
/// 2) right case: (_, a) partial_cmp to (_, b)
fn non_empty_cmp_side<T: PartialOrd>(
    side: Side,
    left: Option<&Bound<T>>,
    right: Option<&Bound<T>>,
) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(right)) => match side {
            Side::Left => Ordering::Less,
            Side::Right => Ordering::Greater,
        },
        (Some(left), None) => match side {
            Side::Left => Ordering::Greater,
            Side::Right => Ordering::Less,
        },
        (Some(left), Some(right)) => {
            if left == right {
                return Ordering::Equal;
            }

            match side {
                Side::Left => {
                    if left.contains(side, right.value()) {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                Side::Right => {
                    if left.contains(side, right.value()) {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
            }
        }
    }
}

/// binary cmp for two types that impl the Bounds trait
fn impl_cmp<U, T>(lhs: &U, rhs: &U) -> std::cmp::Ordering
where
    T: Clone + PartialOrd,
    U: Bounding<T>,
{
    match non_empty_cmp_side(Side::Left, lhs.left(), rhs.left()) {
        Ordering::Equal => non_empty_cmp_side(Side::Right, lhs.right(), rhs.right()),
        ordering => ordering,
    }
}

/// A generic impl of partial_cmp in terms of the `Bounds` trait.
/// This is done as a free generic function to make it easy to implement
/// `PartialOrd` for types without resorting to a blanket implementation.
fn impl_partial_cmp<U, T>(lhs: &U, rhs: &U) -> Option<std::cmp::Ordering>
where
    T: Clone + PartialOrd,
    U: Bounding<T> + MaybeEmpty,
{
    if lhs.is_empty() || rhs.is_empty() {
        return None;
    }

    impl_cmp(lhs, rhs).into()
}

impl<T: Domain> PartialOrd for BoundCase<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_partial_cmp(self, rhs)
    }
}

impl<T: Domain> PartialOrd for HalfBounded<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_cmp(self, rhs).into()
    }
}

impl<T: Domain> PartialOrd for Finite<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_partial_cmp(self, rhs)
    }
}

fn impl_total_ord_cmp<T, S>(lhs: &S, rhs: &S) -> std::cmp::Ordering
where
    T: Domain + Eq + Ord,
    S: Bounding<T> + MaybeEmpty,
{
    match (lhs.is_empty(), rhs.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => impl_cmp(lhs, rhs),
    }
}

impl<T: Domain + Eq> Eq for Finite<T> {}
impl<T: Domain + Ord> Ord for Finite<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        impl_total_ord_cmp(self, other)
    }
}

impl<T: Domain + Eq> Eq for HalfBounded<T> {}
impl<T: Domain + Ord> Ord for HalfBounded<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        impl_cmp(self, other)
    }
}

impl<T: Domain + Eq> Eq for BoundCase<T> {}
impl<T: Domain + Ord> Ord for BoundCase<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        impl_total_ord_cmp(self, other)
    }
}

/*
// this might be interesting to try as an impl for a while
// just because it's less likely to silently swallow an ordering issue.

fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.partial_cmp(other) {
        Some(ordering) => ordering,
        // one or both are the empty set (or else something is broken).
        // we choose to put empty in a total ordering first before any other.
        None => match (self.is_empty(), other.is_empty()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => panic!("T claims Ord; We done did screwed up somewhere")
        }
    }
}
*/

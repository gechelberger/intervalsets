use std::cmp::Ordering;

use crate::Bounding;
use crate::MaybeEmpty;
use crate::{Side, Bound};
use crate::numeric::Domain;

use super::{Finite, HalfBounded, BoundCase};

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

use std::cmp::Ordering;

use crate::bounds::Bounds;
use crate::empty::MaybeEmpty;
use crate::ival::{IVal, Side};
use crate::numeric::Domain;
use crate::{FiniteInterval, HalfBounded, Interval};

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
fn non_empty_cmp_side<T: Domain>(
    side: Side,
    left: Option<IVal<T>>,
    right: Option<IVal<T>>,
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
                    if left.contains(side, &right.value) {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                Side::Right => {
                    if left.contains(side, &right.value) {
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
    T: Domain,
    U: Bounds<T>,
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
    T: Domain,
    U: Bounds<T> + MaybeEmpty,
{
    if lhs.is_empty() || rhs.is_empty() {
        return None;
    }

    impl_cmp(lhs, rhs).into()
}

impl<T: Domain> PartialOrd for Interval<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_partial_cmp(self, rhs)
    }
}

impl<T: Domain> PartialOrd for HalfBounded<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_cmp(self, rhs).into()
    }
}

impl<T: Domain> PartialOrd for FiniteInterval<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_partial_cmp(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_lt(Interval::unbound_open(50.0), Interval::unbound());

        // (<-, ->) < (0, 50)
        assert_lt(Interval::unbound(), Interval::open(0.0, 50.0));

        // (<-, ->) < (0, ->)
        assert_lt(Interval::unbound(), Interval::open_unbound(0.0));

        // Empty Set should not compare
        assert_eq!(Interval::<u8>::empty() <= Interval::<u8>::unbound(), false);
        assert_eq!(Interval::<u8>::empty() >= Interval::<u8>::unbound(), false);
    }
}

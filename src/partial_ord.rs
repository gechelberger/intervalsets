use std::cmp::Ordering;
use std::ops::Sub;

use num::Zero;

use crate::empty::MaybeEmpty;
use crate::ival::{IVal, Side};
use crate::Interval;
use crate::bounds::Bounds;

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
fn non_empty_cmp_side<T: PartialEq + PartialOrd>(
    side: Side, 
    left: Option<IVal<T>>,
    right: Option<IVal<T>>,
) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (None, None) => Some(Ordering::Equal),
        (None, Some(right)) => match side {
            Side::Left => Some(Ordering::Less),
            Side::Right => Some(Ordering::Greater)
        }
        (Some(left), None) => match side {
            Side::Left => Some(Ordering::Greater),
            Side::Right => Some(Ordering::Less)
        }
        (Some(left), Some(right)) => {
            if left == right {
                return Some(Ordering::Equal);
            } 
            
            match side {
                Side::Left => {
                    if left.contains(side, &right.value) {
                        Some(Ordering::Less)
                    } else {
                        Some(Ordering::Greater)
                    }
                },
                Side::Right => {
                    if left.contains(side, &right.value) {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    }
                },
            }
        }
    }
}


/// A generic impl of partial_cmp in terms of the `Bounds` trait.
/// This is done as a free generic function to make it easy to implement
/// `PartialOrd` for types without resorting to a blanket implementation.
fn impl_partial_cmp<U, T>(lhs: &U, rhs: &U) -> Option<std::cmp::Ordering> 
where 
    T: Copy + PartialOrd,
    U: Bounds<T> + MaybeEmpty
{
    if lhs.is_empty() || rhs.is_empty() {
        return None;
    }

    match non_empty_cmp_side(Side::Left, lhs.left(), rhs.left()) {
        None => non_empty_cmp_side(Side::Right, lhs.right(), rhs.right()),
        Some(ordering) => match ordering {
            Ordering::Equal => non_empty_cmp_side(Side::Right, lhs.right(), rhs.right()),
            _ => Some(ordering)
        }
    }
} 

impl<T: Copy + PartialOrd + PartialEq> PartialOrd for Interval<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        impl_partial_cmp(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_lt<T: Copy + PartialOrd + PartialEq>(itv1: Interval<T>, itv2: Interval<T>) {
        assert!(itv1 < itv2);
        assert!(!(itv1 >= itv2)); // antisymmetry

        assert!(itv2 > itv1); // duality
        assert!(!(itv2 <= itv1)); // antisymmetry
    }

    #[test]
    fn test_interval_cmp() {
        // (0, _) < (200, _)
        assert_lt(Interval::open(0.0, 100.0), Interval::open(200.0, 300.0));

        // [0, _] < (0.0, _)
        assert_lt(Interval::closed(0.0, 100.0), Interval::open(0.0, 100.0));

        // (<-, _) < (0.0, _)
        assert_lt(Interval::unbound_open(5.0), Interval::open(0.0, 3.0));
        
    }
}
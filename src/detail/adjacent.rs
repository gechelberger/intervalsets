use crate::numeric::Domain;
use crate::util::commutative_predicate_impl;
use crate::{Bound, Side};

use super::{BoundCase, Finite, HalfBounded};

/// Defines whether two sets are contiguous.
///
/// Given two Sets A and B which are both
/// Subsets of T:
///
/// > A and B are adjacent if their extrema
/// > have no elements in T between them.
///
/// # Example
///
/// > [1, 5] is adjacent to [6, 10]
///
/// > [1.0, 5.0] is not adjacent to [6.0, 10.0]
///
pub trait Adjacent<Rhs = Self> {
    fn is_adjacent_to(&self, rhs: &Rhs) -> bool;
}

fn are_continuous_adjacent<T: PartialEq>(right: &Bound<T>, left: &Bound<T>) -> bool {
    // not sure how to deal with the rounding issues of floats
    right.value() == left.value() && (right.is_closed() || left.is_closed())
}

/// <---][---->
fn are_adjacent<T: Domain>(right: &Bound<T>, left: &Bound<T>) -> bool {
    let right_up = right.value().try_adjacent(Side::Right);
    let left_down = left.value().try_adjacent(Side::Left);

    match (right_up, left_down) {
        (None, None) => are_continuous_adjacent(right, left),
        (None, _) => false, // assume we are at T::Max
        (_, None) => false, // assume we are at T::Min
        (Some(right_up), Some(left_down)) => {
            right_up == *left.value() && left_down == *right.value()
        }
    }
}

impl<T: Domain> Adjacent<Self> for Finite<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        self.ref_map_or(false, |a_left, a_right| {
            rhs.ref_map_or(false, |b_left, b_right| {
                are_adjacent(a_right, b_left) || are_adjacent(b_right, a_left)
            })
        })
        /*
        match self {
            Self::FullyBounded(a_left, a_right) => match rhs {
                Self::FullyBounded(b_left, b_right) => {
                    are_adjacent(a_right, b_left) || are_adjacent(b_right, a_left)
                }
                Self::Empty => false,
            },
            Self::Empty => false,
        }*/
    }
}

impl<T: Domain> Adjacent<Self> for HalfBounded<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        match (self.side, rhs.side) {
            (Side::Left, Side::Right) => are_adjacent(&rhs.bound, &self.bound),
            (Side::Right, Side::Left) => are_adjacent(&self.bound, &rhs.bound),
            _ => false,
        }
    }
}

impl<T: Domain> Adjacent<HalfBounded<T>> for Finite<T> {
    fn is_adjacent_to(&self, rhs: &HalfBounded<T>) -> bool {
        self.ref_map_or(false, |left, right| match rhs.side {
            Side::Left => are_adjacent(right, &rhs.bound),
            Side::Right => are_adjacent(&rhs.bound, left),
        })
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, HalfBounded<T>, Finite<T>);

impl<T: Domain> Adjacent<Finite<T>> for BoundCase<T> {
    fn is_adjacent_to(&self, rhs: &Finite<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, Finite<T>, BoundCase<T>);

impl<T: Domain> Adjacent<HalfBounded<T>> for BoundCase<T> {
    fn is_adjacent_to(&self, rhs: &HalfBounded<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, HalfBounded<T>, BoundCase<T>);

impl<T: Domain> Adjacent<Self> for BoundCase<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        match rhs {
            Self::Finite(rhs) => self.is_adjacent_to(rhs),
            Self::Half(rhs) => self.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Factory, Interval};

    #[test]
    fn test_is_adjacent_to() {
        assert_eq!(
            Interval::closed(0, 10)
                .0
                .is_adjacent_to(&Interval::closed(10, 20).0),
            false
        );

        assert_eq!(
            Interval::closed(0, 10)
                .0
                .is_adjacent_to(&Interval::closed(11, 20).0),
            true
        );

        assert_eq!(
            Interval::closed(0.0, 10.0)
                .0
                .is_adjacent_to(&Interval::closed(10.0, 20.0).0),
            true
        );

        assert_eq!(
            Interval::open(0.0, 10.0)
                .0
                .is_adjacent_to(&Interval::open(10.0, 20.0).0),
            false,
        );
    }
}

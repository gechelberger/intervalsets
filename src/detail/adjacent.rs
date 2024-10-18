use crate::numeric::Domain;
use crate::ops::Adjacent;
use crate::util::commutative_predicate_impl;
use crate::{Bound, Side};

use super::{BoundCase, Finite, HalfBounded};

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
        match self {
            Self::FullyBounded(a_left, a_right) => match rhs {
                Self::FullyBounded(b_left, b_right) => {
                    are_adjacent(a_right, b_left) || are_adjacent(b_right, a_left)
                }
                Self::Empty => false,
            },
            Self::Empty => false,
        }
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
        self.map_or(false, |left, right| match rhs.side {
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

use FiniteInterval::Bounded;

use super::util::commutative_predicate_impl;
use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

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
    fn is_adjacent_to(&self, rhs: Rhs) -> bool;
}

fn are_continuous_adjacent<T: PartialEq>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
    // not sure how to deal with the rounding issues of floats
    right.value() == left.value() && (right.is_closed() || left.is_closed())
}

/// <---][---->
fn are_adjacent<T: Domain>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
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

impl<T: Domain> Adjacent<&FiniteBound<T>> for FiniteInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteBound<T>) -> bool {
        let Self::Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        are_adjacent(lhs_max, rhs) || are_adjacent(rhs, lhs_min)
    }
}

impl<T: Domain> Adjacent<&Self> for FiniteInterval<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        are_adjacent(lhs_max, rhs_min) || are_adjacent(rhs_max, lhs_min)
    }
}

impl<T: Domain> Adjacent<&FiniteBound<T>> for HalfInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteBound<T>) -> bool {
        match self.side {
            Side::Left => are_adjacent(rhs, &self.bound),
            Side::Right => are_adjacent(&self.bound, rhs),
        }
    }
}

impl<T: Domain> Adjacent<&Self> for HalfInterval<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        match (self.side, rhs.side) {
            (Side::Left, Side::Right) => are_adjacent(&rhs.bound, &self.bound),
            (Side::Right, Side::Left) => are_adjacent(&self.bound, &rhs.bound),
            _ => false,
        }
    }
}

impl<T: Domain> Adjacent<&HalfInterval<T>> for FiniteInterval<T> {
    fn is_adjacent_to(&self, rhs: &HalfInterval<T>) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        match rhs.side {
            Side::Left => are_adjacent(lhs_max, &rhs.bound),
            Side::Right => are_adjacent(&rhs.bound, lhs_min),
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, HalfInterval<T>, FiniteInterval<T>);

impl<T: Domain> Adjacent<&FiniteInterval<T>> for EnumInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, FiniteInterval<T>, EnumInterval<T>);

impl<T: Domain> Adjacent<&HalfInterval<T>> for EnumInterval<T> {
    fn is_adjacent_to(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, HalfInterval<T>, EnumInterval<T>);

impl<T: Domain> Adjacent<&Self> for EnumInterval<T> {
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        match rhs {
            Self::Finite(rhs) => self.is_adjacent_to(rhs),
            Self::Half(rhs) => self.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Factory};
    use crate::sets::IntervalEnum;

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
*/

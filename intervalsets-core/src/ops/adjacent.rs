use FiniteInterval::Bounded;

use super::util::commutative_predicate_impl;
use crate::bound::{FiniteBound, Side};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if two sets are adjacent.
///
/// Sets are adjacent if they are connected but do not intersect. For sets to be
/// connected no elements may exist in the universal set (or data type set)
/// between the two subsets. The empty set is considered adjacent and connect to
/// any other set since no elements exist between the two.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(1, 5);
/// let y = FiniteInterval::closed(6, 10);
/// assert_eq!(x.is_adjacent_to(&y), true);
///
/// let x = FiniteInterval::closed(0.0, 5.0);
/// let y = FiniteInterval::closed(6.0, 10.0);
/// assert_eq!(x.is_adjacent_to(&y), false);
///
/// let y = FiniteInterval::open(5.0, 10.0);
/// assert_eq!(x.is_adjacent_to(&y), true);
/// ```
pub trait Adjacent<Rhs = Self> {
    #[allow(missing_docs)]
    fn is_adjacent_to(&self, rhs: Rhs) -> bool;
}

#[inline(always)]
fn are_continuous_adjacent<T: PartialEq>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
    // not sure how to deal with float rounding
    right.value() == left.value()
        && ((right.is_closed() && left.is_open()) || (left.is_closed() && right.is_open()))

    // closed_open or open_closed
    // if closed = 1, open = 0 don't branch =>
    //     left.bound_type() as u32 + right.bound_type() as u32 == 1
}

/// <---][---->
#[inline(always)]
fn are_adjacent<T: Element>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
    let right_up = right.value().try_adjacent(Side::Right);
    let left_down = left.value().try_adjacent(Side::Left);

    match (right_up, left_down) {
        (None, None) => are_continuous_adjacent(right, left),
        (None, Some(_)) => {
            // Open(T::MAX) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            right.is_closed() && left.is_open() && right.value() == left.value()
        }
        (Some(_), None) => {
            // Open(T::MIN) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            right.is_open() && left.is_closed() && right.value() == left.value()
        }
        (Some(right_up), Some(left_down)) => {
            right_up == *left.value() && left_down == *right.value()
        }
    }
}

impl<T: Element> Adjacent<&FiniteBound<T>> for FiniteInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteBound<T>) -> bool {
        let Self::Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        are_adjacent(lhs_max, rhs) || are_adjacent(rhs, lhs_min)
    }
}

impl<T: Element> Adjacent<&Self> for FiniteInterval<T> {
    #[inline(always)]
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

impl<T: Element> Adjacent<&FiniteBound<T>> for HalfInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteBound<T>) -> bool {
        match self.side {
            Side::Left => are_adjacent(rhs, &self.bound),
            Side::Right => are_adjacent(&self.bound, rhs),
        }
    }
}

impl<T: Element> Adjacent<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn is_adjacent_to(&self, rhs: &Self) -> bool {
        match (self.side, rhs.side) {
            (Side::Left, Side::Right) => are_adjacent(&rhs.bound, &self.bound),
            (Side::Right, Side::Left) => are_adjacent(&self.bound, &rhs.bound),
            _ => false,
        }
    }
}

impl<T: Element> Adjacent<&HalfInterval<T>> for FiniteInterval<T> {
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

impl<T: Element> Adjacent<&FiniteInterval<T>> for EnumInterval<T> {
    fn is_adjacent_to(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, FiniteInterval<T>, EnumInterval<T>);

impl<T: Element> Adjacent<&HalfInterval<T>> for EnumInterval<T> {
    fn is_adjacent_to(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.is_adjacent_to(rhs),
            Self::Half(lhs) => lhs.is_adjacent_to(rhs),
            Self::Unbounded => false,
        }
    }
}
commutative_predicate_impl!(Adjacent, is_adjacent_to, HalfInterval<T>, EnumInterval<T>);

impl<T: Element> Adjacent<&Self> for EnumInterval<T> {
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

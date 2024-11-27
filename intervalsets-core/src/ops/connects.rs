use super::util::commutative_predicate_impl;
use super::Intersects;
use crate::bound::{FiniteBound, Side};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if two sets are connected.
///
/// Sets are connected if the elements of both form an ordered chain of
/// values with no gaps. The empty set is trivially connected to
/// any other interval since no elements exist between the two.
///
/// More formally, two sets are connected if their union can not be
/// partitioned into two or more disjoint non-empty open subsets.
///
/// All intersecting intervals are therefore connected.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(1, 5);
/// let y = FiniteInterval::closed(6, 10);
/// assert_eq!(x.connects(&y), true);
///
/// let x = FiniteInterval::closed(0.0, 5.0);
/// let y = FiniteInterval::closed(6.0, 10.0);
/// assert_eq!(x.connects(&y), false);
///
/// let y = FiniteInterval::open(5.0, 10.0);
/// assert_eq!(x.connects(&y), true);
///
/// let z = FiniteInterval::open(0.0, 5.0);
/// assert_eq!(y.connects(&z), false);
/// ```
pub trait Connects<Rhs = Self> {
    #[allow(missing_docs)]
    fn connects(&self, rhs: Rhs) -> bool;
}

/// Treating each bound as a singleton set, are they connected?
///
/// <--][----> | yes
/// <--)[----> | yes
/// <--](----> | yes
/// <--)(----> | no
/// <--]  [--> | no
/// <--[--]--> | no
#[inline(always)]
pub fn are_bounds_connected<T: Element>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
    let right_up = right.value().try_adjacent(Side::Right);
    let left_down = left.value().try_adjacent(Side::Left);

    match (right_up, left_down) {
        (None, None) => right.value() == left.value() && (left.is_closed() || right.is_closed()),
        (None, Some(_)) => {
            // Open(T::MAX) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            right.is_closed() && right.value() == left.value()
        }
        (Some(_), None) => {
            // Open(T::MIN) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            left.is_closed() && right.value() == left.value()
        }
        (Some(right_up), Some(left_down)) => {
            right_up == *left.value() && left_down == *right.value()
        }
    }
}

impl<T: Element> Connects<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        if self.intersects(rhs) {
            return true;
        }

        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return true;
        };

        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return true;
        };

        are_bounds_connected(lhs_max, rhs_min) || are_bounds_connected(rhs_max, lhs_min)
    }
}

impl<T: Element> Connects<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        self.intersects(rhs)
            || match (self.side(), rhs.side()) {
                (Side::Left, Side::Right) => {
                    are_bounds_connected(rhs.finite_bound(), self.finite_bound())
                }
                (Side::Right, Side::Left) => {
                    are_bounds_connected(self.finite_bound(), rhs.finite_bound())
                }
                _ => false,
            }
    }
}

impl<T: Element> Connects<&HalfInterval<T>> for FiniteInterval<T> {
    fn connects(&self, rhs: &HalfInterval<T>) -> bool {
        if self.intersects(rhs) {
            return true;
        }

        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return true;
        };

        match rhs.side() {
            Side::Left => are_bounds_connected(lhs_max, rhs.finite_bound()),
            Side::Right => are_bounds_connected(rhs.finite_bound(), lhs_min),
        }
    }
}
commutative_predicate_impl!(Connects, connects, HalfInterval<T>, FiniteInterval<T>);

impl<T: Element> Connects<&FiniteInterval<T>> for EnumInterval<T> {
    fn connects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.connects(rhs),
            Self::Half(lhs) => lhs.connects(rhs),
            Self::Unbounded => true,
        }
    }
}
commutative_predicate_impl!(Connects, connects, FiniteInterval<T>, EnumInterval<T>);

impl<T: Element> Connects<&HalfInterval<T>> for EnumInterval<T> {
    fn connects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.connects(rhs),
            Self::Half(lhs) => lhs.connects(rhs),
            Self::Unbounded => true,
        }
    }
}
commutative_predicate_impl!(Connects, connects, HalfInterval<T>, EnumInterval<T>);

impl<T: Element> Connects<&Self> for EnumInterval<T> {
    fn connects(&self, rhs: &Self) -> bool {
        match rhs {
            Self::Finite(rhs) => self.connects(rhs),
            Self::Half(rhs) => self.connects(rhs),
            Self::Unbounded => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;
    use crate::EnumInterval as EI;

    #[test]
    fn test_connects() {
        assert_eq!(EI::closed(0, 10).connects(&EI::empty()), true);
        assert_eq!(EI::empty().connects(&EI::closed(0, 10)), true);

        assert_eq!(EI::closed(0, 10).connects(&EI::closed(10, 20)), true);
        assert_eq!(EI::closed(0, 10).connects(&EI::closed(11, 20)), true);
        assert_eq!(EI::closed(0, 10).connects(&EI::closed(12, 20)), false);

        assert_eq!(
            EI::closed(0.0, 10.0).connects(&EI::closed(10.0, 20.0)),
            true
        );
        assert_eq!(EI::closed(0.0, 10.0).connects(&EI::open(10.0, 20.0)), true);
        assert_eq!(EI::open(0.0, 10.0).connects(&EI::closed(10.0, 20.0)), true);
        assert_eq!(EI::open(0.0, 10.0).connects(&EI::open(10.0, 20.0)), false);

        assert_eq!(
            EI::open(0.0, 10.0).connects(&EI::closed_unbound(10.0)),
            true
        );

        assert_eq!(EI::unbounded().connects(&EI::closed(0, 10)), true);
    }
}

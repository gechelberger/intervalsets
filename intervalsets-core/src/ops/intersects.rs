use super::util::commutative_predicate_impl;
use super::Contains;
use crate::bound::ord::{OrdBound, OrdBounded};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if two sets intersect.
///
/// ```text
/// ∃x | x ∈ A ∧ x ∈ B
/// ```
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = EnumInterval::closed(10, 20);
/// let y = EnumInterval::closed_unbound(15);
/// assert_eq!(x.intersects(&y), true);
///
/// let y = EnumInterval::empty();
/// assert_eq!(x.intersects(&y), false);
/// ```
pub trait Intersects<T> {
    /// Tests if two sets share any elements.
    fn intersects(&self, rhs: T) -> bool;

    /// Tests if two sets share no elements.
    fn is_disjoint_from(&self, rhs: T) -> bool {
        !self.intersects(rhs)
    }
}

impl<T: PartialOrd> Intersects<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        /*let Self::Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let Self::Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        lhs_min.contains(Side::Left, rhs_max.value())
            && rhs_min.contains(Side::Left, lhs_max.value())
            && rhs_max.contains(Side::Right, lhs_min.value())
            && lhs_max.contains(Side::Right, rhs_min.value())*/

        let (lhs_min, lhs_max) = self.ord_bound_pair().into_raw();
        let (rhs_min, rhs_max) = rhs.ord_bound_pair().into_raw();
        lhs_max != OrdBound::LeftUnbounded
            && rhs_max != OrdBound::LeftUnbounded
            && lhs_min <= rhs_max
            && rhs_min <= lhs_max
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for HalfInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        let FiniteInterval::Bounded(left, right) = rhs else {
            return false;
        };

        self.contains(left.ord(crate::bound::Side::Left))
            || self.contains(right.ord(crate::bound::Side::Right))
    }
}

impl<T: PartialOrd> Intersects<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        self.contains(rhs.bound.value()) || rhs.contains(self.bound.value())
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.intersects(rhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: PartialOrd> Intersects<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Intersects<&Self> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => rhs.intersects(lhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty.into(),
        }
    }
}

commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, HalfInterval<T>);
commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, EnumInterval<T>);
commutative_predicate_impl!(Intersects, intersects, HalfInterval<T>, EnumInterval<T>);

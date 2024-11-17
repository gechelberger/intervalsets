use super::util::commutative_predicate_impl;
use super::{Contains, Intersects};
use crate::bound::ord::{OrdBound, OrdBounded};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

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

        self.contains(left.value()) || self.contains(right.value())
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

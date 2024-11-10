use super::util::commutative_predicate_impl;
use super::{Contains, Intersects};
use crate::bound::ord::OrdBounded;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

pub fn intersects_ord<T: PartialOrd>(lhs: &FiniteInterval<T>, rhs: &FiniteInterval<T>) -> bool {
    let lhs = lhs.ord_bound_pair();
    let rhs = rhs.ord_bound_pair();
    let empty = lhs.is_empty() || rhs.is_empty();
    let (lhs_min, lhs_max) = lhs.into_raw();
    let (rhs_min, rhs_max) = rhs.into_raw();
    lhs_min <= rhs_max && rhs_min <= lhs_max && !empty
}

impl<T: PartialOrd> Intersects<&Self> for FiniteInterval<T> {
    #[inline]
    fn intersects(&self, rhs: &Self) -> bool {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        let empty = lhs.is_empty() || rhs.is_empty();
        let (lhs_min, lhs_max) = lhs.into_raw();
        let (rhs_min, rhs_max) = rhs.into_raw();
        lhs_min <= rhs_max && rhs_min <= lhs_max && !empty
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for HalfInterval<T> {
    #[inline]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        let FiniteInterval::Bounded(left, right) = rhs else {
            return false;
        };

        self.contains(left.value()) || self.contains(right.value())
    }
}

impl<T: PartialOrd> Intersects<&Self> for HalfInterval<T> {
    #[inline]
    fn intersects(&self, rhs: &Self) -> bool {
        self.contains(rhs.bound.value()) || rhs.contains(self.bound.value())
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.intersects(rhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: PartialOrd> Intersects<&HalfInterval<T>> for EnumInterval<T> {
    #[inline]
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Intersects<&Self> for EnumInterval<T> {
    #[inline]
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

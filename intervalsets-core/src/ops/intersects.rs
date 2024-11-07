use super::util::commutative_predicate_impl;
use super::{Contains, Intersects};
use crate::bound::Side;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Domain> Intersects<Self> for FiniteInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.ref_map(|l1, r1| {
            rhs.ref_map(|l2, r2| {
                l1.contains(Side::Left, r2.value())
                    && l2.contains(Side::Left, r1.value())
                    && r1.contains(Side::Right, l1.value())
                    && r2.contains(Side::Right, l1.value())
            })
            .unwrap_or(false)
        })
        .unwrap_or(false)
    }
}

impl<T: Domain> Intersects<FiniteInterval<T>> for HalfInterval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.ref_map(|left, right| self.contains(left.value()) || self.contains(right.value()))
            .unwrap_or(false)
    }
}

impl<T: Domain> Intersects<Self> for HalfInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.contains(rhs.bound.value()) || rhs.contains(self.bound.value())
    }
}

impl<T: Domain> Intersects<FiniteInterval<T>> for EnumInterval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.intersects(rhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: Domain> Intersects<HalfInterval<T>> for EnumInterval<T> {
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: Domain> Intersects<Self> for EnumInterval<T> {
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

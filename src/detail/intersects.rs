use crate::{commutative_predicate_impl, Contains, Domain, Intersects, Side};

use super::{BoundCase, Finite, HalfBounded};


impl<T: Domain> Intersects<Self> for Finite<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.map_or::<bool>(false, |l1, r1| {
            rhs.map_or::<bool>(false, |l2, r2| {
                l1.contains(Side::Left, r2.value())
                    && l2.contains(Side::Left, r1.value())
                    && r1.contains(Side::Right, l1.value())
                    && r2.contains(Side::Right, l1.value())
            })
        })
    }
}

impl<T: Domain> Intersects<Finite<T>> for HalfBounded<T> {
    fn intersects(&self, rhs: &Finite<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(left.value()) || self.contains(right.value())
        })
    }
}

impl<T: Domain> Intersects<Self> for HalfBounded<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.contains(rhs.bound.value()) || rhs.contains(self.bound.value())
    }
}

impl<T: Domain> Intersects<Finite<T>> for BoundCase<T> {
    fn intersects(&self, rhs: &Finite<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.intersects(rhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => *rhs != Finite::Empty,
        }
    }
}

impl<T: Domain> Intersects<HalfBounded<T>> for BoundCase<T> {
    fn intersects(&self, rhs: &HalfBounded<T>) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: Domain> Intersects<Self> for BoundCase<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => rhs.intersects(lhs),
            Self::Unbounded => *rhs != Finite::Empty.into(),
        }
    }
}

commutative_predicate_impl!(Intersects, intersects, Finite<T>, HalfBounded<T>);
commutative_predicate_impl!(Intersects, intersects, Finite<T>, BoundCase<T>);
commutative_predicate_impl!(Intersects, intersects, HalfBounded<T>, BoundCase<T>);
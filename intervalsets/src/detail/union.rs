use crate::numeric::Domain;
use crate::ops::{Merged, RefMerged, RefUnion, Union};
use crate::IntervalSet;

use crate::util::commutative_op_move_impl;

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Union<Self> for Finite<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        match self.clone().merged(rhs.clone()) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.into(), rhs.into()]),
        }
    }
}

impl<T: Domain> RefUnion<Self> for Finite<T> {
    fn ref_union(&self, rhs: &Self) -> Self::Output {
        match self.ref_merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet::new([self.clone().into(), rhs.clone().into()]),
        }
    }
}

impl<T: Domain> Union<Self> for HalfBounded<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        match self.clone().merged(rhs.clone()) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.into(), rhs.into()]),
        }
    }
}
impl<T: Domain> RefUnion<Self> for HalfBounded<T> {}

impl<T: Domain> Union<HalfBounded<T>> for Finite<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: HalfBounded<T>) -> Self::Output {
        match self.clone().merged(rhs.clone()) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.into(), rhs.into()]),
        }
    }
}
impl<T: Domain> RefUnion<HalfBounded<T>> for Finite<T> {}

impl<T: Domain> Union<Finite<T>> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Finite<T>) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Domain> RefUnion<Finite<T>> for BoundCase<T> {
    fn ref_union(&self, rhs: &Finite<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.ref_union(rhs),
            Self::Half(lhs) => lhs.ref_union(rhs),
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

impl<T: Domain> Union<HalfBounded<T>> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: HalfBounded<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.union(rhs),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

impl<T: Domain> RefUnion<HalfBounded<T>> for BoundCase<T> {
    fn ref_union(&self, rhs: &HalfBounded<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.ref_union(rhs),
            Self::Half(lhs) => lhs.ref_union(rhs),
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

impl<T: Domain> Union<Self> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => rhs.union(lhs),
            Self::Finite(lhs) => rhs.union(lhs),
        }
    }
}

impl<T: Domain> RefUnion<Self> for BoundCase<T> {
    fn ref_union(&self, rhs: &Self) -> Self::Output {
        match self {
            Self::Finite(lhs) => rhs.ref_union(lhs),
            Self::Half(lhs) => rhs.ref_union(rhs),
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

commutative_op_move_impl!(Union, union, HalfBounded<T>, Finite<T>, IntervalSet<T>);
commutative_op_move_impl!(Union, union, HalfBounded<T>, BoundCase<T>, IntervalSet<T>);
commutative_op_move_impl!(Union, union, Finite<T>, BoundCase<T>, IntervalSet<T>);

impl<T: Domain> RefUnion<Finite<T>> for HalfBounded<T> {}
impl<T: Domain> RefUnion<BoundCase<T>> for HalfBounded<T> {}
impl<T: Domain> RefUnion<BoundCase<T>> for Finite<T> {
    fn ref_union(&self, rhs: &BoundCase<T>) -> Self::Output {
        rhs.ref_union(self)
    }
}

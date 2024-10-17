use crate::{Domain, IntervalSet, Merged, Union};

use crate::util::commutative_op_impl;

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Union<Self> for Finite<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.clone().into(), rhs.clone().into()]),
        }
    }
}

impl<T: Domain> Union<Self> for HalfBounded<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.clone().into(), rhs.clone().into()]),
        }
    }
}

impl<T: Domain> Union<HalfBounded<T>> for Finite<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &HalfBounded<T>) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet::new(vec![self.clone().into(), rhs.clone().into()]),
        }
    }
}

impl<T: Domain> Union<Finite<T>> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Finite<T>) -> Self::Output {
        // we don't use contiguous for Interval<T> because we disjointness information gets erased
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Domain> Union<HalfBounded<T>> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &HalfBounded<T>) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Domain> Union<Self> for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self {
            Self::Unbounded => Self::Unbounded.into(),
            Self::Half(lhs) => rhs.union(lhs),
            Self::Finite(lhs) => rhs.union(lhs),
        }
    }
}

commutative_op_impl!(Union, union, HalfBounded<T>, Finite<T>, IntervalSet<T>);
commutative_op_impl!(Union, union, HalfBounded<T>, BoundCase<T>, IntervalSet<T>);
commutative_op_impl!(Union, union, Finite<T>, BoundCase<T>, IntervalSet<T>);

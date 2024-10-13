use crate::empty::MaybeEmpty;
use crate::merged::Merged;
use crate::util::commutative_op_impl;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

pub trait Union<Rhs = Self> {
    type Output;

    fn union(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Copy + PartialOrd> Union<Self> for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![self.clone().into(), rhs.clone().into()],
            },
        }
    }
}

impl<T: Copy + PartialOrd> Union<Self> for HalfInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![self.clone().into(), rhs.clone().into()],
            },
        }
    }
}

impl<T: Copy + PartialOrd> Union<HalfInterval<T>> for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &HalfInterval<T>) -> Self::Output {
        match self.merged(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![self.clone().into(), rhs.clone().into()],
            },
        }
    }
}

impl<T: Copy + PartialOrd> Union<FiniteInterval<T>> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        // we don't use contiguous for Interval<T> because we disjointness information gets erased
        match self {
            Self::Infinite => Self::Infinite.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Copy + PartialOrd> Union<HalfInterval<T>> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &HalfInterval<T>) -> Self::Output {
        match self {
            Self::Infinite => Self::Infinite.into(),
            Self::Half(lhs) => lhs.union(rhs),
            Self::Finite(lhs) => lhs.union(rhs),
        }
    }
}

impl<T: Copy + PartialOrd> Union<Self> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self {
            Self::Infinite => Self::Infinite.into(),
            Self::Half(lhs) => rhs.union(lhs),
            Self::Finite(lhs) => rhs.union(lhs),
        }
    }
}

commutative_op_impl!(
    Union,
    union,
    HalfInterval<T>,
    FiniteInterval<T>,
    IntervalSet<T>
);
commutative_op_impl!(Union, union, HalfInterval<T>, Interval<T>, IntervalSet<T>);
commutative_op_impl!(Union, union, FiniteInterval<T>, Interval<T>, IntervalSet<T>);

impl<T: Copy + PartialOrd> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Self) -> Self::Output {
        let intervals = itertools::merge(self.intervals.clone(), rhs.intervals.clone()).collect();

        /*
        let mut intervals = Vec::with_capacity(self.intervals.len() + rhs.intervals.len());
        let left = itertools::put_back(self.intervals.iter());
        let right = itertools::put_back(rhs.intervals.iter());
        ...
        */

        // need to restore the disjoint invariant
        Self::new_unchecked(Self::merge_sorted(intervals))
    }
}

impl<T: Copy + PartialOrd> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Interval<T>) -> Self::Output {
        if rhs.is_empty() {
            // IntervalSet::new does take care of this, but it has to check more things
            return self.clone();
        }

        let mut intervals = self.intervals.clone();
        intervals.push(rhs.clone());
        Self::new(intervals)
    }
}

impl<T: Copy + PartialOrd> Union<FiniteInterval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        self.union(&Interval::<T>::from(rhs.clone()))
    }
}

impl<T: Copy + PartialOrd> Union<HalfInterval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &HalfInterval<T>) -> Self::Output {
        self.union(&Interval::<T>::from(rhs.clone()))
    }
}

commutative_op_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);
commutative_op_impl!(
    Union,
    union,
    FiniteInterval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);
commutative_op_impl!(
    Union,
    union,
    HalfInterval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);

#[cfg(test)]
mod tests {}

use crate::infinite::{Interval, IntervalSet};
use crate::util::commutative_impl;
use crate::{FiniteInterval, HalfInterval};
use crate::contiguous::Contiguous;

pub trait Union<Rhs = Self> {
    type Output;

    fn union(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Copy + PartialOrd> Union<Self> for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.contiguous(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![
                    self.clone().into(), 
                    rhs.clone().into()
                ]
            }
        }
    }
}

impl<T: Copy + PartialOrd> Union<Self> for HalfInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        match self.contiguous(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![
                    self.clone().into(),
                    rhs.clone().into(),
                ]
            }
        }
    }
    
}

impl<T: Copy + PartialOrd> Union<HalfInterval<T>> for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &HalfInterval<T>) -> Self::Output {
        match self.contiguous(rhs) {
            Some(interval) => interval.into(),
            None => IntervalSet {
                intervals: vec![
                    self.clone().into(),
                    rhs.clone().into()
                ]
            }
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

commutative_impl!(Union, union, HalfInterval<T>, FiniteInterval<T>, IntervalSet<T>);
commutative_impl!(Union, union, HalfInterval<T>, Interval<T>, IntervalSet<T>);
commutative_impl!(Union, union, FiniteInterval<T>, Interval<T>, IntervalSet<T>);




////////////


impl<T> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}

impl<T: Copy + PartialOrd + Eq> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Interval<T>) -> Self::Output {
        if *rhs == Interval::Finite(FiniteInterval::Empty) {
            return self.clone()
        }

        let mut merging = rhs.clone();
        let mut intervals = vec![];

        for s_i in self.intervals.iter() {
            if let Some(merged) = merging.contiguous(s_i) {
                merging = merged;
            } else {
                intervals.push(s_i.clone());
            }
        }

        intervals.push(merging);
        
        Self{ intervals }
    }
}

#[cfg(test)]
mod tests {

    
}
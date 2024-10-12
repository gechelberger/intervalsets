use crate::infinite::{Interval, IntervalSet};
use crate::FiniteInterval;
use crate::contiguous::Contiguous;

/// Union iff. lhs and rhs are not disjoint

pub trait Union<Rhs = Self> {
    type Output;

    fn union(&self, rhs: &Rhs) -> Self::Output;
}

impl<T> Union for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}

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

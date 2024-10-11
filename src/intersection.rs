use crate::infinite::{Interval, IntervalSet};
use crate::finite::FiniteInterval;


pub trait Intersection<Rhs = Self> {
    type Output;

    fn intersection(&self, rhs: &Rhs) -> Self::Output;
}

impl<T> Intersection for Interval<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}
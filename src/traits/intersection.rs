use crate::sets::IntervalSet;
use crate::{Domain, Interval};

/// Defines the intersection of two sets.
pub trait Intersection<Rhs = Self> {
    type Output;

    fn intersection(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Domain> Intersection<Self> for Interval<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}

impl<T: Domain> Intersection<Self> for IntervalSet<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}

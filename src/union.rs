use crate::infinite::{Interval, IntervalSet};

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

impl<T> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Interval<T>) -> Self::Output {
        let stack: Vec<Interval<T>> = vec![];
        
        todo!()
    }
}

/*
pub trait UnionMut {

    fn union_mut(&mut self, rhs: &Self);
}

impl<T> UnionMut for IntervalSet<T> {

    fn union_mut(&mut self, rhs: &Self) {
        todo!()
    }
}*/
use std::ops::Add;

use crate::{half::HalfInterval, FiniteInterval, Interval};

pub trait Shifted<T> {
    fn shifted(&self, amount: T) -> Self;
}

pub trait Shiftable<T> = Copy + PartialOrd + Add<Output=T>;

impl<T: Shiftable<T>> Shifted<T> for FiniteInterval<T> {
    fn shifted(&self, amount: T) -> Self {
        self.map_bounds(|left, right| {
            Self::new_unchecked(*left + amount, *right + amount)
        })
    }
}

impl<T: Shiftable<T>> Shifted<T> for HalfInterval<T> {
    fn shifted(&self, amount: T) -> Self {
        Self::new(self.side, self.ival + amount)
    }
}

impl<T: Shiftable<T>> Shifted<T> for Interval<T> {
    fn shifted(&self, amount: T) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => interval.shifted(amount).into(),
            Self::Finite(interval) => interval.shifted(amount).into(),
        }
    }
}
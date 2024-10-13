use std::ops::{Add, Sub};

use num::Zero;

use crate::{half::HalfInterval, ival::Side, FiniteInterval, Interval};

pub trait Padded<T> 
where 
    T: Copy,
    Self: Sized,
{

    fn padded_lr(&self, left: T, right: T) -> Self;

    fn padded(&self, amount: T) -> Self {
        self.padded_lr(amount, amount)
    }
}

trait Paddable<T> = Copy + PartialOrd + Add<Output=T> + Sub<Output=T>;

impl<T: Paddable<T>> Padded<T> for FiniteInterval<T> {

    fn padded_lr(&self, loffset: T, roffset: T) -> Self {
        self.map_bounds(|left, right| {
            Self::new_unchecked(*left - loffset, *right + roffset)
        })
    }

}

impl<T: Paddable<T>> Padded<T> for HalfInterval<T> {

    fn padded_lr(&self, left: T, right: T) -> Self {
        match self.side {
            Side::Left => Self::new(self.side, self.ival - left),
            Side::Right => Self::new(self.side, self.ival + right),
        }
    }
}

impl<T: Paddable<T>> Padded<T> for Interval<T> {

    fn padded_lr(&self, left: T, right: T) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => Self::Half(interval.padded_lr(left, right)),
            Self::Finite(interval) => Self::Finite(interval.padded_lr(left, right))
        }
    }
}
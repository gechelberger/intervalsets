use std::ops::{Add, Sub};

use crate::ival::Side;
use crate::{FiniteInterval, HalfInterval, Interval};

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

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>> Padded<T> for FiniteInterval<T> {
    fn padded_lr(&self, loffset: T, roffset: T) -> Self {
        self.map_bounds(|left, right| Self::new_unchecked(*left - loffset, *right + roffset))
    }
}

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>> Padded<T> for HalfInterval<T> {
    fn padded_lr(&self, left: T, right: T) -> Self {
        match self.side {
            Side::Left => Self::new(self.side, self.ival - left),
            Side::Right => Self::new(self.side, self.ival + right),
        }
    }
}

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T>> Padded<T> for Interval<T> {
    fn padded_lr(&self, left: T, right: T) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => Self::Half(interval.padded_lr(left, right)),
            Self::Finite(interval) => Self::Finite(interval.padded_lr(left, right)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_padded() {
        let interval = Interval::closed(10, 20);

        assert_eq!(interval.padded(10), Interval::closed(0, 30));
    }
}

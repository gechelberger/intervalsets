use std::ops::{Add, Div, Sub};

use num::{FromPrimitive, Zero};

use crate::finite::Interval;
use crate::ival::IVal;

impl<T> Interval<T>
where
    T: Copy + Ord + Add<Output = T> + Sub<Output = T> + Div<Output = T> + FromPrimitive,
{
    fn map_bounds(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> Self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonZero(left, right) => func(left, right),
        }
    }

    pub fn shifted(&self, offset: T) -> Self {
        self.map_bounds(|left, right| Self::new_unchecked(*left + offset, *right + offset))
    }

    pub fn padded(&self, amount: T) -> Self {
        self.padded_lr(amount, amount)
    }

    pub fn padded_lr(&self, left: T, right: T) -> Self {
        self.map_bounds(|iv_left, iv_right| Self::new_unchecked(*iv_left - left, *iv_right + right))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shifted() {
        assert_eq!(Interval::open(0, 10).shifted(10), Interval::open(10, 20));
    }

    #[test]
    fn test_padded() {
        assert_eq!(Interval::open(10, 20).padded(10), Interval::open(0, 30));
    }
}

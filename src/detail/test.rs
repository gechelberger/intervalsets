use core::ops::{Add, Div, Mul, Sub};

use crate::default_countable_impl;
use crate::numeric::{Domain, Zero};

// This provides a non-copy data type for tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CloneInt(pub i32);

impl Add for CloneInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for CloneInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for CloneInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div for CloneInt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Zero for CloneInt {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Domain for CloneInt {
    fn try_adjacent(&self, side: crate::Side) -> Option<Self> {
        match side {
            crate::Side::Left => self.0.checked_sub(1),
            crate::Side::Right => self.0.checked_add(1),
        }
        .map(|inner| Self(inner))
    }
}

default_countable_impl!(CloneInt);

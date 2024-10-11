use std::ops::{Add, Sub};

use num::Zero;

use crate::{half::HalfInterval, FiniteInterval, Interval};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum ISize<T> {
    Finite(T),
    Infinite,
}

/// Required by Zero trait for some reason
impl<T: Add<Output=T>> Add for ISize<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ISize::Infinite, _) => ISize::Infinite,
            (_, ISize::Infinite) => ISize::Infinite,
            (ISize::Finite(lhs), ISize::Finite(rhs)) => {
                ISize::Finite(lhs + rhs)
            }
        }
    }
}

impl<T: Zero + Eq + Add<Output=T>> Zero for ISize<T> {
    fn zero() -> Self {
        Self::Finite(T::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Infinite => false,
            Self::Finite(wrapped) => *wrapped == T::zero()
        }
    }
}


pub trait Sizable {
    type Output: Zero + Eq;

    fn size(&self) -> Self::Output;

    fn is_empty(&self) -> bool {
        self.size() == Self::Output::zero()
    }
}

impl<T: Zero + Sub<Output=T> + Copy + Eq> Sizable for Interval<T> {
    type Output = ISize<T>;

    fn size(&self) -> Self::Output {
        match self {
            Self::Infinite => ISize::Infinite,
            Self::Half(_) => ISize::Infinite,
            Self::Finite(finite) => {
                ISize::Finite(finite.size())
            }
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::Finite(FiniteInterval::Empty)
    }
}

impl<T: Zero + Sub<Output=T> + Copy + Eq> Sizable for FiniteInterval<T> {
    type Output = T;

    fn size(&self) -> Self::Output {
        match self {
            Self::Empty => T::zero(),
            Self::NonZero(left, right) => {
                right.value - left.value
            }
        }
    }

    fn is_empty(&self) -> bool {
        *self == Self::Empty
    }
}

impl<T: Eq + Zero> Sizable for HalfInterval<T> {
    type Output = ISize<T>;

    fn size(&self) -> Self::Output {
        ISize::Infinite
    }
}
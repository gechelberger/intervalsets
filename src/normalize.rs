use num::PrimInt;

use crate::finite::FiniteInterval;
use crate::half::HalfInterval;
use crate::infinite::{Interval, IntervalSet};
use crate::ival::{Side};


pub trait Normalize {
    fn normalized(&self) -> Self;
}

impl<T: PrimInt> Normalize for FiniteInterval<T> {
    fn normalized(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonZero(left, right) => {
                Self::new(
                    left.normalized(Side::Left),
                    right.normalized(Side::Right)
                )
            }
        }
    }
}

impl<T: PrimInt> Normalize for HalfInterval<T> {
    fn normalized(&self) -> Self {
        Self::new(self.side, self.ival.normalized(self.side))
    }
}

impl<T: PrimInt> Normalize for Interval<T> {
    
    fn normalized(&self) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => Self::Half(interval.normalized()),
            Self::Finite(interval) => Self::Finite(interval.normalized()),
        }
    }
}

impl<T: PrimInt> Normalize for IntervalSet<T> {

    fn normalized(&self) -> Self {
        Self {
            intervals: self.intervals.iter()
                .map(|iv| iv.normalized())
                .collect()
        }
    }
}
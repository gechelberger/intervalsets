use num::PrimInt;

use crate::finite::FiniteInterval;
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

impl<T: PrimInt> Normalize for Interval<T> {
    
    fn normalized(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Infinite => Self::Infinite,
            Self::Half((side, ival)) => {
                Self::Half((*side, ival.normalized(*side)))
            },
            Self::Finite((left, right)) => {
                Self::new_finite(
                    left.normalized(Side::Left), 
                    right.normalized(Side::Right)
                )
            }
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
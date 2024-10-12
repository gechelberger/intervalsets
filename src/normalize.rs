use num::PrimInt;

use crate::finite::FiniteInterval;
use crate::half::HalfInterval;
use crate::infinite::{Interval, IntervalSet};
use crate::ival::{Side};
use crate::numeric::Numeric;


pub trait Normalize {
    fn normalized(self) -> Self;
}

impl<T: Numeric + Copy> Normalize for FiniteInterval<T> {
    fn normalized(self) -> Self {

        self.map_bounds(|left, right| {
            Self::new(
                left.normalized(Side::Left),
                right.normalized(Side::Right)
            )
        })
    }
}

impl<T: Numeric + Copy> Normalize for HalfInterval<T> {
    fn normalized(self) -> Self {
        Self::new(self.side, self.ival.normalized(self.side))
    }
}

impl<T: Numeric + Copy> Normalize for Interval<T> {
    
    fn normalized(self) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => Self::Half(interval.normalized()),
            Self::Finite(interval) => Self::Finite(interval.normalized()),
        }
    }
}

impl<T: Numeric + Copy> Normalize for IntervalSet<T> {

    fn normalized(self) -> Self {
        Self {
            intervals: self.intervals.into_iter()
                .map(|iv| iv.normalized())
                .collect()
        }
    }
}
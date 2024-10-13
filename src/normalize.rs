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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_integers() {
        //let interval = Interval::open(50.0, 60.0);
        //let foo = interval.normalized();

        assert_eq!(Interval::open(0, 10).normalized(), Interval::closed(1, 9));
        assert_eq!(Interval::open_closed(0, 10).normalized(), Interval::closed(1, 10));
        assert_eq!(Interval::unbound_open(5 as i8).normalized(), Interval::unbound_closed(4 as i8));
        assert_eq!(Interval::unbound_closed(5 as i8).normalized(), Interval::unbound_closed(5 as i8));
        assert_eq!(Interval::open_unbound(5 as i8).normalized(), Interval::closed_unbound(6 as i8));
        assert_eq!(Interval::closed(0, 10).normalized(), Interval::closed(0, 10));
    }

    #[test]
    fn test_normalized_reals() {
        let interval = Interval::open(0.0, 50.0);
        assert_eq!(interval.clone().normalized(), interval);
    }
}
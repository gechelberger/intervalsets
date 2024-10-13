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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shifted_finite() {
        let interval = Interval::closed(0, 10);

        assert_eq!(interval.shifted(10), Interval::closed(10, 20));
        assert_eq!(interval.shifted(10).shifted(10), Interval::closed(20, 30));
    }

    #[test]
    fn test_shifted_back_finite() {
        let offset: i8 = 55;
        let interval: Interval<i64> = Interval::closed(0, 10);

        let fwd = interval.shifted(offset as i64);
        let rev = fwd.shifted(-offset as i64);
        assert_eq!(interval, rev);
        //assert_eq!(interval.shifted(offset as i64).shifted(-offset as i64), interval);
    }
}
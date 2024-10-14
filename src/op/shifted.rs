use crate::numeric::Numeric;
use crate::{FiniteInterval, HalfInterval, Interval};

pub trait Shifted<T> {
    fn shifted(&self, amount: &T) -> Self;
}

impl<T: Numeric> Shifted<T> for FiniteInterval<T> {
    fn shifted(&self, amount: &T) -> Self {
        self.map_bounds(|left, right| {
            Self::new_unchecked(
                left.clone() + amount.clone(),
                right.clone() + amount.clone(),
            )
        })
    }
}

impl<T: Numeric> Shifted<T> for HalfInterval<T> {
    fn shifted(&self, amount: &T) -> Self {
        Self::new(self.side, self.ival.clone() + amount.clone())
    }
}

impl<T: Numeric> Shifted<T> for Interval<T> {
    fn shifted(&self, amount: &T) -> Self {
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

        assert_eq!(interval.shifted(&10), Interval::closed(10, 20));
        assert_eq!(interval.shifted(&10).shifted(&10), Interval::closed(20, 30));
    }
}

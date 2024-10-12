use crate::ival::Side;
use crate::{FiniteInterval, HalfInterval, Interval};
use crate::infinite::IntervalSet;

pub trait Complement {
    type Output;

    fn complement(&self) -> Self::Output;
}

impl<T: Copy> Complement for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        match self {
            Self::Empty => Interval::Infinite.into(),
            Self::NonZero(left, right) => {
                let intervals: Vec<Interval<T>> = vec![
                    HalfInterval::new(Side::Right, left.flip()).into(),
                    HalfInterval::new(Side::Left, right.flip()).into()
                ];
                IntervalSet { intervals }
            }
        }
    }
}
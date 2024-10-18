use crate::numeric::Domain;
use crate::ops::Complement;
use crate::{Interval, IntervalSet, Side};

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Complement for Finite<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        match self {
            Self::Empty => BoundCase::Unbounded.into(),
            Self::FullyBounded(left, right) => {
                let intervals: Vec<Interval<T>> = vec![
                    HalfBounded::new(Side::Right, left.flip()).into(),
                    HalfBounded::new(Side::Left, right.flip()).into(),
                ];
                IntervalSet::new_unchecked(intervals)
            }
        }
    }
}

impl<T: Domain> Complement for HalfBounded<T> {
    type Output = HalfBounded<T>;

    fn complement(&self) -> Self::Output {
        Self::new(self.side.flip(), self.bound.flip())
    }
}

impl<T: Domain> Complement for BoundCase<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        match self {
            Self::Finite(interval) => interval.complement(),
            Self::Half(interval) => interval.complement().into(),
            Self::Unbounded => Finite::Empty.into(),
        }
    }
}

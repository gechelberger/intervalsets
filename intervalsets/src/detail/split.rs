use crate::numeric::Domain;
use crate::ops::{Contains, RefSplit, Split};
use crate::{Bound, Side};

use super::{BoundCase, Finite, HalfBounded};

fn split_bounds_at<T: Clone>(at: T, closed: Side) -> (Bound<T>, Bound<T>) {
    match closed {
        Side::Left => (Bound::closed(at.clone()), Bound::open(at)),
        Side::Right => (Bound::open(at.clone()), Bound::closed(at)),
    }
}

impl<T: Domain> Split<T> for Finite<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output)
    where
        Self: Sized,
    {
        let contains = self.contains(&at);
        match self {
            Self::FullyBounded(left, right) => {
                if contains {
                    let (l_max, r_min) = split_bounds_at(at, closed);
                    let split_left = Self::new(left, l_max);
                    let split_right = Self::new(r_min, right);
                    (split_left, split_right)
                } else if left.contains(Side::Left, &at) {
                    (Self::FullyBounded(left, right), Self::Empty)
                } else {
                    (Self::Empty, Self::FullyBounded(left, right))
                }
            }
            Self::Empty => (Self::Empty, Self::Empty),
        }
    }
}

impl<T: Domain> Split<T> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        if self.contains(&at) {
            let (l_max, r_min) = split_bounds_at(at, closed);
            match self.side {
                Side::Left => {
                    let left = Finite::new(self.bound, l_max);
                    let right = Self::new(self.side, r_min);
                    (left.into(), right.into())
                }
                Side::Right => {
                    let left = Self::new(self.side, l_max);
                    let right = Finite::new(r_min, self.bound);
                    (left.into(), right.into())
                }
            }
        } else {
            match self.side {
                Side::Left => (Finite::Empty.into(), self.into()),
                Side::Right => (self.into(), Finite::Empty.into()),
            }
        }
    }
}

impl<T: Domain> Split<T> for BoundCase<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        match self {
            Self::Finite(inner) => {
                let (left, right) = inner.split(at, closed);
                (left.into(), right.into())
            }
            Self::Half(inner) => inner.split(at, closed),
            Self::Unbounded => {
                let (l_max, r_min) = split_bounds_at(at, closed);
                (
                    HalfBounded::right(l_max).into(),
                    HalfBounded::left(r_min).into(),
                )
            }
        }
    }
}

impl<T: Domain> RefSplit<T> for Finite<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.clone().split(at, closed)
    }
}

impl<T: Domain> RefSplit<T> for HalfBounded<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.clone().split(at, closed)
    }
}

impl<T: Domain> RefSplit<T> for BoundCase<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.clone().split(at, closed)
    }
}

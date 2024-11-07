use super::{Contains, Split};
use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn split_bounds_at<T: Clone>(at: T, closed: Side) -> (FiniteBound<T>, FiniteBound<T>) {
    match closed {
        Side::Left => (FiniteBound::closed(at.clone()), FiniteBound::open(at)),
        Side::Right => (FiniteBound::open(at.clone()), FiniteBound::closed(at)),
    }
}

impl<T: Domain + Clone> Split<T> for FiniteInterval<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output)
    where
        Self: Sized,
    {
        let contains = self.contains(&at);
        match self {
            Self::Bounded(left, right) => {
                if contains {
                    let (l_max, r_min) = split_bounds_at(at, closed);
                    let split_left = Self::new(left, l_max).unwrap();
                    let split_right = Self::new(r_min, right).unwrap();
                    (split_left, split_right)
                } else if left.contains(Side::Left, &at) {
                    (Self::Bounded(left, right), Self::Empty)
                } else {
                    (Self::Empty, Self::Bounded(left, right))
                }
            }
            Self::Empty => (Self::Empty, Self::Empty),
        }
    }
}

impl<T: Domain + Clone> Split<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        if self.contains(&at) {
            let (l_max, r_min) = split_bounds_at(at, closed);
            match self.side {
                Side::Left => {
                    let left = FiniteInterval::new(self.bound, l_max).unwrap();
                    let right = Self::new(self.side, r_min);
                    (left.into(), right.into())
                }
                Side::Right => {
                    let left = Self::new(self.side, l_max);
                    let right = FiniteInterval::new(r_min, self.bound).unwrap();
                    (left.into(), right.into())
                }
            }
        } else {
            match self.side {
                Side::Left => (FiniteInterval::Empty.into(), self.into()),
                Side::Right => (self.into(), FiniteInterval::Empty.into()),
            }
        }
    }
}

impl<T: Domain + Clone> Split<T> for EnumInterval<T> {
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
                    HalfInterval::right(l_max).into(),
                    HalfInterval::left(r_min).into(),
                )
            }
        }
    }
}

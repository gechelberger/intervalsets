use super::{Contains, Split};
use crate::bound::{FiniteBound, SetBounds, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};

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

impl<T: Domain + Clone> Split<T> for StackSet<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        if self.is_empty() {
            return (Self::empty(), Self::empty());
        }

        let mut left = crate::sets::StackSetStorage::new();
        let mut right = crate::sets::StackSetStorage::new();

        let intervals = self.into_raw();

        // faster than a binary search for small (typical) N.
        for subset in intervals.into_iter() {
            if subset.contains(&at) {
                let (ileft, iright) = subset.split(at.clone(), closed);
                let _ = left.push(ileft);
                let _ = right.push(iright);
            } else if let Some(rbound) = subset.right() {
                if !rbound.contains(Side::Right, &at) {
                    let _ = left.push(subset);
                } else {
                    let _ = right.push(subset);
                }
            } else {
                let _ = right.push(subset);
            }
        }

        unsafe { (Self::new_unchecked(left), Self::new_unchecked(right)) }
    }
}

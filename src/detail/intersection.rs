use crate::numeric::Domain;
use crate::util::commutative_op_impl;
use crate::{Bound, Contains, Intersection, Side};

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Intersection<Self> for Finite<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        self.map(|a_left, a_right| {
            rhs.map(|b_left, b_right| {
                // new() will clean up empty sets where left & right have violated bounds
                Self::new(
                    Bound::max_left(a_left, b_left),
                    Bound::min_right(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Domain> Intersection<HalfBounded<T>> for Finite<T> {
    type Output = Finite<T>;

    fn intersection(&self, rhs: &HalfBounded<T>) -> Self::Output {
        self.map(|left, right| {
            let n_seen = [left, right]
                .into_iter()
                .filter(|limit| rhs.contains(limit.value()))
                .count();

            if n_seen == 2 {
                Self::new(left.clone(), right.clone())
            } else if n_seen == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.bound.clone(), right.clone()),
                    Side::Right => Self::new(left.clone(), rhs.bound.clone()),
                }
            } else {
                Self::Empty
            }
        })
    }
}

impl<T: Domain> Intersection<Self> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.clone().into()
            } else {
                self.clone().into()
            }
        } else {
            // new() handles degenerate cases => Empty
            match self.side {
                Side::Left => Finite::new(self.bound.clone(), rhs.bound.clone()),
                Side::Right => Finite::new(rhs.bound.clone(), self.bound.clone()),
            }
            .into()
        }
    }
}

impl<T: Domain> Intersection<Finite<T>> for BoundCase<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Finite<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs).into(),
            Self::Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<HalfBounded<T>> for BoundCase<T> {
    type Output = Self;

    fn intersection(&self, rhs: &HalfBounded<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs),
            Self::Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<Self> for BoundCase<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        match self {
            Self::Finite(lhs) => rhs.intersection(lhs),
            Self::Half(lhs) => rhs.intersection(lhs),
            Self::Unbounded => rhs.clone(),
        }
    }
}

commutative_op_impl!(
    Intersection,
    intersection,
    HalfBounded<T>,
    Finite<T>,
    Finite<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    Finite<T>,
    BoundCase<T>,
    BoundCase<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    HalfBounded<T>,
    BoundCase<T>,
    BoundCase<T>
);

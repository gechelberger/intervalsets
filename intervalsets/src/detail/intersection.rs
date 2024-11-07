use std::borrow::Cow::*;

use super::{BoundCase, Finite, HalfBounded};
use crate::numeric::Domain;
use crate::ops::{Contains, Intersection, RefIntersection};
use crate::util::commutative_op_move_impl;
use crate::{Bound, Side};

impl<T: Domain> Intersection<Self> for Finite<T> {
    type Output = Self;

    fn intersection(self, rhs: Self) -> Self::Output {
        self.map(|a_left, a_right| {
            rhs.map(|b_left, b_right| {
                // new() will clean up empty sets where left & right have violated bounds
                Self::new(
                    Bound::take_max(Side::Left, a_left, b_left),
                    Bound::take_min(Side::Right, a_right, b_right),
                )
            })
        })
    }
}

impl<T: Domain> RefIntersection<Self> for Finite<T> {
    fn ref_intersection(&self, rhs: &Self) -> <Self as Intersection>::Output {
        self.ref_map(|lhs_min, lhs_max| {
            rhs.ref_map(|rhs_min, rhs_max| {
                Self::new(
                    Bound::max_cow(Side::Left, Borrowed(lhs_min), Borrowed(rhs_min)).into_owned(),
                    Bound::min_cow(Side::Right, Borrowed(lhs_max), Borrowed(rhs_max)).into_owned(),
                )
            })
        })
    }
}

impl<T: Domain> RefIntersection<HalfBounded<T>> for Finite<T> {}
impl<T: Domain> RefIntersection<Finite<T>> for HalfBounded<T> {}
impl<T: Domain> RefIntersection<HalfBounded<T>> for HalfBounded<T> {}

impl<T: Domain> Intersection<HalfBounded<T>> for Finite<T> {
    type Output = Finite<T>;

    fn intersection(self, rhs: HalfBounded<T>) -> Self::Output {
        self.map(|left, right| {
            let n_seen = [&left, &right]
                .into_iter()
                .filter(|bound| rhs.contains(bound.value()))
                .count();

            if n_seen == 2 {
                Self::new(left, right)
            } else if n_seen == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.bound, right),
                    Side::Right => Self::new(left, rhs.bound),
                }
            } else {
                Self::Empty
            }
        })
    }
}

impl<T: Domain> Intersection<Self> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.into()
            } else {
                self.into()
            }
        } else {
            // new() handles degenerate cases => Empty
            match self.side {
                Side::Left => Finite::new(self.bound, rhs.bound),
                Side::Right => Finite::new(rhs.bound, self.bound),
            }
            .into()
        }
    }
}

impl<T: Domain> Intersection<Finite<T>> for BoundCase<T> {
    type Output = Self;

    fn intersection(self, rhs: Finite<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs).into(),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain> RefIntersection<Finite<T>> for BoundCase<T> {
    fn ref_intersection(&self, rhs: &Finite<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.ref_intersection(rhs).into(),
            Self::Half(lhs) => lhs.ref_intersection(rhs).into(),
            Self::Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<HalfBounded<T>> for BoundCase<T> {
    type Output = Self;

    fn intersection(self, rhs: HalfBounded<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain> RefIntersection<HalfBounded<T>> for BoundCase<T> {
    fn ref_intersection(&self, rhs: &HalfBounded<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.ref_intersection(rhs).into(),
            Self::Half(lhs) => lhs.ref_intersection(rhs),
            Self::Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<Self> for BoundCase<T> {
    type Output = Self;

    fn intersection(self, rhs: Self) -> Self::Output {
        match self {
            Self::Finite(lhs) => rhs.intersection(lhs),
            Self::Half(lhs) => rhs.intersection(lhs),
            Self::Unbounded => rhs.clone(),
        }
    }
}

impl<T: Domain> RefIntersection<Self> for BoundCase<T> {
    fn ref_intersection(&self, rhs: &Self) -> <Self as Intersection>::Output {
        match self {
            Self::Finite(lhs) => rhs.ref_intersection(lhs),
            Self::Half(lhs) => rhs.ref_intersection(lhs),
            Self::Unbounded => rhs.clone(),
        }
    }
}

commutative_op_move_impl!(
    Intersection,
    intersection,
    HalfBounded<T>,
    Finite<T>,
    Finite<T>
);
commutative_op_move_impl!(
    Intersection,
    intersection,
    Finite<T>,
    BoundCase<T>,
    BoundCase<T>
);
commutative_op_move_impl!(
    Intersection,
    intersection,
    HalfBounded<T>,
    BoundCase<T>,
    BoundCase<T>
);

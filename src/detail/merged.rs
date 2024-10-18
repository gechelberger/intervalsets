use crate::numeric::Domain;
use crate::ops::{Contains, Intersects, Merged};
use crate::{Bound, MaybeEmpty, Side};

use super::adjacent::Adjacent;
use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Merged<Self> for Finite<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) && !self.is_adjacent_to(rhs) {
            if self.is_empty() {
                return Some(rhs.clone());
            } else if rhs.is_empty() {
                return Some(self.clone());
            } else {
                return None;
            }
        }

        Some(self.map(|a_left, a_right| {
            rhs.map(|b_left, b_right| {
                Finite::FullyBounded(
                    Bound::min_left(a_left, b_left),
                    Bound::max_right(a_right, b_right),
                )
            })
        }))
    }
}

impl<T: Domain> Merged<Self> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            if self.contains(rhs.bound.value())
                || rhs.contains(self.bound.value())
                || self.is_adjacent_to(rhs)
            {
                Some(BoundCase::Unbounded)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Domain> Merged<Finite<T>> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &Finite<T>) -> Option<Self::Output> {
        match rhs {
            Finite::Empty => Some(self.clone().into()),
            Finite::FullyBounded(left, right) => {
                let n_seen = [left, right]
                    .into_iter()
                    .filter(|bound| self.contains(bound.value()))
                    .count();

                if n_seen == 2 {
                    Some(self.clone().into())
                } else if n_seen == 0 && !self.is_adjacent_to(rhs) {
                    None
                } else {
                    match self.side {
                        Side::Left => Some(HalfBounded::new(self.side, left.clone()).into()),
                        Side::Right => Some(HalfBounded::new(self.side, right.clone()).into()),
                    }
                }
            }
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for Finite<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &HalfBounded<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged<Finite<T>> for BoundCase<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &Finite<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.merged(rhs).map(|itv| itv.into()),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for BoundCase<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &HalfBounded<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<Self> for BoundCase<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs),
            Self::Half(lhs) => rhs.merged(lhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<BoundCase<T>> for Finite<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &BoundCase<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged<BoundCase<T>> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn merged(&self, rhs: &BoundCase<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

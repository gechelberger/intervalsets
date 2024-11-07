use std::borrow::Cow::*;

use super::adjacent::Adjacent;
use super::{BoundCase, Finite, HalfBounded};
use crate::numeric::Domain;
use crate::ops::{Contains, Intersects, Merged};
use crate::traits::merged::RefMerged;
use crate::{Bound, MaybeEmpty, Side};

impl<T: Domain> Merged<Self> for Finite<T> {
    type Output = Self;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        if self.is_disjoint_from(&rhs) && !self.is_adjacent_to(&rhs) {
            if self.is_empty() {
                return Some(rhs);
            } else if rhs.is_empty() {
                return Some(self);
            } else {
                return None;
            }
        }

        self.map(|a_left, a_right| {
            rhs.map(|b_left, b_right| {
                Finite::FullyBounded(
                    Bound::take_min(Side::Left, a_left, b_left),
                    Bound::take_max(Side::Right, a_right, b_right),
                )
            })
        })
        .into()
    }
}

impl<T: Domain> RefMerged<Self> for Finite<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) && !self.is_adjacent_to(rhs) {
            if self.is_empty() {
                return Some(rhs.clone());
            } else if rhs.is_empty() {
                return Some(self.clone());
            } else {
                return None;
            }
        }

        self.ref_map(|a_left, a_right| {
            rhs.ref_map(|b_left, b_right| {
                Finite::FullyBounded(
                    Bound::min_cow(Side::Left, Borrowed(a_left), Borrowed(b_left)).into_owned(),
                    Bound::max_cow(Side::Right, Borrowed(a_right), Borrowed(b_right)).into_owned(),
                )
            })
        })
        .into()
    }
}

impl<T: Domain> Merged<Self> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.into())
            } else {
                Some(rhs.into())
            }
        } else {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            if self.contains(rhs.bound.value())
                || rhs.contains(self.bound.value())
                || self.is_adjacent_to(&rhs)
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

    fn merged(self, rhs: Finite<T>) -> Option<Self::Output> {
        let n_seen = rhs.ref_map_or(2, |left, right| {
            [left, right]
                .into_iter()
                .filter(|b| self.contains(b.value()))
                .count()
        });

        if n_seen == 2 {
            Some(self.into())
        } else if n_seen == 0 && !self.is_adjacent_to(&rhs) {
            None
        } else {
            rhs.map_or(None, |left, right| {
                let merged = match self.side {
                    Side::Left => HalfBounded::new(self.side, left),
                    Side::Right => HalfBounded::new(self.side, right),
                };

                Some(merged.into())
            })
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for Finite<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: HalfBounded<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged<Finite<T>> for BoundCase<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: Finite<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.merged(rhs).map(|itv| itv.into()),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> RefMerged<Finite<T>> for BoundCase<T> {
    fn ref_merged(&self, rhs: &Finite<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => lhs.ref_merged(rhs).map(|x| x.into()),
            Self::Half(lhs) => lhs.ref_merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for BoundCase<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: HalfBounded<T>) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<Self> for BoundCase<T> {
    type Output = Self;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.merged(lhs),
            Self::Half(lhs) => rhs.merged(lhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> RefMerged<Self> for BoundCase<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Finite(lhs) => rhs.ref_merged(lhs),
            Self::Half(lhs) => rhs.ref_merged(lhs),
            Self::Unbounded => Some(Self::Unbounded),
        }
    }
}

impl<T: Domain> Merged<BoundCase<T>> for Finite<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: BoundCase<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> RefMerged<BoundCase<T>> for Finite<T> {
    fn ref_merged(&self, rhs: &BoundCase<T>) -> Option<Self::Output> {
        rhs.ref_merged(self)
    }
}

impl<T: Domain> Merged<BoundCase<T>> for HalfBounded<T> {
    type Output = BoundCase<T>;

    fn merged(self, rhs: BoundCase<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> RefMerged<HalfBounded<T>> for Finite<T> {}
impl<T: Domain> RefMerged<Finite<T>> for HalfBounded<T> {}
impl<T: Domain> RefMerged<HalfBounded<T>> for HalfBounded<T> {}
impl<T: Domain> RefMerged<HalfBounded<T>> for BoundCase<T> {}

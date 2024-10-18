use super::{BoundCase, Finite, HalfBounded};
use crate::numeric::Domain;
use crate::{Bound, Contains, Intersects, MaybeEmpty, Merged, Side};

impl<T: Domain> Merged<Self> for Finite<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) {
            // TODO Adjacency?
            // For T in Real, (0, 1) U [1, 2] => (0, 2]
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
            // unfortunately we have to check from both sides to catch the
            // case where left and right values are the same but open & closed
            if self.contains(rhs.bound.value()) && rhs.contains(self.bound.value()) {
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

                match n_seen {
                    2 => Some(self.clone().into()),
                    1 => match self.side {
                        Side::Left => Some(HalfBounded::new(self.side, left.clone()).into()),
                        Side::Right => Some(HalfBounded::new(self.side, right.clone()).into()),
                    },
                    _ => None, // disjoint
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

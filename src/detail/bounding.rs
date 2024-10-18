use crate::numeric::Domain;
use crate::traits::bounding::Bounding;
use crate::{Bound, Side};

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> Bounding<T> for Finite<T> {
    fn bound(&self, side: crate::Side) -> Option<&Bound<T>> {
        match self {
            Self::Empty => None,
            Self::FullyBounded(left, right) => match side {
                Side::Left => Some(left),
                Side::Right => Some(right),
            },
        }
    }
}

impl<T: Domain> Bounding<T> for HalfBounded<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>> {
        if self.side == side {
            Some(&self.bound)
        } else {
            None
        }
    }
}

impl<T: Domain> Bounding<T> for BoundCase<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>> {
        match self {
            Self::Finite(inner) => inner.bound(side),
            Self::Half(inner) => inner.bound(side),
            Self::Unbounded => None,
        }
    }
}

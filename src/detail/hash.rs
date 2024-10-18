use std::hash::Hash;

use super::{BoundCase, Finite, HalfBounded};

impl<T: Hash> Hash for Finite<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Empty => "Finite::Empty".hash(state),
            Self::FullyBounded(left, right) => {
                "Finite::FullyBounded".hash(state);
                left.hash(state);
                right.hash(state);
            }
        }
    }
}

impl<T: Hash> Hash for HalfBounded<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "HalfBounded".hash(state);
        self.side.hash(state);
        self.bound.hash(state);
    }
}

impl<T: Hash> Hash for BoundCase<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Finite(inner) => {
                "BoundCase::Finite".hash(state);
                inner.hash(state);
            }
            Self::Half(inner) => {
                "BoundCase::Half".hash(state);
                inner.hash(state);
            }
            Self::Unbounded => {
                "BoundCase::Unbounded".hash(state);
            }
        }
    }
}

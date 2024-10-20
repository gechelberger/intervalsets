mod adjacent;
mod bounding;
mod complement;
mod contains;
mod empty;
mod from;
mod hash;
mod intersection;
mod intersects;
mod merged;
mod union;

mod measure;

mod partial_ord;

use crate::bound::{Bound, Side};
use crate::numeric::Domain;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Finite<T> {
    Empty,
    FullyBounded(Bound<T>, Bound<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HalfBounded<T> {
    side: Side,
    bound: Bound<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BoundCase<T> {
    Finite(Finite<T>),
    Half(HalfBounded<T>),
    Unbounded,
}

impl<T: Domain> Finite<T> {
    pub fn new(left: Bound<T>, right: Bound<T>) -> Self {
        if left.value() > right.value() {
            Self::Empty
        } else if left.value() == right.value() {
            if left.is_open() || right.is_open() {
                Self::Empty
            } else {
                // singleton set
                Self::new_unchecked(left, right)
            }
        } else {
            Self::new_unchecked(left.normalized(Side::Left), right.normalized(Side::Right))
        }
    }

    pub fn new_unchecked(left: Bound<T>, right: Bound<T>) -> Self {
        Self::FullyBounded(left, right)
    }

    pub fn map(&self, func: impl FnOnce(&Bound<T>, &Bound<T>) -> Self) -> Self {
        self.map_or(Self::Empty, func)
    }

    pub fn map_or<U>(&self, default: U, func: impl FnOnce(&Bound<T>, &Bound<T>) -> U) -> U {
        match self {
            Self::FullyBounded(left, right) => func(left, right),
            Self::Empty => default,
        }
    }

    pub fn map_or_else<F, D, U>(&self, default: D, func: F) -> U
    where
        D: FnOnce() -> U,
        F: FnOnce(&Bound<T>, &Bound<T>) -> U,
    {
        match self {
            Self::FullyBounded(left, right) => func(left, right),
            Self::Empty => default(),
        }
    }
}

impl<T: Domain> HalfBounded<T> {
    pub fn new(side: Side, bound: Bound<T>) -> Self {
        Self { side, bound }
    }
}

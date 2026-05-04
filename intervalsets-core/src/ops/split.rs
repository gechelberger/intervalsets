use super::Contains;
use crate::bound::{FiniteBound, Side};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Split a Set into two disjoint subsets, fully covering the original.
///
/// `at` provides the new bounds where the set should be split.
///
/// # Contract
///
/// Tier 3 (`try_*` + panicking sugar).
/// [`try_split`](Self::try_split) returns `Err(Self::Error)` on
/// logical violation (typically: a non-comparable user-supplied
/// `at`, e.g. NaN); it never panics. [`split`](Self::split) is the
/// panicking unwrap of `try_split`. See [`crate::ops`] for the full
/// tier model.
///
/// # Example
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// let (left, right) = x.split(5, Side::Left);
/// assert_eq!(left, FiniteInterval::closed(0, 5));
/// assert_eq!(right, FiniteInterval::closed(6, 10));
/// ```
pub trait Split<T>: Sized {
    /// The type of `Set` to create when split.
    type Output;
    type Error: core::error::Error;

    /// Creates two disjoint subsets with elements partitioned by `at`.
    ///
    /// # Panics
    ///
    /// Panic if `at` is not comparable.
    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.try_split(at, closed).unwrap()
    }

    fn try_split(self, at: T, closed: Side)
        -> Result<(Self::Output, Self::Output), Self::Error>;
}

fn split_bounds_at<T: Clone>(at: T, closed: Side) -> (FiniteBound<T>, FiniteBound<T>) {
    match closed {
        Side::Left => (FiniteBound::closed(at.clone()), FiniteBound::open(at)),
        Side::Right => (FiniteBound::open(at.clone()), FiniteBound::closed(at)),
    }
}

impl<T: Element + Clone> Split<T> for FiniteInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn try_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        let Some((min, max)) = self.into_raw() else {
            return Ok((Self::empty(), Self::empty()));
        };

        if !min.try_contains(Side::Left, &at)? {
            let repacked = Self::new_assume_valid(min, max);
            return Ok((Self::empty(), repacked));
        }

        if !max.try_contains(Side::Right, &at)? {
            let repacked = Self::new_assume_valid(min, max);
            return Ok((repacked, Self::empty()));
        }

        let (lhs_max, rhs_min) = split_bounds_at(at, closed);
        // try_new_or_empty: splitting at a boundary value with the
        // boundary kind on one side produces a degenerate empty side
        // (e.g. [min, min) when closed = Right and at = min). That's
        // the correct answer, not an error.
        let split_left = Self::try_new_or_empty(min, lhs_max)?;
        let split_right = Self::try_new_or_empty(rhs_min, max)?;
        Ok((split_left, split_right))
    }
}

impl<T: Element + Clone> Split<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn try_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        if !self.contains(&at) {
            return match self.side() {
                Side::Left => Ok((Self::Output::empty(), self.into())),
                Side::Right => Ok((self.into(), Self::Output::empty())),
            };
        }

        let (lhs_max, rhs_min) = split_bounds_at(at, closed);
        let (side, bound) = self.into_raw();
        // try_new_or_empty: a split exactly at the half-bounded interval's
        // own boundary produces a degenerate empty side, which is the
        // correct answer (not an error).
        match side {
            Side::Left => {
                let left = FiniteInterval::try_new_or_empty(bound, lhs_max)?;
                let right = HalfInterval::try_new(side, rhs_min)?;
                Ok((left.into(), right.into()))
            }
            Side::Right => {
                let left = HalfInterval::try_new(side, lhs_max)?;
                let right = FiniteInterval::try_new_or_empty(rhs_min, bound)?;
                Ok((left.into(), right.into()))
            }
        }
    }
}

impl<T: Element + Clone> Split<T> for EnumInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn try_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        match self {
            Self::Finite(inner) => inner
                .try_split(at, closed)
                .map(|(l, r)| (l.into(), r.into())),
            Self::Half(inner) => inner.try_split(at, closed),
            Self::Unbounded => {
                let (lhs_max, rhs_min) = split_bounds_at(at, closed);
                let left = HalfInterval::try_new(Side::Right, lhs_max)?;
                let right = HalfInterval::try_new(Side::Left, rhs_min)?;
                Ok((left.into(), right.into()))
            }
        }
    }

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


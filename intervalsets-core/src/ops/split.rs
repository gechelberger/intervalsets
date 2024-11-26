use num_traits::Zero;

use super::Contains;
use crate::bound::{FiniteBound, Side};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Split a Set into two disjoint subsets, fully covering the original.
///
/// `at` provides the new bounds where the set should be split.
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
        self.strict_split(at, closed).unwrap()
    }

    fn strict_split(self, at: T, closed: Side)
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

    fn strict_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        let Some((min, max)) = self.into_raw() else {
            return Ok((Self::empty(), Self::empty()));
        };

        if !min.strict_contains(Side::Left, &at)? {
            let repacked = unsafe { Self::new_unchecked(min, max) };
            return Ok((Self::empty(), repacked));
        }

        if !max.strict_contains(Side::Right, &at)? {
            let repacked = unsafe { Self::new_unchecked(min, max) };
            return Ok((repacked, Self::empty()));
        }

        let (lhs_max, rhs_min) = split_bounds_at(at, closed);
        let split_left = Self::new_strict(min, lhs_max)?;
        let split_right = Self::new_strict(rhs_min, max)?;
        Ok((split_left, split_right))
    }
}

impl<T: Element + Clone + Zero> Split<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn strict_split(
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
        match side {
            Side::Left => {
                let left = FiniteInterval::new_strict(bound, lhs_max)?;
                let right = HalfInterval::new_strict(side, rhs_min)?;
                Ok((left.into(), right.into()))
            }
            Side::Right => {
                let left = HalfInterval::new_strict(side, lhs_max)?;
                let right = FiniteInterval::new_strict(rhs_min, bound)?;
                Ok((left.into(), right.into()))
            }
        }
    }
}

impl<T: Element + Clone + Zero> Split<T> for EnumInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn strict_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        match self {
            Self::Finite(inner) => inner
                .strict_split(at, closed)
                .map(|(l, r)| (l.into(), r.into())),
            Self::Half(inner) => inner.strict_split(at, closed),
            Self::Unbounded => {
                let (lhs_max, rhs_min) = split_bounds_at(at, closed);
                let left = HalfInterval::new_strict(Side::Right, lhs_max)?;
                let right = HalfInterval::new_strict(Side::Left, rhs_min)?;
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

/*
impl<T: Element + Clone> Split<T> for StackSet<T> {
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
*/

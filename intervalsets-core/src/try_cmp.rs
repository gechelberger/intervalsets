//! Fallible comparison for `PartialOrd` types.
//!
//! [`TryCmp`] provides fallible compare/min/max operations for types
//! that implement [`PartialOrd`] but not [`Ord`]. This allows users to
//! work with subsets of a type that do have a **total order** even if
//! the type as a whole does not, and fails gracefully if elements
//! outside of that totally ordered subset are used.
//!
//! These methods are infallible for types implementing [`Ord`].

use core::cmp::Ordering::{self, *};

use crate::error::TotalOrderError;

/// Fallible comparison built on top of [`PartialOrd`].
///
/// Blanket-implemented for every `T: PartialOrd`, so any partially
/// ordered type gets the methods automatically.
pub trait TryCmp: PartialOrd + Sized {
    /// Returns the [`Ordering`] of `self` and `rhs`, or [`TotalOrderError`]
    /// if the two values are incomparable.
    #[inline]
    fn try_cmp(&self, rhs: &Self) -> Result<Ordering, TotalOrderError> {
        self.partial_cmp(rhs).ok_or(TotalOrderError)
    }

    /// Returns `(min, max)` of `self` and `rhs`, or [`TotalOrderError`]
    /// if the two values are incomparable.
    #[inline]
    fn try_min_max(self, rhs: Self) -> Result<(Self, Self), TotalOrderError> {
        match self.try_cmp(&rhs)? {
            Less | Equal => Ok((self, rhs)),
            Greater => Ok((rhs, self)),
        }
    }

    /// Returns the lesser of `self` and `rhs`, or [`TotalOrderError`]
    /// if the two values are incomparable.
    #[inline]
    fn try_min(self, rhs: Self) -> Result<Self, TotalOrderError> {
        self.try_min_max(rhs).map(|(min, _)| min)
    }

    /// Returns the greater of `self` and `rhs`, or [`TotalOrderError`]
    /// if the two values are incomparable.
    #[inline]
    fn try_max(self, rhs: Self) -> Result<Self, TotalOrderError> {
        self.try_min_max(rhs).map(|(_, max)| max)
    }
}

impl<T: PartialOrd> TryCmp for T {}

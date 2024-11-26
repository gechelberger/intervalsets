//! Fallible min/max for `PartialOrd` types.
//!
//! `TryMin` and `TryMax` provide fallible min/max
//! operations for types that implement `PartialOrd`
//! but not `Ord`. This allows users to work with
//! subsets of a type that do have a **total order**
//! even if the type as a whole does not, and fails
//! gracefully if elements outside of that totally
//! ordered subset are used.
//!
//! These traits should be infallible for types implementing [`Ord`].

use core::cmp::Ordering::{self, *};

use crate::error::TotalOrderError;

/// Returns the ordering of lhs and rhs or TotalOrderError
pub trait TryCmp {
    fn try_cmp(&self, rhs: &Self) -> Result<Ordering, TotalOrderError>;
}

/// Return the min item *iff* self and rhs are ordered.
pub trait TryMin: Sized {
    #[allow(missing_docs)]
    fn try_min(self, rhs: Self) -> Result<Self, TotalOrderError>;
}

/// Return the max item *iff* self and rhs are ordered.
pub trait TryMax: Sized {
    #[allow(missing_docs)]
    fn try_max(self, rhs: Self) -> Result<Self, TotalOrderError>;
}

impl<T: PartialOrd> TryCmp for T {
    #[inline]
    fn try_cmp(&self, rhs: &Self) -> Result<Ordering, TotalOrderError> {
        self.partial_cmp(rhs).ok_or(TotalOrderError)
    }
}

impl<T: PartialOrd> TryMin for T {
    fn try_min(self, rhs: Self) -> Result<Self, TotalOrderError> {
        match self.try_cmp(&rhs)? {
            Less | Equal => Ok(self),
            Greater => Ok(rhs),
        }
    }
}

impl<T: PartialOrd> TryMax for T {
    fn try_max(self, rhs: Self) -> Result<Self, TotalOrderError> {
        match self.try_cmp(&rhs)? {
            Greater | Equal => Ok(self),
            Less => Ok(rhs),
        }
    }
}

pub fn try_ord_pair<A: PartialOrd>(lhs: A, rhs: A) -> Result<[A; 2], TotalOrderError> {
    match lhs.try_cmp(&rhs)? {
        Less | Equal => Ok([lhs, rhs]),
        Greater => Ok([rhs, lhs]),
    }
}

pub fn try_ord_tuple<A: PartialOrd>(lhs: A, rhs: A) -> Result<(A, A), TotalOrderError> {
    match lhs.try_cmp(&rhs)? {
        Less | Equal => Ok((lhs, rhs)),
        Greater => Ok((rhs, lhs)),
    }
}

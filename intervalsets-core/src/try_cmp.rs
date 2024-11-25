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

use core::cmp::Ordering::*;

use crate::error::TotalOrderError;

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

impl<T: PartialOrd> TryMin for T {
    fn try_min(self, rhs: Self) -> Result<Self, TotalOrderError> {
        let order = self.partial_cmp(&rhs).ok_or(TotalOrderError)?;

        match order {
            Less | Equal => Ok(self),
            Greater => Ok(rhs),
        }
    }
}

impl<T: PartialOrd> TryMax for T {
    fn try_max(self, rhs: Self) -> Result<Self, TotalOrderError> {
        let order = self.partial_cmp(&rhs).ok_or(TotalOrderError)?;
        match order {
            Greater | Equal => Ok(self),
            Less => Ok(rhs),
        }
    }
}

pub fn try_ord_pair<A: PartialOrd>(lhs: A, rhs: A) -> Result<[A; 2], TotalOrderError> {
    let order = lhs.partial_cmp(&rhs).ok_or(TotalOrderError)?;

    match order {
        Less | Equal => Ok([lhs, rhs]),
        Greater => Ok([rhs, lhs]),
    }
}

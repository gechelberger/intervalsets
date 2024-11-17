//! TryMin and TryMax provide fallible min/max
//! operations for types that implement PartialOrd
//! but not Ord. This allows users to work with
//! subsets of a type that do have a **total order**
//! even if the type as a whole does not, and fail
//! gracefully if elements outside of that totally
//! ordered subset are accidentally used.
use core::cmp::Ordering::*;

/// Return the min item *iff* items are ordered.
pub trait TryMin: Sized {
    fn try_min(self, rhs: Self) -> Option<Self>;
}

/// Return the max item *iff* items are ordered.
pub trait TryMax: Sized {
    fn try_max(self, rhs: Self) -> Option<Self>;
}

impl<T: PartialOrd> TryMin for T {
    fn try_min(self, rhs: Self) -> Option<Self> {
        match self.partial_cmp(&rhs)? {
            Less | Equal => Some(self),
            Greater => Some(rhs),
        }
    }
}

impl<T: PartialOrd> TryMax for T {
    fn try_max(self, rhs: Self) -> Option<Self> {
        match self.partial_cmp(&rhs)? {
            Greater | Equal => Some(self),
            Less => Some(rhs),
        }
    }
}

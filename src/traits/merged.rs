use crate::{Domain, Interval};

/// Defines the union of two intervals if contiguous.
/// 
/// Disjoint sets return `None` unless one is the `Empty` Set,
/// in which case the other input Set is the result (which could
/// be `Empty`).
pub trait Merged<Rhs = Self> {
    type Output;

    fn merged(&self, rhs: &Rhs) -> Option<Self::Output>;
}

impl<T: Domain> Merged<Self> for Interval<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        self.0.merged(&rhs.0).map(|v| v.into())
    }
}

use crate::{Domain, Interval};

/// Union for two intervals that are contiguous.
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

//! Set-level midpoint accessor on [`Interval`].
//!
//! [`IntervalSet::midpoint`] is intentionally not exposed: the
//! midpoint of a disjoint union is ill-defined. Users who want a
//! per-component midpoint should iterate the components.
//!
//! [`IntervalSet`]: crate::IntervalSet

use intervalsets_core::error::MathError;
use intervalsets_core::numeric::Midpoint;

use crate::error::Error;
use crate::Interval;

impl<T> Interval<T>
where
    T: Clone + Midpoint,
    <T as Midpoint>::Error: Into<MathError>,
{
    /// Midpoint of the inhabited fully-bounded interval. Empty,
    /// half-bounded, and unbounded intervals all return
    /// `Err(Error::Math(MathError::Domain))`.
    pub fn midpoint(&self) -> Result<T, Error> {
        self.0.midpoint().map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, UnboundedFactory};

    #[test]
    fn finite_integer() {
        let x = Interval::closed(0_i32, 10);
        assert_eq!(x.midpoint(), Ok(5));
    }

    #[test]
    fn empty_returns_domain() {
        let x: Interval<i32> = Interval::empty();
        assert_eq!(x.midpoint(), Err(Error::Math(MathError::Domain)));
    }

    #[test]
    fn unbounded_returns_domain() {
        let x: Interval<i32> = Interval::unbounded();
        assert_eq!(x.midpoint(), Err(Error::Math(MathError::Domain)));
    }
}

//! Set-level [`Midpoint`] trait impls for [`Interval`] and
//! [`IntervalSet`] — hull-midpoint semantics.
//!
//! `set.midpoint()` returns the midpoint of the convex hull,
//! `(inf(S) + sup(S)) / 2`. For a single connected interval this is
//! the interval's midpoint; for an [`IntervalSet`] with multiple
//! pieces this is the hull midpoint and **may lie in a gap** between
//! components. Callers wanting a point guaranteed to lie inside some
//! component should reach for `Centroid` (planned).
//!
//! Empty / unbounded / half-bounded inputs return
//! `Err(Error::Math(MathError::Domain))`.

use intervalsets_core::error::MathError;
use intervalsets_core::numeric::{Element, Midpointable};
pub use intervalsets_core::ops::Midpoint;

use crate::error::Error;
use crate::{Interval, IntervalSet};

impl<T> Midpoint<T> for Interval<T>
where
    T: Clone + Midpointable,
    <T as Midpointable>::Error: Into<MathError>,
{
    type Error = Error;

    /// Hull midpoint. Empty / half-bounded / unbounded ⇒
    /// `Err(Error::Math(MathError::Domain))`. Single connected
    /// interval ⇒ midpoint of its bounds.
    fn midpoint(&self) -> Result<T, Error> {
        intervalsets_core::ops::Midpoint::midpoint(&self.0).map_err(Error::from)
    }
}

impl<T> Midpoint<T> for IntervalSet<T>
where
    T: Element + Clone + Midpointable,
    <T as Midpointable>::Error: Into<MathError>,
{
    type Error = Error;

    /// Hull midpoint. Empty or unbounded hull ⇒
    /// `Err(Error::Math(MathError::Domain))`. For a multi-piece set
    /// the result is the midpoint of the convex hull and may lie in a
    /// gap between components — see the trait docs.
    fn midpoint(&self) -> Result<T, Error> {
        <Interval<T> as Midpoint<T>>::midpoint(&self.hull())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

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

    #[test]
    fn interval_set_single_piece_delegates() {
        let s = IntervalSet::new([Interval::closed(0_i32, 10)]);
        assert_eq!(s.midpoint(), Ok(5));
    }

    #[test]
    fn interval_set_multi_piece_returns_hull_midpoint() {
        // [0, 10] ∪ [100, 110] → hull [0, 110] → midpoint 55
        // (in the gap, not a member of the set — documented).
        let s = IntervalSet::new([Interval::closed(0_i32, 10), Interval::closed(100, 110)]);
        assert_eq!(s.midpoint(), Ok(55));
    }

    #[test]
    fn interval_set_empty_returns_domain() {
        let s: IntervalSet<i32> = IntervalSet::empty();
        assert_eq!(s.midpoint(), Err(Error::Math(MathError::Domain)));
    }

    #[test]
    fn interval_set_unbounded_piece_returns_domain() {
        let s = IntervalSet::new([Interval::unbound_closed(0_i32), Interval::closed(10, 20)]);
        assert_eq!(s.midpoint(), Err(Error::Math(MathError::Domain)));
    }
}

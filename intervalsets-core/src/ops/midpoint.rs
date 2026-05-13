//! Set-level [`Midpoint`] trait — hull-midpoint semantics.
//!
//! `set.midpoint()` returns the midpoint of the set's convex hull,
//! `(inf(S) + sup(S)) / 2`. For a single connected interval this is
//! the interval's midpoint; for multi-piece sets the result may lie in
//! a gap. Callers wanting a point guaranteed to lie in some component
//! should reach for `Centroid` (planned).
//!
//! `Err(MathError::Domain)` covers shapes that have no defined hull
//! midpoint — empty, half-bounded, or unbounded sets. For an inhabited
//! fully-bounded hull, the result is whatever `T::midpoint` (per
//! [`Midpointable`]) produces; this is `Ok` for every in-tree library
//! `T` except `Decimal`, which can return [`MathError::Range`] on
//! rounding overflow at the bounds of its range.

use crate::error::MathError;
use crate::numeric::{Element, Midpointable};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Hull midpoint of a set.
///
/// `midpoint` returns `(inf(S) + sup(S)) / 2` for an inhabited
/// fully-bounded set. For a single connected interval this is the
/// interval's midpoint; for multi-piece sets ([`MaybeDisjoint`],
/// `IntervalSet` in the wrapper crate) it is the midpoint of the
/// convex hull and **may lie in a gap** — i.e. not be a member of `S`.
/// Callers wanting an in-component result should use `Centroid`
/// (planned).
///
/// # Errors
///
/// `Err(MathError::Domain)` for shapes without a defined hull midpoint:
/// empty sets, half-bounded intervals, fully unbounded intervals, or
/// multi-piece sets whose hull is unbounded. Otherwise the error
/// surface is whatever the underlying [`Midpointable`] impl produces
/// (lifted through [`Self::Error`]).
///
/// # Tier
///
/// Tier 3a (`try_*`, total, panic-free) by contract. The method is
/// named `midpoint` rather than `try_midpoint`: there is no
/// panicking sibling, because hull-midpoint failure (empty /
/// unbounded shapes) is a routine outcome rather than an exceptional
/// one, and there is no `Extent`-style sentinel that could absorb the
/// "no midpoint" answer the way `Extent::Infinite` absorbs unbounded
/// width.
pub trait Midpoint<T> {
    type Error: core::error::Error;
    fn midpoint(&self) -> Result<T, Self::Error>;
}

impl<T> Midpoint<T> for FiniteInterval<T>
where
    T: Clone + Midpointable,
    <T as Midpointable>::Error: Into<MathError>,
{
    type Error = MathError;

    /// Midpoint of the interval. `Empty` ⇒ `Err(MathError::Domain)`.
    /// For an inhabited interval, delegates to `T::midpoint`; this is
    /// `Ok` for every in-tree library `T` except `Decimal` at extreme
    /// values.
    fn midpoint(&self) -> Result<T, MathError> {
        match self.view_raw() {
            None => Err(MathError::Domain),
            Some((l, r)) => l
                .value()
                .clone()
                .midpoint(r.value().clone())
                .map_err(Into::into),
        }
    }
}

impl<T> Midpoint<T> for HalfInterval<T> {
    type Error = MathError;

    /// A half-bounded interval has an unbounded hull and therefore no
    /// midpoint — always `Err(MathError::Domain)`.
    fn midpoint(&self) -> Result<T, MathError> {
        Err(MathError::Domain)
    }
}

impl<T> Midpoint<T> for EnumInterval<T>
where
    T: Clone + Midpointable,
    <T as Midpointable>::Error: Into<MathError>,
{
    type Error = MathError;

    /// Midpoint of the inhabited fully-bounded variant. `Half` /
    /// `Unbounded` / empty `Finite` ⇒ `Err(MathError::Domain)`.
    fn midpoint(&self) -> Result<T, MathError> {
        match self {
            Self::Finite(inner) => <FiniteInterval<T> as Midpoint<T>>::midpoint(inner),
            Self::Half(_) | Self::Unbounded => Err(MathError::Domain),
        }
    }
}

impl<T> Midpoint<T> for MaybeDisjoint<T>
where
    T: Element + Clone + Midpointable,
    <T as Midpointable>::Error: Into<MathError>,
{
    type Error = MathError;

    /// Hull midpoint. `Connected(iv)` delegates to `iv.midpoint()`;
    /// `Disjoint(a, b)` computes the midpoint of the convex hull
    /// spanning `a`'s left bound to `b`'s right bound — which **may
    /// lie in the gap between the pieces** (i.e. not be a member of
    /// the set). Callers needing a point guaranteed to lie in some
    /// component should use the planned `Centroid` trait.
    ///
    /// `Err(MathError::Domain)` when the hull is unbounded (any piece
    /// is half/unbounded) or when the set is empty.
    fn midpoint(&self) -> Result<T, MathError> {
        <EnumInterval<T> as Midpoint<T>>::midpoint(&self.hull())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::FiniteBound;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};

    #[test]
    fn finite_empty_returns_domain() {
        let x: FiniteInterval<i32> = FiniteInterval::empty();
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }

    #[test]
    fn finite_integer_midpoint() {
        let x = FiniteInterval::closed(0_i32, 10);
        assert_eq!(x.midpoint(), Ok(5));
    }

    #[test]
    fn finite_full_range_no_overflow() {
        // std's i32::midpoint is overflow-safe; the value must lie in
        // [MIN, MAX] (i.e. not panic / wrap to garbage).
        let x = FiniteInterval::closed(i32::MIN, i32::MAX);
        let m = x.midpoint().unwrap();
        assert_eq!(m, i32::midpoint(i32::MIN, i32::MAX));
    }

    #[test]
    fn finite_float_midpoint() {
        let x = FiniteInterval::closed(0.0_f64, 10.0);
        assert_eq!(x.midpoint(), Ok(5.0));
    }

    #[test]
    fn half_bounded_returns_domain() {
        let x = HalfInterval::<i32>::left(FiniteBound::closed(0));
        assert_eq!(Midpoint::<i32>::midpoint(&x), Err(MathError::Domain));

        let x = HalfInterval::<i32>::right(FiniteBound::closed(0));
        assert_eq!(Midpoint::<i32>::midpoint(&x), Err(MathError::Domain));
    }

    #[test]
    fn enum_finite_midpoint() {
        let x = EnumInterval::closed(0_i32, 10);
        assert_eq!(x.midpoint(), Ok(5));
    }

    #[test]
    fn enum_unbounded_returns_domain() {
        let x: EnumInterval<i32> = EnumInterval::Unbounded;
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }

    // ===== MaybeDisjoint =====

    #[test]
    fn md_connected_delegates() {
        let x = MaybeDisjoint::from_interval(EnumInterval::closed(0_i32, 10));
        assert_eq!(x.midpoint(), Ok(5));
    }

    #[test]
    fn md_empty_returns_domain() {
        let x = MaybeDisjoint::<i32>::empty();
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }

    #[test]
    fn md_connected_half_returns_domain() {
        // Connected(half_interval) delegates and gets Domain from the inner.
        let half = EnumInterval::closed_unbound(0_i32);
        let x = MaybeDisjoint::from_interval(half);
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }

    #[test]
    fn md_disjoint_returns_hull_midpoint() {
        // `Disjoint([0, 5], [10, 15])` has hull `[0, 15]` whose
        // midpoint is `7`. The point lies in the gap `(5, 10)` — i.e.
        // is NOT a member of the set — which is documented behavior
        // for hull-midpoint semantics. Callers wanting an in-component
        // result need `Centroid` (planned).
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        assert_eq!(x.midpoint(), Ok(7));
    }

    #[test]
    fn md_disjoint_with_unbounded_piece_returns_domain() {
        // Hull is unbounded → Domain.
        let x = MaybeDisjoint::from_pair(
            EnumInterval::unbound_closed(0_i32),
            EnumInterval::closed(10, 15),
        );
        assert_eq!(x.midpoint(), Err(MathError::Domain));

        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i32, 5),
            EnumInterval::closed_unbound(10),
        );
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }
}

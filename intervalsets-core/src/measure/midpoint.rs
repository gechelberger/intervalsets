//! Set-level midpoint accessors.
//!
//! Inherent methods on [`FiniteInterval`], [`HalfInterval`], and
//! [`EnumInterval`] that compute the midpoint of an inhabited
//! fully-bounded interval. The "fallibility boundary" is the set
//! shape: empty / half-bounded / unbounded all return
//! `Err(MathError::Domain)` because the midpoint is undefined for
//! those inputs. For a fully-bounded inhabited interval, the result is
//! whatever `T::midpoint` produces — for in-tree library types this is
//! `Infallible` (collapsed via `From<Infallible> for MathError`),
//! except for `Decimal` which can return `MathError::Range` on
//! rounding overflow at the bounds of its range.

use crate::error::MathError;
use crate::numeric::Midpoint;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

impl<T> FiniteInterval<T>
where
    T: Clone + Midpoint,
    <T as Midpoint>::Error: Into<MathError>,
{
    /// Midpoint of the interval. `Empty` ⇒ `Err(MathError::Domain)`.
    /// For an inhabited fully-bounded interval, returns whatever
    /// `T::midpoint` produces; this is `Ok` for every in-tree library
    /// `T` except `Decimal` at extreme values.
    pub fn midpoint(&self) -> Result<T, MathError> {
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

impl<T> HalfInterval<T> {
    /// A half-bounded interval has no midpoint — always
    /// `Err(MathError::Domain)`.
    pub fn midpoint(&self) -> Result<T, MathError> {
        Err(MathError::Domain)
    }
}

impl<T> EnumInterval<T>
where
    T: Clone + Midpoint,
    <T as Midpoint>::Error: Into<MathError>,
{
    /// Midpoint of the inhabited fully-bounded variant. `Half` /
    /// `Unbounded` / empty `Finite` ⇒ `Err(MathError::Domain)`.
    pub fn midpoint(&self) -> Result<T, MathError> {
        match self {
            Self::Finite(inner) => inner.midpoint(),
            Self::Half(_) | Self::Unbounded => Err(MathError::Domain),
        }
    }
}

impl<T> MaybeDisjoint<T>
where
    T: Clone + Midpoint,
    <T as Midpoint>::Error: Into<MathError>,
{
    /// Midpoint of the inhabited fully-bounded *single connected*
    /// variant. `Connected(iv)` delegates to `iv.midpoint()`;
    /// `Disjoint(_, _)` ⇒ `Err(MathError::Domain)`.
    ///
    /// # Why `Disjoint` is `Domain`, not the hull midpoint
    ///
    /// `Midpoint` is a connected-interval concept: `(left + right) / 2`.
    /// For a multi-component set, the hull midpoint may lie in a gap
    /// (not in the set), and a measure-weighted "center of mass" is a
    /// different operation with different semantics. Rather than pick
    /// either silently, this method returns `Err(MathError::Domain)`
    /// and forces the caller to make an explicit choice:
    ///
    /// - For the hull midpoint: `md.into_hull().midpoint()`.
    /// - For the measure-weighted center: see the planned `Centroid`
    ///   trait (separate operation with its own contract).
    pub fn midpoint(&self) -> Result<T, MathError> {
        match self {
            Self::Connected(iv) => iv.midpoint(),
            Self::Disjoint(_, _) => Err(MathError::Domain),
        }
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
        assert_eq!(x.midpoint(), Err(MathError::Domain));

        let x = HalfInterval::<i32>::right(FiniteBound::closed(0));
        assert_eq!(x.midpoint(), Err(MathError::Domain));
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
    fn md_disjoint_returns_domain() {
        // Disjoint is always Domain — even when the hull midpoint would
        // be well-defined. Callers wanting the hull midpoint must spell
        // it as `md.into_hull().midpoint()`.
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        assert_eq!(x.midpoint(), Err(MathError::Domain));
    }

    #[test]
    fn md_hull_midpoint_path_works_when_caller_opts_in() {
        // Documenting the workaround: callers who want the hull midpoint
        // can compose `into_hull().midpoint()`. For this `Disjoint`, the
        // hull `[0, 15]` has midpoint 7 (i32::midpoint(0, 15) = 7).
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        assert_eq!(x.into_hull().midpoint(), Ok(7));
    }
}

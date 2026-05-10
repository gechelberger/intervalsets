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
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::FiniteBound;
    use crate::factory::FiniteFactory;

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
}

use rust_decimal::Decimal;

use crate::continuous_domain_impl;
use crate::error::MathError;
use crate::measure::Widthable;
use crate::numeric::Midpoint;
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

continuous_domain_impl!(Decimal);

impl Widthable for Decimal {
    type Output = Decimal;

    fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
        // Decimal has bounded precision (≈ ±7.92e28); the diff can
        // overflow at extremes (e.g. `Decimal::MAX - Decimal::MIN`).
        right.checked_sub(*left)
    }
}

impl Midpoint for Decimal {
    type Error = MathError;

    /// Computes the midpoint as `(self / 2) + (other / 2)`, halving
    /// each input first so the leading addition cannot overflow the
    /// `Decimal` range (≈ ±7.92e28). Result is exact for typical
    /// inputs and within 0.5 ULP of the true midpoint at extreme
    /// precision (where each half rounds).
    ///
    /// # Errors
    ///
    /// Returns [`MathError::Range`] when rounding each half pushes the
    /// addition out of range — for example `Decimal::MAX.midpoint(MAX)`,
    /// where `MAX / 2` rounds up by 0.5 and the two halves sum to
    /// `MAX + 1`. Symmetric on the negative side for `MIN`.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        (self / Decimal::TWO)
            .checked_add(other / Decimal::TWO)
            .ok_or(MathError::Range)
    }
}

// === Value-level TryOp impls (E3) ===
//
// `Decimal` has bounded precision (≈ ±7.92e28); all four ops can
// overflow → `MathError::Range`. `Decimal::checked_div` returns `None`
// for both `/0` and overflow, so we pre-check zero to surface `/0` as
// `MathError::Domain`.

impl TryAdd for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_add(rhs).ok_or(MathError::Range)
    }
}

impl TrySub for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_sub(rhs).ok_or(MathError::Range)
    }
}

impl TryMul for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_mul(rhs).ok_or(MathError::Range)
    }
}

impl TryDiv for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        if rhs.is_zero() {
            return Err(MathError::Domain);
        }
        self.checked_div(rhs).ok_or(MathError::Range)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_decimal_interval() {
        let interval = EnumInterval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));

        assert_eq!(interval.width().finite(), Decimal::new(798, 2));
    }

    #[test]
    fn test_midpoint_decimal() {
        // integer-valued, exact
        let mid = Decimal::new(10, 0).midpoint(Decimal::new(20, 0)).unwrap();
        assert_eq!(mid, Decimal::new(15, 0));

        // mixed-scale: 2.02 and 10.0 -> 6.01
        let mid = Decimal::new(202, 2).midpoint(Decimal::new(100, 1)).unwrap();
        assert_eq!(mid, Decimal::new(601, 2));

        // negative midpoint -- Decimal is continuous, so the result is
        // the exact mean (-3.5), not truncation toward zero.
        let mid = Decimal::new(-7, 0).midpoint(Decimal::new(0, 0)).unwrap();
        assert_eq!(mid, Decimal::new(-35, 1));

        // commutativity
        let a = Decimal::new(1_000_001, 3);
        let b = Decimal::new(-3, 0);
        assert_eq!(a.midpoint(b).unwrap(), b.midpoint(a).unwrap());

        // Half-then-add accepts pairs near MAX whose direct sum would
        // overflow. Result may round within 0.5 ULP of the true midpoint;
        // we assert boundedness rather than the exact rounded value.
        let high = Decimal::MAX;
        let lower = Decimal::MAX - Decimal::ONE;
        let mid = high.midpoint(lower).unwrap();
        assert!(mid >= lower && mid <= high);

        // The extreme pair -- both at MAX -- still fails because each
        // half rounds up, pushing the sum above MAX.
        assert_eq!(Decimal::MAX.midpoint(Decimal::MAX), Err(MathError::Range));
        assert_eq!(Decimal::MIN.midpoint(Decimal::MIN), Err(MathError::Range));
    }

    #[test]
    fn test_try_ops_decimal() {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = Decimal::new(10, 0);
        let b = Decimal::new(3, 0);

        assert_eq!(a.try_add(b).unwrap(), Decimal::new(13, 0));
        assert_eq!(a.try_sub(b).unwrap(), Decimal::new(7, 0));
        assert_eq!(a.try_mul(b).unwrap(), Decimal::new(30, 0));
        assert_eq!(
            Decimal::new(10, 0).try_div(Decimal::new(2, 0)).unwrap(),
            Decimal::new(5, 0)
        );

        // Domain on /0
        assert_eq!(a.try_div(Decimal::ZERO), Err(MathError::Domain));

        // Range on overflow (MAX + MAX)
        assert_eq!(Decimal::MAX.try_add(Decimal::MAX), Err(MathError::Range));
        assert_eq!(Decimal::MAX.try_mul(Decimal::MAX), Err(MathError::Range));
    }
}

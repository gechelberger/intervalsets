use rust_decimal::Decimal;

use crate::continuous_domain_impl;
use crate::error::MidpointError;
use crate::numeric::Midpoint;

continuous_domain_impl!(Decimal);

impl Midpoint for Decimal {
    type Error = MidpointError;

    /// Computes the midpoint as `(self / 2) + (other / 2)`, halving
    /// each input first so the leading addition cannot overflow the
    /// `Decimal` range (≈ ±7.92e28). Result is exact for typical
    /// inputs and within 0.5 ULP of the true midpoint at extreme
    /// precision (where each half rounds).
    ///
    /// # Errors
    ///
    /// Returns [`MidpointError`] when rounding each half pushes the
    /// addition out of range — for example `Decimal::MAX.midpoint(MAX)`,
    /// where `MAX / 2` rounds up by 0.5 and the two halves sum to
    /// `MAX + 1`. Symmetric on the negative side for `MIN`.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        (self / Decimal::TWO)
            .checked_add(other / Decimal::TWO)
            .ok_or(MidpointError)
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
        assert!(Decimal::MAX.midpoint(Decimal::MAX).is_err());
        assert!(Decimal::MIN.midpoint(Decimal::MIN).is_err());
    }
}

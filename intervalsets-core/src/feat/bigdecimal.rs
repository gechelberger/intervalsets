use core::convert::Infallible;
use core::ops::Add;

use bigdecimal::{BigDecimal, Zero};

use crate::continuous_domain_impl;
use crate::error::MathError;
use crate::numeric::Midpoint;
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

continuous_domain_impl!(BigDecimal);

impl Midpoint for BigDecimal {
    type Error = core::convert::Infallible;

    /// Infallible: `BigDecimal` is arbitrary precision, so the midpoint
    /// of any pair is always representable.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok(self.add(other).half())
    }
}

// === Value-level TryOp impls (E3) ===
//
// `BigDecimal` is arbitrary precision: add/sub/mul cannot fail.
// Division panics on `/0`; we pre-check and surface that as
// `MathError::Domain`. (Non-terminating expansions like `1/3` are
// handled by `bigdecimal` internally via its precision setting and do
// not raise an error.)

impl TryAdd for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self + rhs)
    }
}

impl TrySub for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self - rhs)
    }
}

impl TryMul for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self * rhs)
    }
}

impl TryDiv for BigDecimal {
    type Output = BigDecimal;
    type Error = MathError;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        if rhs.is_zero() {
            return Err(MathError::Domain);
        }
        Ok(self / rhs)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_big_decimal() -> Result<(), bigdecimal::ParseBigDecimalError> {
        let x = EnumInterval::closed(BigDecimal::from_str("0.0")?, BigDecimal::from_str("10.0")?);
        assert_eq!(x.width().finite(), BigDecimal::from_str("10")?);
        Ok(())
    }

    #[test]
    fn test_try_ops_bigdecimal() -> Result<(), bigdecimal::ParseBigDecimalError> {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = BigDecimal::from_str("10.0")?;
        let b = BigDecimal::from_str("4.0")?;
        let two_five = BigDecimal::from_str("2.5")?;

        assert_eq!(
            a.clone().try_add(b.clone()).unwrap(),
            BigDecimal::from_str("14.0")?
        );
        assert_eq!(
            a.clone().try_sub(b.clone()).unwrap(),
            BigDecimal::from_str("6.0")?
        );
        assert_eq!(
            a.clone().try_mul(b.clone()).unwrap(),
            BigDecimal::from_str("40.00")?
        );
        assert_eq!(a.clone().try_div(b.clone()).unwrap(), two_five);

        // /0
        assert_eq!(
            a.try_div(BigDecimal::from_str("0")?),
            Err(crate::error::MathError::Domain)
        );

        Ok(())
    }
}

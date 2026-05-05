use bigdecimal::BigDecimal;

use crate::continuous_domain_impl;

continuous_domain_impl!(BigDecimal);

use crate::numeric::Midpoint;
use core::ops::Add;

impl Midpoint for BigDecimal {
    type Error = core::convert::Infallible;

    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok(self.add(other).half())
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
}

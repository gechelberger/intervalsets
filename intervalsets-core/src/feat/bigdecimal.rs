use bigdecimal::BigDecimal;

use crate::continuous_domain_impl;
use crate::numeric::Domain;

continuous_domain_impl!(BigDecimal);

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

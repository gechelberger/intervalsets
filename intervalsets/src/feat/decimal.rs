use rust_decimal::Decimal;

use crate::continuous_domain_impl;

continuous_domain_impl!(Decimal);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_decimal_interval() {
        let interval = Interval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));

        assert_eq!(interval.width().finite(), Decimal::new(798, 2));
    }
}

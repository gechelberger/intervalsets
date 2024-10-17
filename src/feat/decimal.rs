use crate::continuous_domain_impl;
use crate::adapt_num_traits_zero_impl;
use crate::Side;
use crate::numeric::Domain;
use rust_decimal::Decimal;

continuous_domain_impl!(Decimal);

adapt_num_traits_zero_impl!(Decimal);

#[cfg(test)]
mod test {
    use super::*;
    use crate::Contains;
    use crate::Interval;
    use crate::measure::Width;

    #[test]
    fn test_decimal_interval() {
        let interval = Interval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));

        assert_eq!(interval.width().finite(), Decimal::new(798, 2));
    }
}
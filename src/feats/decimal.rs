use crate::continuous_domain_impl;
use crate::ival::Side;
use crate::numeric::Domain;
use rust_decimal::Decimal;

continuous_domain_impl!(Decimal);

#[cfg(test)]
mod test {
    use super::*;
    use crate::pred::contains::Contains;
    use crate::Interval;

    #[test]
    fn test_decimal_interval() {
        let interval = Interval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));
    }
}

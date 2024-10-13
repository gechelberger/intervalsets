use rust_decimal::Decimal;
use crate::numeric::{Numeric, NumericSet};

impl Numeric for Decimal {
    fn numeric_set() -> NumericSet {
        NumericSet::Real
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Interval;
    use crate::contains::Contains;

    #[test]
    fn test_decimal_interval() {
        let interval = Interval::open(
            Decimal::new(202, 2),
            Decimal::new(100, 1)
        );

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));
    }
}
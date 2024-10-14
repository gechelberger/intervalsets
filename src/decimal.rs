use crate::numeric::{Numeric, NumericSet};
use rust_decimal::Decimal;

impl Numeric for Decimal {
    fn numeric_set() -> NumericSet {
        NumericSet::Real
    }

    fn try_finite_add(self, rhs: Self) -> Option<Self> {
        self.checked_add(rhs)
    }

    fn try_finite_sub(self, rhs: Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

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

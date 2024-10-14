use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub};

use crate::numeric::{Numeric, NumericSet};

impl Numeric for BigInt {
    fn numeric_set() -> NumericSet {
        NumericSet::Integer
    }

    fn try_finite_add(&self, rhs: &Self) -> Option<Self> {
        self.checked_add(rhs)
    }

    fn try_finite_sub(&self, rhs: &Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

impl Numeric for BigUint {
    fn numeric_set() -> NumericSet {
        NumericSet::Natural
    }

    fn try_finite_add(&self, rhs: &Self) -> Option<Self> {
        self.checked_add(rhs)
    }

    fn try_finite_sub(&self, rhs: &Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use crate::sizeable::{ISize, Sizable};
    use crate::Interval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a.clone(), b);
        assert_eq!(interval.size(), ISize::Finite(a));
    }
}

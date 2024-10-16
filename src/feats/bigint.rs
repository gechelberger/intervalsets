use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One};

use crate::ival::Side;
use crate::numeric::Domain;
use crate::Countable;

impl Domain for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigInt::one()),
            Side::Right => self.checked_add(&BigInt::one()),
        }
    }
}

impl Countable for BigInt {}

impl Domain for BigUint {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigUint::one()),
            Side::Right => self.checked_add(&BigUint::one()),
        }
    }
}

impl Countable for BigUint {}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use crate::measure::width::Width;
    use crate::Interval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }
}

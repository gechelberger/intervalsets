use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One};

use crate::ival::Side;
use crate::numeric::Domain;

impl Domain for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigInt::one()),
            Side::Right => self.checked_add(&BigInt::one()),
        }
    }
}

impl Domain for BigUint {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigUint::one()),
            Side::Right => self.checked_add(&BigUint::one()),
        }
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use crate::{ISize, Interval, Sizable};

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a.clone(), b);
        assert_eq!(interval.size(), ISize::Finite(a));
    }
}

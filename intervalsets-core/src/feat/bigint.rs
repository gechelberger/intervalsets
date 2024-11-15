use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One};

use crate::bound::Side::{self, *};
use crate::default_countable_impl;
use crate::numeric::Domain;

impl Domain for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigInt::one()),
            Right => self.checked_add(&BigInt::one()),
        }
    }
}

default_countable_impl!(BigInt);

impl Domain for BigUint {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigUint::one()),
            Right => self.checked_add(&BigUint::one()),
        }
    }
}

default_countable_impl!(BigUint);

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use crate::factory::Factory;
    use crate::measure::Width;
    use crate::EnumInterval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = EnumInterval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }
}

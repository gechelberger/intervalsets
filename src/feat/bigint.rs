use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One};

use crate::numeric::Domain;
use crate::{adapt_num_traits_zero_impl, default_countable_impl, Side};

impl Domain for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigInt::one()),
            Side::Right => self.checked_add(&BigInt::one()),
        }
    }
}

default_countable_impl!(BigInt);

impl Domain for BigUint {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Left => self.checked_sub(&BigUint::one()),
            Side::Right => self.checked_add(&BigUint::one()),
        }
    }
}

default_countable_impl!(BigUint);

adapt_num_traits_zero_impl!(BigInt, BigUint);

#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;

    use crate::factory::Factory;
    use crate::measure::Width;
    use crate::Interval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }
}

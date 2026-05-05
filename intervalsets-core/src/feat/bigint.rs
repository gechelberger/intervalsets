use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One};

use crate::bound::Side::{self, *};
use crate::default_countable_impl;
use crate::numeric::Element;

impl Element for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigInt::one()),
            Right => self.checked_add(&BigInt::one()),
        }
    }
}

default_countable_impl!(BigInt);

impl Element for BigUint {
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
    use num_bigint::{BigInt, ToBigInt};

    use crate::factory::FiniteFactory;
    use crate::measure::{Count, Width};
    use crate::EnumInterval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = EnumInterval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }

    #[test]
    fn test_count_exceeds_primitive_range() {
        // 2^200 + 1 elements - well beyond what any primitive integer
        // (including u128) can represent. Demonstrates that BigInt's
        // arbitrary-precision Countable::Output can carry counts that
        // would overflow the primitive widening path.
        let lower = BigInt::from(0u8);
        let upper: BigInt = BigInt::from(1u8) << 200;
        let interval = EnumInterval::closed(lower, upper.clone());

        let expected = upper + BigInt::from(1u8);
        assert!(expected > BigInt::from(usize::MAX));
        assert!(expected > BigInt::from(u128::MAX));
        assert_eq!(interval.count().finite(), expected);
    }
}

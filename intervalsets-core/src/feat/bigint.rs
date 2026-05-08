use core::convert::Infallible;

use num_bigint::{BigInt, BigUint};
use num_traits::{CheckedAdd, CheckedSub, One, Zero};

use crate::bound::Side::{self, *};
use crate::default_countable_impl;
use crate::error::MathError;
use crate::numeric::{Element, Midpoint};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

impl Element for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigInt::one()),
            Right => self.checked_add(&BigInt::one()),
        }
    }
}

default_countable_impl!(BigInt);

impl Midpoint for BigInt {
    type Error = core::convert::Infallible;

    /// Infallible: `BigInt` is arbitrary precision, so the midpoint of
    /// any pair is always representable. `/2` truncates toward zero,
    /// matching std's signed-primitive midpoint semantics.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok((self + other) / 2)
    }
}

impl Element for BigUint {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigUint::one()),
            Right => self.checked_add(&BigUint::one()),
        }
    }
}

default_countable_impl!(BigUint);

impl Midpoint for BigUint {
    type Error = core::convert::Infallible;

    /// Infallible: `BigUint` is arbitrary precision, so the midpoint
    /// of any pair is always representable.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok((self + other) >> 1)
    }
}

// === Value-level TryOp impls (E3) ===
//
// `BigInt` and `BigUint` are arbitrary precision: `Add`, `Mul`, and
// (for BigInt) `Sub` cannot fail, so they expose `Error = Infallible`.
// `BigUint::Sub` *can* fail when `rhs > self` (no negative repr), and
// `Div` panics on `/0` for both — those return `MathError`.

impl TryAdd for BigInt {
    type Output = BigInt;
    type Error = Infallible;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self + rhs)
    }
}

impl TrySub for BigInt {
    type Output = BigInt;
    type Error = Infallible;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self - rhs)
    }
}

impl TryMul for BigInt {
    type Output = BigInt;
    type Error = Infallible;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self * rhs)
    }
}

impl TryDiv for BigInt {
    type Output = BigInt;
    type Error = MathError;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        if rhs.is_zero() {
            return Err(MathError::Domain);
        }
        Ok(self / rhs)
    }
}

impl TryAdd for BigUint {
    type Output = BigUint;
    type Error = Infallible;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self + rhs)
    }
}

impl TrySub for BigUint {
    type Output = BigUint;
    type Error = MathError;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        // BigUint has no negative representation; `rhs > self` underflows.
        self.checked_sub(&rhs).ok_or(MathError::Range)
    }
}

impl TryMul for BigUint {
    type Output = BigUint;
    type Error = Infallible;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self * rhs)
    }
}

impl TryDiv for BigUint {
    type Output = BigUint;
    type Error = MathError;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        if rhs.is_zero() {
            return Err(MathError::Domain);
        }
        Ok(self / rhs)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::{BigInt, BigUint, ToBigInt};

    use crate::factory::FiniteFactory;
    use crate::measure::{Count, Width};
    use crate::numeric::Midpoint;
    use crate::EnumInterval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = EnumInterval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }

    #[test]
    fn test_midpoint_bigint() {
        let mid = BigInt::from(10).midpoint(BigInt::from(20)).unwrap();
        assert_eq!(mid, BigInt::from(15));

        // truncation toward zero (matches std signed-primitive midpoint)
        let mid = BigInt::from(-7).midpoint(BigInt::from(0)).unwrap();
        assert_eq!(mid, BigInt::from(-3));

        let mid = BigInt::from(0).midpoint(BigInt::from(-7)).unwrap();
        assert_eq!(mid, BigInt::from(-3));

        // commutativity
        let a = BigInt::from(1_000_001);
        let b = BigInt::from(-3);
        assert_eq!(
            a.clone().midpoint(b.clone()).unwrap(),
            b.midpoint(a).unwrap()
        );
    }

    #[test]
    fn test_midpoint_biguint() {
        let mid = BigUint::from(10u32).midpoint(BigUint::from(20u32)).unwrap();
        assert_eq!(mid, BigUint::from(15u32));

        // odd sum rounds toward 0 (vacuous for unsigned, == floor)
        let mid = BigUint::from(7u32).midpoint(BigUint::from(0u32)).unwrap();
        assert_eq!(mid, BigUint::from(3u32));

        // exceeds u128 to confirm we're not silently widening
        let huge: BigUint = BigUint::from(1u32) << 200;
        let mid = huge.clone().midpoint(BigUint::from(0u32)).unwrap();
        assert_eq!(mid, huge >> 1);
    }

    #[test]
    fn test_try_ops_bigint() {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = BigInt::from(10);
        let b = BigInt::from(3);
        assert_eq!(a.clone().try_add(b.clone()).unwrap(), BigInt::from(13));
        assert_eq!(a.clone().try_sub(b.clone()).unwrap(), BigInt::from(7));
        assert_eq!(a.clone().try_mul(b.clone()).unwrap(), BigInt::from(30));
        assert_eq!(a.clone().try_div(b.clone()).unwrap(), BigInt::from(3));

        // /0 is the only failure path
        assert_eq!(
            a.try_div(BigInt::from(0)),
            Err(crate::error::MathError::Domain)
        );
    }

    #[test]
    fn test_try_ops_biguint() {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = BigUint::from(10u32);
        let b = BigUint::from(3u32);
        assert_eq!(a.clone().try_add(b.clone()).unwrap(), BigUint::from(13u32));
        assert_eq!(a.clone().try_sub(b.clone()).unwrap(), BigUint::from(7u32));
        assert_eq!(a.clone().try_mul(b.clone()).unwrap(), BigUint::from(30u32));
        assert_eq!(a.clone().try_div(b.clone()).unwrap(), BigUint::from(3u32));

        // BigUint underflow (would-be-negative)
        assert_eq!(
            BigUint::from(3u32).try_sub(BigUint::from(10u32)),
            Err(crate::error::MathError::Range)
        );
        // /0
        assert_eq!(
            a.try_div(BigUint::from(0u32)),
            Err(crate::error::MathError::Domain)
        );
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

use core::convert::Infallible;

use num_bigint::{BigInt, BigUint};
use num_traits::{Bounded, CheckedAdd, CheckedSub, FromPrimitive, NumCast, One, Signed, Zero};

use crate::bound::Side::{self, *};
use crate::cast::{CastElement, LossyCastElement, TryCastElement};
use crate::error::MathError;
use crate::numeric::{Element, Midpointable};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};
use crate::{default_countable_impl, default_width_impl};

impl Element for BigInt {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Left => self.checked_sub(&BigInt::one()),
            Right => self.checked_add(&BigInt::one()),
        }
    }
}

default_countable_impl!(BigInt);
default_width_impl!(BigInt);

impl Midpointable for BigInt {
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
default_width_impl!(BigUint);

impl Midpointable for BigUint {
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

// === Cast support ===
//
// `BigInt` and `BigUint` are arbitrary-precision integers, so they
// accept every primitive numeric value (lossless `Cast`) and only fail
// when going the other direction (overflow).
//
// `Cast` (Tier 1, infallible) is provided for:
//
// - `{i*, u*, bool} → BigInt` (all primitives via upstream `From`).
// - `{u*, bool} → BigUint` (unsigned primitives via upstream `From`).
//   Signed primitives cannot `Cast` to `BigUint` (negative values
//   wouldn't fit); use `TryCast` instead.
// - `BigUint → BigInt` (upstream `From<BigUint> for BigInt`).
// - `BigInt → BigInt`, `BigUint → BigUint` (reflexive).
//
// Floats cannot `Cast` to either `BigInt` or `BigUint`: a finite f64
// can have a fractional part, and truncating that is not "infallible
// lossless conversion". Use `TryCast` (truncates fractional, errors on
// NaN/INF) or convert manually for explicit rounding semantics.
//
// `TryCast` covers every direction including floats and integer
// narrowing. `LossyCast` covers `BigInt/BigUint → primitive` with
// saturation on overflow. `LossyCast` targeting `BigInt`/`BigUint` is
// intentionally not provided — they're unbounded (no `Bounded` impl,
// no saturation extremum).

// ---------- CastElement<BigInt> for primitive sources ----------

macro_rules! cast_element_to_bigint {
    ($($T:ty),+ $(,)?) => {
        $(
            impl CastElement<BigInt> for $T {
                #[inline]
                fn cast_element(self) -> BigInt {
                    BigInt::from(self)
                }
            }
        )+
    };
}

cast_element_to_bigint!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool);

impl CastElement<BigInt> for BigUint {
    #[inline]
    fn cast_element(self) -> BigInt {
        BigInt::from(self)
    }
}

impl CastElement<BigInt> for BigInt {
    #[inline]
    fn cast_element(self) -> BigInt {
        self
    }
}

// ---------- CastElement<BigUint> for unsigned primitive sources ----------

macro_rules! cast_element_to_biguint {
    ($($T:ty),+ $(,)?) => {
        $(
            impl CastElement<BigUint> for $T {
                #[inline]
                fn cast_element(self) -> BigUint {
                    BigUint::from(self)
                }
            }
        )+
    };
}

cast_element_to_biguint!(u8, u16, u32, u64, u128, usize, bool);

impl CastElement<BigUint> for BigUint {
    #[inline]
    fn cast_element(self) -> BigUint {
        self
    }
}

// ---------- TryCastElement: source → BigInt ----------

macro_rules! try_cast_element_to_bigint_from_int {
    ($($T:ty),+ $(,)?) => {
        $(
            impl TryCastElement<BigInt> for $T {
                #[inline]
                fn try_cast_element(self) -> Option<BigInt> {
                    Some(BigInt::from(self))
                }
            }
        )+
    };
}

try_cast_element_to_bigint_from_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool
);

// Floats → BigInt via `FromPrimitive::from_f*`: returns `None` for
// NaN/±INF, `Some(truncated_int)` for finite values (drops fractional
// part). Matches the truncating-fractional behavior of the primitive
// `TryCast<i32> for f64` blanket via `NumCast`.
impl TryCastElement<BigInt> for f32 {
    #[inline]
    fn try_cast_element(self) -> Option<BigInt> {
        BigInt::from_f32(self)
    }
}

impl TryCastElement<BigInt> for f64 {
    #[inline]
    fn try_cast_element(self) -> Option<BigInt> {
        BigInt::from_f64(self)
    }
}

impl TryCastElement<BigInt> for BigUint {
    #[inline]
    fn try_cast_element(self) -> Option<BigInt> {
        Some(BigInt::from(self))
    }
}

impl TryCastElement<BigInt> for BigInt {
    #[inline]
    fn try_cast_element(self) -> Option<BigInt> {
        Some(self)
    }
}

// ---------- TryCastElement: source → BigUint ----------

macro_rules! try_cast_element_to_biguint_from_unsigned {
    ($($T:ty),+ $(,)?) => {
        $(
            impl TryCastElement<BigUint> for $T {
                #[inline]
                fn try_cast_element(self) -> Option<BigUint> {
                    Some(BigUint::from(self))
                }
            }
        )+
    };
}

try_cast_element_to_biguint_from_unsigned!(u8, u16, u32, u64, u128, usize, bool);

// Signed primitives → BigUint: fails on negative values.
macro_rules! try_cast_element_to_biguint_from_signed {
    ($($T:ty),+ $(,)?) => {
        $(
            impl TryCastElement<BigUint> for $T {
                #[inline]
                fn try_cast_element(self) -> Option<BigUint> {
                    BigUint::try_from(self).ok()
                }
            }
        )+
    };
}

try_cast_element_to_biguint_from_signed!(i8, i16, i32, i64, i128, isize);

// Floats → BigUint: same caveat as BigInt (truncates fractional);
// also fails on negative values.
impl TryCastElement<BigUint> for f32 {
    #[inline]
    fn try_cast_element(self) -> Option<BigUint> {
        BigUint::from_f32(self)
    }
}

impl TryCastElement<BigUint> for f64 {
    #[inline]
    fn try_cast_element(self) -> Option<BigUint> {
        BigUint::from_f64(self)
    }
}

impl TryCastElement<BigUint> for BigInt {
    #[inline]
    fn try_cast_element(self) -> Option<BigUint> {
        BigUint::try_from(self).ok()
    }
}

impl TryCastElement<BigUint> for BigUint {
    #[inline]
    fn try_cast_element(self) -> Option<BigUint> {
        Some(self)
    }
}

// ---------- TryCastElement: BigInt/BigUint → primitive ----------
//
// Both `BigInt` and `BigUint` impl `ToPrimitive` upstream, and
// primitives impl `NumCast`. Delegate to `NumCast::from`, which
// returns `None` on overflow.

impl<U> TryCastElement<U> for BigInt
where
    U: NumCast + crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<U> {
        <U as NumCast>::from(self)
    }
}

impl<U> TryCastElement<U> for BigUint
where
    U: NumCast + crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<U> {
        <U as NumCast>::from(self)
    }
}

// ---------- LossyCastElement: BigInt/BigUint → primitive ----------
//
// Saturating downcast. Out-of-range values clamp to the target's
// extremum; for `BigUint` (always non-negative), overflow always
// clamps to `U::max_value()`.

impl<U> LossyCastElement<U> for BigInt
where
    U: NumCast + Bounded + crate::cast::Primitive,
{
    fn lossy_cast_element(self) -> U {
        let is_negative = self.is_negative();
        NumCast::from(self).unwrap_or_else(|| {
            if is_negative {
                U::min_value()
            } else {
                U::max_value()
            }
        })
    }
}

impl<U> LossyCastElement<U> for BigUint
where
    U: NumCast + Bounded + crate::cast::Primitive,
{
    fn lossy_cast_element(self) -> U {
        NumCast::from(self).unwrap_or_else(U::max_value)
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::{BigInt, BigUint, ToBigInt};

    use crate::factory::FiniteFactory;
    use crate::measure::{Count, Width};
    use crate::numeric::Midpointable;
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

    // === Cast trait coverage ===

    mod cast {
        use num_bigint::{BigInt, BigUint};

        use crate::cast::{Cast, LossyCast, TryCast};
        use crate::error::Error;
        use crate::factory::FiniteFactory;
        use crate::sets::FiniteInterval;

        // ---------- Cast (infallible) ----------

        #[test]
        fn cast_int_to_bigint() {
            let x = FiniteInterval::closed(0_i32, 10);
            let y: FiniteInterval<BigInt> = x.cast();
            assert_eq!(y, FiniteInterval::closed(BigInt::from(0), BigInt::from(10)));
        }

        #[test]
        fn cast_u64_to_biguint() {
            let x = FiniteInterval::closed(0_u64, u64::MAX);
            let y: FiniteInterval<BigUint> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(BigUint::from(0u64), BigUint::from(u64::MAX))
            );
        }

        #[test]
        fn cast_biguint_to_bigint() {
            let x = FiniteInterval::closed(BigUint::from(0u32), BigUint::from(100u32));
            let y: FiniteInterval<BigInt> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(BigInt::from(0), BigInt::from(100))
            );
        }

        // ---------- TryCast (BigInt/BigUint → primitive via NumCast) ----------

        #[test]
        fn try_cast_bigint_to_i64() {
            let x = FiniteInterval::closed(BigInt::from(-100), BigInt::from(100));
            let y: FiniteInterval<i64> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(-100_i64, 100));
        }

        #[test]
        fn try_cast_bigint_to_i32_overflow_errors() {
            let huge = BigInt::from(i64::MAX);
            let x = FiniteInterval::closed(BigInt::from(0), huge);
            let y: Result<FiniteInterval<i32>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- TryCast (primitive → BigInt) ----------

        #[test]
        fn try_cast_int_to_bigint() {
            let x = FiniteInterval::closed(0_i32, 10);
            let y: FiniteInterval<BigInt> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(BigInt::from(0), BigInt::from(10)));
        }

        #[test]
        fn try_cast_f64_to_bigint_truncates_fractional() {
            // Closed-closed integer-valued floats: lossless.
            let x = FiniteInterval::closed(0.0_f64, 10.0);
            let y: FiniteInterval<BigInt> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(BigInt::from(0), BigInt::from(10)));
        }

        // ---------- TryCast: signed → BigUint fails on negative ----------

        #[test]
        fn try_cast_negative_to_biguint_errors() {
            let x = FiniteInterval::closed(-10_i32, 10);
            let y: Result<FiniteInterval<BigUint>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn try_cast_bigint_to_biguint_negative_errors() {
            let x = FiniteInterval::closed(BigInt::from(-5), BigInt::from(10));
            let y: Result<FiniteInterval<BigUint>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- LossyCast (BigInt → primitive with saturation) ----------

        #[test]
        fn lossy_cast_bigint_to_i32_clamps_positive() {
            let huge = BigInt::from(i64::MAX);
            let x = FiniteInterval::closed(BigInt::from(0), huge);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_i32, i32::MAX));
        }

        #[test]
        fn lossy_cast_bigint_to_i32_clamps_negative() {
            let very_neg = BigInt::from(i64::MIN);
            let very_pos = BigInt::from(i64::MAX);
            let x = FiniteInterval::closed(very_neg, very_pos);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(i32::MIN, i32::MAX));
        }

        #[test]
        fn lossy_cast_bigint_to_u32_negative_clamps_to_zero() {
            let neg = BigInt::from(-100);
            let pos = BigInt::from(100);
            let x = FiniteInterval::closed(neg, pos);
            let y: FiniteInterval<u32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_u32, 100));
        }

        #[test]
        fn lossy_cast_biguint_to_u32_overflow_clamps_to_max() {
            // BigUint::MAX is unbounded; values above u32::MAX clamp.
            let huge: BigUint = BigUint::from(1u32) << 200;
            let x = FiniteInterval::closed(BigUint::from(0u32), huge);
            let y: FiniteInterval<u32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_u32, u32::MAX));
        }
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

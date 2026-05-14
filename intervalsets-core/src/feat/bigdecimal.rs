use core::convert::Infallible;
use core::ops::Add;

use bigdecimal::{BigDecimal, Signed, Zero};
use num_traits::{Bounded, NumCast};

use crate::cast::{CastElement, LossyCastElement, TryCastElement};
use crate::default_continuous_element_impl;
use crate::error::MathError;
use crate::numeric::Midpointable;
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

default_continuous_element_impl!(BigDecimal);

impl Midpointable for BigDecimal {
    type Error = core::convert::Infallible;

    /// Infallible: `BigDecimal` is arbitrary precision, so the midpoint
    /// of any pair is always representable.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok(self.add(other).half())
    }
}

// === Value-level TryOp impls (E3) ===
//
// `BigDecimal` is arbitrary precision: add/sub/mul cannot fail.
// Division panics on `/0`; we pre-check and surface that as
// `MathError::Domain`. (Non-terminating expansions like `1/3` are
// handled by `bigdecimal` internally via its precision setting and do
// not raise an error.)

impl TryAdd for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self + rhs)
    }
}

impl TrySub for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self - rhs)
    }
}

impl TryMul for BigDecimal {
    type Output = BigDecimal;
    type Error = Infallible;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        Ok(self * rhs)
    }
}

impl TryDiv for BigDecimal {
    type Output = BigDecimal;
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
// `Cast<Interval<BigDecimal>> for Interval<T>` requires
// `T: CastElement<BigDecimal>`. We provide that for every primitive
// numeric type:
//
// - `{i*, u*} → BigDecimal`: lossless via the upstream `From<int> for
//   BigDecimal` impls.
// - `{f32, f64} → BigDecimal`: lossless **given the FiniteBound
//   invariant**. `BigDecimal::try_from` only fails for NaN/±INF, both
//   of which `Element::validate` rejects at construction. So inside
//   `cast_element` we `.expect()` and document the precondition.
//   Tier 4 `new_assume_valid` misuse that smuggles NaN into
//   `FiniteBound<f64>` would reach the panic.
//
// `BigDecimal` does not impl `NumCast` (orphan rule blocks us from
// adding it), so the `TryCast` / `LossyCast` surface needs explicit
// `TryCastElement` / `LossyCastElement` impls (below).
//
// `LossyCast` targeting `BigDecimal` is intentionally not provided:
// `BigDecimal` is arbitrary precision (no `Bounded` impl, no
// saturation extremum), so the trait's snap-to-extremum semantics have
// no meaningful interpretation. `Cast` covers every primitive source
// (including floats) into `BigDecimal` losslessly.

// `CastElement<BigDecimal>` for integer primitives — `BigDecimal` has
// `From<int>` upstream, so this is total.
macro_rules! cast_element_int_to_bigdec {
    ($($T:ty),+ $(,)?) => {
        $(
            impl CastElement<BigDecimal> for $T {
                #[inline]
                fn cast_element(self) -> BigDecimal {
                    BigDecimal::from(self)
                }
            }
        )+
    };
}

cast_element_int_to_bigdec!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// `CastElement<BigDecimal>` for `f32` / `f64`. Sound because callers
// from the set-level `Cast` impls always pass values that have already
// passed `Element::validate` (which rejects NaN / ±INF) — the only
// values that would make `BigDecimal::try_from` fail. Tier 4
// `new_assume_valid` misuse can reach the `.expect()` panic.
impl CastElement<BigDecimal> for f32 {
    #[inline]
    fn cast_element(self) -> BigDecimal {
        BigDecimal::try_from(self).expect(
            "CastElement<BigDecimal> for f32 requires a finite input; \
             FiniteBound<f32> invariant ensures this via Element::validate. \
             Reaching this panic indicates Tier 4 bypass misuse.",
        )
    }
}

impl CastElement<BigDecimal> for f64 {
    #[inline]
    fn cast_element(self) -> BigDecimal {
        BigDecimal::try_from(self).expect(
            "CastElement<BigDecimal> for f64 requires a finite input; \
             FiniteBound<f64> invariant ensures this via Element::validate. \
             Reaching this panic indicates Tier 4 bypass misuse.",
        )
    }
}

// Reflexive identity.
impl CastElement<BigDecimal> for BigDecimal {
    #[inline]
    fn cast_element(self) -> BigDecimal {
        self
    }
}

// `LossyCastElement<U> for BigDecimal` (primitive `U`) — saturating
// downcast. Out-of-range values clamp to `U::min_value()` /
// `U::max_value()` based on sign.
impl<U> LossyCastElement<U> for BigDecimal
where
    U: NumCast + Bounded + crate::cast::Primitive,
{
    fn lossy_cast_element(self) -> U {
        // Capture sign before the value is consumed by `NumCast::from`.
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

// === TryCastElement impls ===
//
// Cover both directions:
//
// - `T_primitive -> BigDecimal`: integer sources via `From<T> for
//   BigDecimal` (lossless, always `Some`); `f32`/`f64` via
//   `BigDecimal: TryFrom<f*>` (returns `None` for NaN).
//
// - `BigDecimal -> U_primitive`: delegate to `NumCast::from` since
//   `BigDecimal: ToPrimitive` and the target primitive is `NumCast`.
//
// - `BigDecimal -> BigDecimal`: identity (reflexive cast, returns
//   `Some(self)`).
//
// The blanket primitive impl in `cast::element` does not fire here
// because the sealed `Primitive` marker excludes `BigDecimal`.

macro_rules! try_cast_to_bigdec_from_int {
    ($($T:ty),+ $(,)?) => {
        $(
            impl TryCastElement<BigDecimal> for $T {
                #[inline]
                fn try_cast_element(self) -> Option<BigDecimal> {
                    Some(BigDecimal::from(self))
                }
            }
        )+
    };
}

try_cast_to_bigdec_from_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

impl TryCastElement<BigDecimal> for f32 {
    #[inline]
    fn try_cast_element(self) -> Option<BigDecimal> {
        BigDecimal::try_from(self).ok()
    }
}

impl TryCastElement<BigDecimal> for f64 {
    #[inline]
    fn try_cast_element(self) -> Option<BigDecimal> {
        BigDecimal::try_from(self).ok()
    }
}

// `BigDecimal -> U_primitive`: blanket via NumCast::from (works
// because `BigDecimal: ToPrimitive`).
impl<U> TryCastElement<U> for BigDecimal
where
    U: NumCast + crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<U> {
        <U as NumCast>::from(self)
    }
}

// Reflexive `BigDecimal -> BigDecimal`.
impl TryCastElement<BigDecimal> for BigDecimal {
    #[inline]
    fn try_cast_element(self) -> Option<BigDecimal> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_big_decimal() -> Result<(), bigdecimal::ParseBigDecimalError> {
        let x = EnumInterval::closed(BigDecimal::from_str("0.0")?, BigDecimal::from_str("10.0")?);
        assert_eq!(x.measure().finite(), BigDecimal::from_str("10")?);
        Ok(())
    }

    #[test]
    fn test_try_ops_bigdecimal() -> Result<(), bigdecimal::ParseBigDecimalError> {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = BigDecimal::from_str("10.0")?;
        let b = BigDecimal::from_str("4.0")?;
        let two_five = BigDecimal::from_str("2.5")?;

        assert_eq!(
            a.clone().try_add(b.clone()).unwrap(),
            BigDecimal::from_str("14.0")?
        );
        assert_eq!(
            a.clone().try_sub(b.clone()).unwrap(),
            BigDecimal::from_str("6.0")?
        );
        assert_eq!(
            a.clone().try_mul(b.clone()).unwrap(),
            BigDecimal::from_str("40.00")?
        );
        assert_eq!(a.clone().try_div(b.clone()).unwrap(), two_five);

        // /0
        assert_eq!(
            a.try_div(BigDecimal::from_str("0")?),
            Err(crate::error::MathError::Domain)
        );

        Ok(())
    }

    // === Cast trait coverage ===

    mod cast {
        use core::str::FromStr;

        use super::*;
        use crate::cast::{Cast, LossyCast, TryCast};
        use crate::error::Error;
        use crate::sets::FiniteInterval;

        // ---------- Cast (int → BigDecimal lossless widening) ----------

        #[test]
        fn cast_int_to_bigdecimal() {
            let x = FiniteInterval::closed(0_i32, 10);
            let y: FiniteInterval<BigDecimal> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(BigDecimal::from(0), BigDecimal::from(10))
            );
        }

        #[test]
        fn cast_i64_to_bigdecimal() {
            let x = FiniteInterval::closed(0_i64, i64::MAX);
            let y: FiniteInterval<BigDecimal> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(BigDecimal::from(0), BigDecimal::from(i64::MAX))
            );
        }

        // ---------- TryCast (BigDecimal → primitive via existing ToPrimitive) ----------

        #[test]
        fn try_cast_bigdecimal_to_i64() {
            let x = FiniteInterval::closed(
                BigDecimal::from_str("0").unwrap(),
                BigDecimal::from_str("10").unwrap(),
            );
            let y: FiniteInterval<i64> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(0_i64, 10));
        }

        #[test]
        fn try_cast_bigdecimal_to_i32_overflow_errors() {
            // BigDecimal value bigger than i32::MAX → NumCast returns
            // None → Error::InvalidBoundLimit.
            let huge = BigDecimal::from(i64::MAX);
            let x = FiniteInterval::closed(BigDecimal::from(0), huge);
            let y: Result<FiniteInterval<i32>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn try_cast_bigdecimal_to_f64() {
            let x = FiniteInterval::closed(
                BigDecimal::from_str("0.5").unwrap(),
                BigDecimal::from_str("10.25").unwrap(),
            );
            let y: FiniteInterval<f64> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(0.5_f64, 10.25));
        }

        // ---------- LossyCast (BigDecimal → primitive with saturation) ----------

        #[test]
        fn lossy_cast_bigdecimal_to_i32_clamps_positive_overflow() {
            // Way bigger than i32::MAX → saturates to i32::MAX.
            let huge = BigDecimal::from(i64::MAX);
            let x = FiniteInterval::closed(BigDecimal::from(0), huge);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_i32, i32::MAX));
        }

        #[test]
        fn lossy_cast_bigdecimal_to_i32_clamps_negative_overflow() {
            // Way smaller than i32::MIN → saturates to i32::MIN.
            let very_neg = BigDecimal::from(i64::MIN);
            let very_pos = BigDecimal::from(i64::MAX);
            let x = FiniteInterval::closed(very_neg, very_pos);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(i32::MIN, i32::MAX));
        }

        #[test]
        fn lossy_cast_bigdecimal_to_i64_in_range() {
            let x = FiniteInterval::closed(
                BigDecimal::from_str("-100").unwrap(),
                BigDecimal::from_str("100").unwrap(),
            );
            let y: FiniteInterval<i64> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(-100_i64, 100));
        }

        #[test]
        fn lossy_cast_bigdecimal_to_u32_negative_clamps_to_zero() {
            // BigDecimal::to_u32 on a negative value returns None →
            // saturate. is_negative ⇒ U::min_value() ⇒ 0u32.
            let neg = BigDecimal::from(-100_i64);
            let pos = BigDecimal::from(100_i64);
            let x = FiniteInterval::closed(neg, pos);
            let y: FiniteInterval<u32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_u32, 100));
        }

        // ---------- TryCast with BigDecimal as target (new under option 1) ----------

        #[test]
        fn try_cast_int_to_bigdecimal() {
            let x = FiniteInterval::closed(0_i32, 10);
            let y: FiniteInterval<BigDecimal> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(BigDecimal::from(0), BigDecimal::from(10))
            );
        }

        #[test]
        fn try_cast_i64_to_bigdecimal_full_range() {
            // i64::MAX is too big for f64 round-trip, but BigDecimal
            // takes it via the integer From impl losslessly.
            let x = FiniteInterval::closed(i64::MIN, i64::MAX);
            let y: FiniteInterval<BigDecimal> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(BigDecimal::from(i64::MIN), BigDecimal::from(i64::MAX))
            );
        }

        #[test]
        fn try_cast_f64_to_bigdecimal() {
            let x = FiniteInterval::closed(0.5_f64, 10.25);
            let y: FiniteInterval<BigDecimal> = x.try_cast().unwrap();
            let (l, r) = y.view_raw().unwrap();
            assert_eq!(l.value(), &BigDecimal::from_str("0.5").unwrap());
            assert_eq!(r.value(), &BigDecimal::from_str("10.25").unwrap());
        }

        #[test]
        fn try_cast_bigdecimal_to_bigdecimal_identity() {
            let x = FiniteInterval::closed(
                BigDecimal::from_str("1.5").unwrap(),
                BigDecimal::from_str("2.5").unwrap(),
            );
            let y: FiniteInterval<BigDecimal> = x.clone().try_cast().unwrap();
            assert_eq!(x, y);
        }

        // ---------- Infallible Cast f32/f64 → BigDecimal ----------
        // The headline case: FiniteInterval<f*> invariants rule out
        // NaN/±INF, so the conversion to BigDecimal is total (no
        // `.unwrap()` at the call site).

        #[test]
        fn cast_f64_to_bigdecimal_infallible() {
            let x = FiniteInterval::closed(0.5_f64, 10.25);
            let y: FiniteInterval<BigDecimal> = x.cast();
            let (l, r) = y.view_raw().unwrap();
            assert_eq!(l.value(), &BigDecimal::from_str("0.5").unwrap());
            assert_eq!(r.value(), &BigDecimal::from_str("10.25").unwrap());
        }

        #[test]
        fn cast_f32_to_bigdecimal_infallible() {
            let x = FiniteInterval::closed(0.5_f32, 10.25);
            let y: FiniteInterval<BigDecimal> = x.cast();
            let (l, r) = y.view_raw().unwrap();
            assert_eq!(l.value(), &BigDecimal::from_str("0.5").unwrap());
            assert_eq!(r.value(), &BigDecimal::from_str("10.25").unwrap());
        }

        #[test]
        fn cast_f64_extremes_to_bigdecimal_infallible() {
            // f64::MAX is finite — within the FiniteBound invariant —
            // and BigDecimal accommodates it losslessly.
            let x = FiniteInterval::closed(-f64::MAX, f64::MAX);
            let y: FiniteInterval<BigDecimal> = x.cast();
            assert!(y.is_fully_bounded());
        }
    }
}

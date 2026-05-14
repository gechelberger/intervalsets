use fixed::traits::{FromFixed, ToFixed};
// lint doesn't detect usage inside macro
#[allow(unused_imports)]
use num_traits::{Bounded, CheckedAdd, CheckedSub, Zero};

use crate::cast::{LossyCastElement, TryCastElement};
use crate::error::MathError;
use crate::measure::{Countable, Widthable};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

/// private macro for Element on fixed crate types.
macro_rules! fixed_domain {
    ($($t:ty,) +) => {
        $(
            impl<N: typenum::Unsigned> crate::numeric::Element for $t {
                fn try_adjacent(&self, side: crate::bound::Side) -> Option<Self> {
                    let bits = self.to_bits();
                    let next = match side {
                        crate::bound::Side::Left => bits.checked_sub(1)?,
                        crate::bound::Side::Right => bits.checked_add(1)?,
                    };
                    Some(Self::from_bits(next))
                }
            }
        )+
    }
}

fixed_domain!(
    fixed::FixedI8<N>,
    fixed::FixedU8<N>,
    fixed::FixedI16<N>,
    fixed::FixedU16<N>,
    fixed::FixedI32<N>,
    fixed::FixedU32<N>,
    fixed::FixedI64<N>,
    fixed::FixedU64<N>,
    fixed::FixedI128<N>,
    fixed::FixedU128<N>,
);

/// private macro for Midpointable on fixed crate types.
///
/// Each fixed-point type delegates to the fixed crate's inherent
/// `mean` method, the canonical midpoint operation for fixed-point
/// arithmetic. It is overflow-safe and `const`-correct, implemented
/// via the bit trick `(a & b) + ((a ^ b) >> 1)` on the underlying
/// integer bits.
///
/// Note: `Fix::mean` rounds toward negative infinity (floor) for
/// signed types, **not** toward zero like std's signed-integer
/// `midpoint` or our `BigInt` impl. This is a deliberate
/// inheritance from the fixed crate's API — fixed-point users live
/// in fixed's mental model, where switching between
/// `a.mean(b)` and `a.midpoint(b)` should not produce different
/// answers.
macro_rules! fixed_midpoint_delegate_impl {
    ($($t:ty,) +) => {
        $(
            impl<N: typenum::Unsigned> crate::numeric::Midpointable for $t {
                type Error = ::core::convert::Infallible;

                /// Infallible: delegates to the fixed crate's inherent
                /// `mean`, which is total and overflow-safe. Rounds
                /// toward negative infinity for signed types (inherits
                /// from fixed's API; differs from std's signed
                /// `midpoint` which rounds toward zero).
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
                    Ok(self.mean(other))
                }
            }
        )+
    }
}

fixed_midpoint_delegate_impl!(
    fixed::FixedI8<N>,
    fixed::FixedU8<N>,
    fixed::FixedI16<N>,
    fixed::FixedU16<N>,
    fixed::FixedI32<N>,
    fixed::FixedU32<N>,
    fixed::FixedI64<N>,
    fixed::FixedU64<N>,
    fixed::FixedI128<N>,
    fixed::FixedU128<N>,
);

// === Value-level TryOp impls (E3) ===
//
// Fixed-point types have bounded precision; all four ops can overflow
// → `MathError::Range`. `checked_div` returns `None` for both `/0`
// and overflow, so we pre-check zero to surface `/0` as `Domain`.

// `checked_mul` and `checked_div` on Fixed<N> require their matching
// `LeEqU*` bound on `N` (e.g. `FixedI16<N>` needs `N: LeEqU16`), so the
// macro takes the bound per type.
macro_rules! fixed_try_ops_impl {
    ($t:ty, $bound:path) => {
        impl<N: $bound> TryAdd for $t {
            type Output = $t;
            type Error = MathError;

            #[inline]
            fn try_add(self, rhs: Self) -> Result<Self, MathError> {
                self.checked_add(rhs).ok_or(MathError::Range)
            }
        }

        impl<N: $bound> TrySub for $t {
            type Output = $t;
            type Error = MathError;

            #[inline]
            fn try_sub(self, rhs: Self) -> Result<Self, MathError> {
                self.checked_sub(rhs).ok_or(MathError::Range)
            }
        }

        impl<N: $bound> TryMul for $t {
            type Output = $t;
            type Error = MathError;

            #[inline]
            fn try_mul(self, rhs: Self) -> Result<Self, MathError> {
                self.checked_mul(rhs).ok_or(MathError::Range)
            }
        }

        impl<N: $bound> TryDiv for $t {
            type Output = $t;
            type Error = MathError;

            #[inline]
            fn try_div(self, rhs: Self) -> Result<Self, MathError> {
                if rhs.is_zero() {
                    return Err(MathError::Domain);
                }
                self.checked_div(rhs).ok_or(MathError::Range)
            }
        }
    };
}

fixed_try_ops_impl!(fixed::FixedI8<N>, fixed::types::extra::LeEqU8);
fixed_try_ops_impl!(fixed::FixedU8<N>, fixed::types::extra::LeEqU8);
fixed_try_ops_impl!(fixed::FixedI16<N>, fixed::types::extra::LeEqU16);
fixed_try_ops_impl!(fixed::FixedU16<N>, fixed::types::extra::LeEqU16);
fixed_try_ops_impl!(fixed::FixedI32<N>, fixed::types::extra::LeEqU32);
fixed_try_ops_impl!(fixed::FixedU32<N>, fixed::types::extra::LeEqU32);
fixed_try_ops_impl!(fixed::FixedI64<N>, fixed::types::extra::LeEqU64);
fixed_try_ops_impl!(fixed::FixedU64<N>, fixed::types::extra::LeEqU64);
fixed_try_ops_impl!(fixed::FixedI128<N>, fixed::types::extra::LeEqU128);
fixed_try_ops_impl!(fixed::FixedU128<N>, fixed::types::extra::LeEqU128);

// Widthable: fixed-point widths can overflow at extremes (e.g.
// MAX - MIN); use `checked_sub` to surface overflow as `None`.
macro_rules! fixed_width_impl {
    ($t:ty) => {
        impl<N: typenum::Unsigned> Widthable for $t {
            type Output = $t;

            fn width_between(left: &Self, right: &Self) -> Option<Self::Output> {
                right.checked_sub(left)
            }
        }
    };
}

fixed_width_impl!(fixed::FixedI8<N>);
fixed_width_impl!(fixed::FixedU8<N>);
fixed_width_impl!(fixed::FixedI16<N>);
fixed_width_impl!(fixed::FixedU16<N>);
fixed_width_impl!(fixed::FixedI32<N>);
fixed_width_impl!(fixed::FixedU32<N>);
fixed_width_impl!(fixed::FixedI64<N>);
fixed_width_impl!(fixed::FixedU64<N>);
fixed_width_impl!(fixed::FixedI128<N>);
fixed_width_impl!(fixed::FixedU128<N>);

// Countable: fixed-point types are discrete (one representation per
// ULP); count of `[left, right]` = bit-rep diff + 1. Output is u128 to
// mirror the primitive Countable widening convention.
//
// Narrow widths (≤ 64 bits) widen the bit diff to i128 before
// subtraction — no intermediate overflow possible, always returns
// `Some`. 128-bit fixed types need bespoke handling because the count
// of `[MIN, MAX]` is 2^128, one past the u128 representable range
// (parallel to the i128 / u128 Countable impls in measure/count.rs).
macro_rules! fixed_countable_narrow_impl {
    ($t:ty) => {
        impl<N: typenum::Unsigned> Countable for $t {
            type Output = u128;

            fn count_inclusive(left: &Self, right: &Self) -> Option<u128> {
                // Interval invariant: right >= left, so the i128 diff is non-negative.
                let diff = (right.to_bits() as i128) - (left.to_bits() as i128);
                Some(diff as u128 + 1)
            }
        }
    };
}

fixed_countable_narrow_impl!(fixed::FixedI8<N>);
fixed_countable_narrow_impl!(fixed::FixedU8<N>);
fixed_countable_narrow_impl!(fixed::FixedI16<N>);
fixed_countable_narrow_impl!(fixed::FixedU16<N>);
fixed_countable_narrow_impl!(fixed::FixedI32<N>);
fixed_countable_narrow_impl!(fixed::FixedU32<N>);
fixed_countable_narrow_impl!(fixed::FixedI64<N>);
fixed_countable_narrow_impl!(fixed::FixedU64<N>);

impl<N: typenum::Unsigned> Countable for fixed::FixedU128<N> {
    type Output = u128;

    fn count_inclusive(left: &Self, right: &Self) -> Option<u128> {
        right
            .to_bits()
            .checked_sub(left.to_bits())
            .and_then(|d| d.checked_add(1))
    }
}

impl<N: typenum::Unsigned> Countable for fixed::FixedI128<N> {
    type Output = u128;

    fn count_inclusive(left: &Self, right: &Self) -> Option<u128> {
        // Interval invariant: right >= left. wrapping_sub on i128
        // reinterpreted as u128 yields the true unsigned distance up
        // to 2^128 - 1.
        let diff = right.to_bits().wrapping_sub(left.to_bits()) as u128;
        diff.checked_add(1)
    }
}

// === Cast support ===
//
// Fixed-point types are unusually well-suited for our cast traits:
// `fixed` ships its own `ToFixed`/`FromFixed`/`checked_*`/`saturating_*`
// machinery that maps directly onto our element-layer trait shapes.
//
// We provide two blankets per fixed storage type:
//
// 1. **Source-side** (`Src → Fixed<Frac>`) bound on `Src: ToFixed`.
//    Covers every primitive (int/float/bool) and every other Fixed
//    type in one impl. Body delegates to
//    `Fixed::checked_from_num` (TryCast) / `Fixed::saturating_from_num`
//    (LossyCast).
//
// 2. **Target-side** (`Fixed<Frac> → Dst`) bound on `Dst: FromFixed`
//    plus the sealed `Primitive` marker. The `Primitive` bound is
//    what dodges coherence overlap with the source-side blanket for
//    cross-Fixed casts — Rust knows no Fixed type can ever become
//    `Primitive`, so cross-Fixed routing is the exclusive domain of
//    the source-side blanket.
//
// `Cast` (Tier 1, infallible) is **not** provided for fixed types:
// lossless conversion is value-dependent for almost every (storage,
// Frac, source-value) triple. `fixed` exposes its own
// `LosslessTryFrom`/`LossyFrom` traits if a downstream user needs the
// exact-conversion semantics in their own code.
//
// NaN handling: `fixed`'s `checked_from_num(f64::NAN)` returns `None`
// (matching TryCast semantics); `saturating_from_num(f64::NAN)`
// returns `0` (matching LossyCast's "total" contract — no Tier 4
// panic site like the `az` blanket has for integer targets).

macro_rules! fixed_cast_impls {
    ($Fix:ident, $LeEqU:ident) => {
        // --- TryCast: Src → Fix<Frac> ---
        impl<Frac, Src> TryCastElement<fixed::$Fix<Frac>> for Src
        where
            Frac: fixed::types::extra::$LeEqU,
            Src: ToFixed,
        {
            #[inline]
            fn try_cast_element(self) -> Option<fixed::$Fix<Frac>> {
                fixed::$Fix::<Frac>::checked_from_num(self)
            }
        }

        // --- TryCast: Fix<Frac> → primitive ---
        impl<Frac, Dst> TryCastElement<Dst> for fixed::$Fix<Frac>
        where
            Frac: fixed::types::extra::$LeEqU,
            Dst: FromFixed + crate::cast::Primitive,
        {
            #[inline]
            fn try_cast_element(self) -> Option<Dst> {
                self.checked_to_num::<Dst>()
            }
        }

        // --- LossyCast: Src → Fix<Frac> ---
        impl<Frac, Src> LossyCastElement<fixed::$Fix<Frac>> for Src
        where
            Frac: fixed::types::extra::$LeEqU,
            Src: ToFixed,
        {
            #[inline]
            fn lossy_cast_element(self) -> fixed::$Fix<Frac> {
                fixed::$Fix::<Frac>::saturating_from_num(self)
            }
        }

        // --- LossyCast: Fix<Frac> → primitive ---
        impl<Frac, Dst> LossyCastElement<Dst> for fixed::$Fix<Frac>
        where
            Frac: fixed::types::extra::$LeEqU,
            Dst: FromFixed + Bounded + crate::cast::Primitive,
        {
            #[inline]
            fn lossy_cast_element(self) -> Dst {
                self.saturating_to_num::<Dst>()
            }
        }
    };
}

fixed_cast_impls!(FixedI8, LeEqU8);
fixed_cast_impls!(FixedU8, LeEqU8);
fixed_cast_impls!(FixedI16, LeEqU16);
fixed_cast_impls!(FixedU16, LeEqU16);
fixed_cast_impls!(FixedI32, LeEqU32);
fixed_cast_impls!(FixedU32, LeEqU32);
fixed_cast_impls!(FixedI64, LeEqU64);
fixed_cast_impls!(FixedU64, LeEqU64);
fixed_cast_impls!(FixedI128, LeEqU128);
fixed_cast_impls!(FixedU128, LeEqU128);

#[cfg(test)]
mod tests {
    use fixed::types::{I6F2, U6F2};

    use crate::bound::Side::*;
    use crate::numeric::{Element, Midpointable};

    #[test]
    fn test_adjacent() {
        let x = I6F2::from_num(5.50);

        let left = x.try_adjacent(Left);
        assert_eq!(left, Some(I6F2::from_num(5.25)));

        let right = x.try_adjacent(Right);
        assert_eq!(right, Some(I6F2::from_num(5.75)));
    }

    #[test]
    fn test_adjacent_uint() {
        let x = U6F2::from_num(0.0);

        let left = x.try_adjacent(Left);
        assert_eq!(left, None);

        let right = x.try_adjacent(Right);
        assert_eq!(right, Some(U6F2::from_num(0.25)));
    }

    #[test]
    fn test_midpoint_signed() {
        // exact representable midpoint
        let mid = I6F2::from_num(2.0).midpoint(I6F2::from_num(4.0)).unwrap();
        assert_eq!(mid, I6F2::from_num(3.0));

        // Floor rounding (toward -inf) inherited from fixed::mean.
        // (-0.25 + 0.0)/2 = -0.125, not representable in I6F2 (step
        // 0.25). The bit trick `(a & b) + ((a ^ b) >> 1)` on
        // (-1, 0) yields -1, so the fixed-point result is -0.25 --
        // *not* 0.0, which would be std's toward-zero rounding.
        let mid = I6F2::from_num(-0.25).midpoint(I6F2::from_num(0.0)).unwrap();
        assert_eq!(mid, I6F2::from_num(-0.25));

        // No overflow at the bounds of the type.
        assert_eq!(I6F2::MAX.midpoint(I6F2::MAX).unwrap(), I6F2::MAX);
        assert_eq!(I6F2::MIN.midpoint(I6F2::MIN).unwrap(), I6F2::MIN);
    }

    #[test]
    fn test_midpoint_unsigned() {
        let mid = U6F2::from_num(2.0).midpoint(U6F2::from_num(4.0)).unwrap();
        assert_eq!(mid, U6F2::from_num(3.0));

        // No overflow at MAX.
        assert_eq!(U6F2::MAX.midpoint(U6F2::MAX).unwrap(), U6F2::MAX);
    }

    #[test]
    fn test_try_ops_fixed_signed() {
        use crate::error::MathError;
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = I6F2::from_num(2.0);
        let b = I6F2::from_num(1.0);

        assert_eq!(a.try_add(b).unwrap(), I6F2::from_num(3.0));
        assert_eq!(a.try_sub(b).unwrap(), I6F2::from_num(1.0));
        assert_eq!(a.try_mul(b).unwrap(), I6F2::from_num(2.0));
        assert_eq!(a.try_div(b).unwrap(), I6F2::from_num(2.0));

        // /0 → Domain
        assert_eq!(a.try_div(I6F2::from_num(0.0)), Err(MathError::Domain));

        // Overflow at MAX → Range
        assert_eq!(
            I6F2::MAX.try_add(I6F2::from_num(1.0)),
            Err(MathError::Range)
        );
        assert_eq!(
            I6F2::MIN.try_sub(I6F2::from_num(1.0)),
            Err(MathError::Range)
        );
    }

    // === Cast trait coverage ===

    mod cast {
        use fixed::types::{I16F16, I32F0, I6F2, U16F16, U6F2};

        use crate::cast::{LossyCast, TryCast};
        use crate::error::Error;
        use crate::factory::FiniteFactory;
        use crate::sets::FiniteInterval;

        // ---------- TryCast: primitive → Fixed ----------

        #[test]
        fn try_cast_i32_to_i16f16_in_range() {
            // I16F16 holds values in roughly [-32768, 32768).
            let x = FiniteInterval::closed(-100_i32, 100);
            let y: FiniteInterval<I16F16> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(I16F16::from_num(-100), I16F16::from_num(100))
            );
        }

        #[test]
        fn try_cast_i32_to_i6f2_overflow_errors() {
            // I6F2 only holds [-32.0, 31.75] approximately.
            let x = FiniteInterval::closed(0_i32, 1000);
            let y: Result<FiniteInterval<I6F2>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn try_cast_f64_to_i6f2_in_range() {
            let x = FiniteInterval::closed(0.0_f64, 10.5);
            let y: FiniteInterval<I6F2> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(I6F2::from_num(0), I6F2::from_num(10.5))
            );
        }

        #[test]
        fn try_cast_f64_to_i6f2_overflow_errors() {
            let x = FiniteInterval::closed(0.0_f64, 1000.0);
            let y: Result<FiniteInterval<I6F2>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- TryCast: Fixed → primitive ----------

        #[test]
        fn try_cast_i16f16_to_i32() {
            let x = FiniteInterval::closed(I16F16::from_num(-100), I16F16::from_num(100));
            let y: FiniteInterval<i32> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(-100_i32, 100));
        }

        #[test]
        fn try_cast_i16f16_to_i8_overflow_errors() {
            let x = FiniteInterval::closed(I16F16::from_num(-1000), I16F16::from_num(1000));
            let y: Result<FiniteInterval<i8>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- TryCast: Fixed → Fixed (cross-type) ----------

        #[test]
        fn try_cast_i6f2_to_i16f16_widening() {
            let x = FiniteInterval::closed(I6F2::from_num(-30.25), I6F2::from_num(31.75));
            let y: FiniteInterval<I16F16> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(I16F16::from_num(-30.25), I16F16::from_num(31.75))
            );
        }

        #[test]
        fn try_cast_i16f16_to_i6f2_overflow_errors() {
            let x = FiniteInterval::closed(I16F16::from_num(-100), I16F16::from_num(100));
            let y: Result<FiniteInterval<I6F2>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- TryCast: signed Fixed → unsigned Fixed ----------

        #[test]
        fn try_cast_i6f2_negative_to_u6f2_errors() {
            let x = FiniteInterval::closed(I6F2::from_num(-5), I6F2::from_num(5));
            let y: Result<FiniteInterval<U6F2>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- LossyCast: primitive → Fixed (saturating) ----------

        #[test]
        fn lossy_cast_i32_to_i6f2_clamps() {
            // I6F2 max ≈ 31.75, min ≈ -32.0.
            let x = FiniteInterval::closed(-1000_i32, 1000);
            let y: FiniteInterval<I6F2> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(I6F2::MIN, I6F2::MAX));
        }

        #[test]
        fn lossy_cast_f64_to_i6f2_clamps() {
            let x = FiniteInterval::closed(-1000.0_f64, 1000.0);
            let y: FiniteInterval<I6F2> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(I6F2::MIN, I6F2::MAX));
        }

        // ---------- LossyCast: Fixed → primitive (saturating) ----------

        #[test]
        fn lossy_cast_i32f0_to_i8_clamps() {
            let x = FiniteInterval::closed(I32F0::from_num(-1000), I32F0::from_num(1000));
            let y: FiniteInterval<i8> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(i8::MIN, i8::MAX));
        }

        #[test]
        fn lossy_cast_u16f16_to_i8_overflow_clamps_to_max() {
            let big = U16F16::from_num(1000);
            let x = FiniteInterval::closed(U16F16::from_num(0), big);
            let y: FiniteInterval<i8> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_i8, i8::MAX));
        }

        // ---------- LossyCast: Fixed → Fixed (cross-type) ----------

        #[test]
        fn lossy_cast_i16f16_to_i6f2_clamps() {
            // I16F16 has wider integer range; I6F2 maxes at ~31.75.
            let x = FiniteInterval::closed(I16F16::from_num(-1000), I16F16::from_num(1000));
            let y: FiniteInterval<I6F2> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(I6F2::MIN, I6F2::MAX));
        }
    }

    #[test]
    fn test_cardinality_fixed_signed() {
        use fixed::types::I6F2;

        use crate::factory::FiniteFactory;
        use crate::measure::Cardinality;
        use crate::sets::FiniteInterval;

        // I6F2 step is 0.25: count of [0.0, 1.0] = 5 (0.00, 0.25, 0.50, 0.75, 1.00).
        let x = FiniteInterval::closed(I6F2::from_num(0.0), I6F2::from_num(1.0));
        assert_eq!(x.cardinality().finite(), 5_u128);

        // Singleton: count_inclusive(x, x) = 1.
        let s = FiniteInterval::closed(I6F2::from_num(3.5), I6F2::from_num(3.5));
        assert_eq!(s.cardinality().finite(), 1_u128);

        // Full range of I6F2 (i8 bits, U2 fractional bits): 256 representable
        // values, fits trivially in u128.
        let full = FiniteInterval::closed(I6F2::MIN, I6F2::MAX);
        assert_eq!(full.cardinality().finite(), 256_u128);
    }

    #[test]
    fn test_cardinality_fixed_unsigned() {
        use fixed::types::U6F2;

        use crate::factory::FiniteFactory;
        use crate::measure::Cardinality;
        use crate::sets::FiniteInterval;

        // U6F2 step is 0.25: count of [0.0, 0.5] = 3 (0.00, 0.25, 0.50).
        let x = FiniteInterval::closed(U6F2::from_num(0.0), U6F2::from_num(0.5));
        assert_eq!(x.cardinality().finite(), 3_u128);
    }

    #[test]
    fn test_cardinality_fixed_i128_full_range_overflows_u128() {
        use fixed::types::I64F64;

        use crate::factory::FiniteFactory;
        use crate::measure::Cardinality;
        use crate::sets::FiniteInterval;

        // I64F64 = FixedI128<U64> has i128 bit width, so [MIN, MAX] has
        // 2^128 representations — one past u128's range.
        let x = FiniteInterval::closed(I64F64::MIN, I64F64::MAX);
        assert!(x.try_cardinality().is_err());
    }

    #[test]
    fn test_cardinality_fixed_u128_full_range_overflows_u128() {
        use fixed::types::U64F64;

        use crate::factory::FiniteFactory;
        use crate::measure::Cardinality;
        use crate::sets::FiniteInterval;

        let x = FiniteInterval::closed(U64F64::MIN, U64F64::MAX);
        assert!(x.try_cardinality().is_err());
    }

    #[test]
    fn test_try_ops_fixed_unsigned() {
        use crate::error::MathError;
        use crate::ops::math::{TryDiv, TrySub};

        // unsigned underflow → Range
        assert_eq!(
            U6F2::from_num(0.0).try_sub(U6F2::from_num(1.0)),
            Err(MathError::Range)
        );

        // /0 → Domain
        assert_eq!(
            U6F2::from_num(1.0).try_div(U6F2::from_num(0.0)),
            Err(MathError::Domain)
        );
    }
}

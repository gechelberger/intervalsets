use num_traits::{Bounded, NumCast};
use rust_decimal::Decimal;

use crate::bound::Side;
use crate::cast::{CastElement, LossyCastElement, TryCastElement};
use crate::error::MathError;
use crate::numeric::{ContinuousKind, Element, Midpointable};
use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

impl Element for Decimal {
    type Kind = ContinuousKind;
    type Measure = Decimal;

    #[inline]
    fn try_adjacent(&self, _: Side) -> Option<Self> {
        None
    }

    #[inline]
    fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
        // Decimal has bounded precision (≈ ±7.92e28); the diff can
        // overflow at extremes (e.g. `Decimal::MAX - Decimal::MIN`).
        right.checked_sub(*left)
    }
}

impl Midpointable for Decimal {
    type Error = MathError;

    /// Computes the midpoint as `(self / 2) + (other / 2)`, halving
    /// each input first so the leading addition cannot overflow the
    /// `Decimal` range (≈ ±7.92e28). Result is exact for typical
    /// inputs and within 0.5 ULP of the true midpoint at extreme
    /// precision (where each half rounds).
    ///
    /// # Errors
    ///
    /// Returns [`MathError::Range`] when rounding each half pushes the
    /// addition out of range — for example `Decimal::MAX.midpoint(MAX)`,
    /// where `MAX / 2` rounds up by 0.5 and the two halves sum to
    /// `MAX + 1`. Symmetric on the negative side for `MIN`.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        (self / Decimal::TWO)
            .checked_add(other / Decimal::TWO)
            .ok_or(MathError::Range)
    }
}

// === Value-level TryOp impls (E3) ===
//
// `Decimal` has bounded precision (≈ ±7.92e28); all four ops can
// overflow → `MathError::Range`. `Decimal::checked_div` returns `None`
// for both `/0` and overflow, so we pre-check zero to surface `/0` as
// `MathError::Domain`.

impl TryAdd for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_add(rhs).ok_or(MathError::Range)
    }
}

impl TrySub for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_sub(rhs).ok_or(MathError::Range)
    }
}

impl TryMul for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self, Self::Error> {
        self.checked_mul(rhs).ok_or(MathError::Range)
    }
}

impl TryDiv for Decimal {
    type Output = Decimal;
    type Error = MathError;

    #[inline]
    fn try_div(self, rhs: Self) -> Result<Self, Self::Error> {
        if rhs.is_zero() {
            return Err(MathError::Domain);
        }
        self.checked_div(rhs).ok_or(MathError::Range)
    }
}

// === Cast support ===
//
// `rust_decimal::Decimal` is bounded-precision (≈ ±7.92e28). Every
// primitive integer type fits losslessly (96-bit mantissa), so
// integer-to-Decimal is infallible. Floats can overflow — `f32::MAX`,
// `f64::MAX`, and even modest values like `1e30_f64` exceed Decimal's
// range — so float-to-Decimal is `TryCast` only.
//
// Unlike `BigDecimal`, we **cannot** provide infallible
// `CastElement<Decimal> for f64` because a finite f64 input may still
// overflow Decimal's range. Callers must use `TryCast`.
//
// `Cast` (Tier 1, infallible) is provided for:
// - `{i*, u*, bool} → Decimal` (upstream `From<int>` impls).
// - `Decimal → Decimal` (reflexive).
//
// `TryCast` covers floats → Decimal (via upstream `TryFrom<f*>`) and
// `Decimal → primitive` (via `NumCast::from` over `ToPrimitive`).
//
// `LossyCast` covers `Decimal → primitive` with saturation.
// `LossyCast` *targeting* `Decimal` is intentionally not provided:
// `Decimal` does not impl `num_traits::Bounded` (orphan rule blocks us
// from adding it), so the set-level `LossyCast` bound `U: Bounded`
// isn't satisfiable for `U = Decimal`.
// Upstream PR pending: https://github.com/paupino/rust-decimal/pull/800
// adds `Bounded for Decimal`. When merged + released, drop this caveat
// and add `LossyCast` targeting `Decimal` (mirroring the BigDecimal
// pattern).

// ---------- CastElement<Decimal> for primitive sources ----------
//
// `Decimal: From<int>` is total for every primitive integer type
// (96-bit mantissa fits up to i128/u128). `From<bool>` is not provided
// upstream, so it's omitted here.

macro_rules! cast_element_to_decimal {
    ($($T:ty),+ $(,)?) => {
        $(
            impl CastElement<Decimal> for $T {
                #[inline]
                fn cast_element(self) -> Decimal {
                    Decimal::from(self)
                }
            }
        )+
    };
}

cast_element_to_decimal!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl CastElement<Decimal> for Decimal {
    #[inline]
    fn cast_element(self) -> Decimal {
        self
    }
}

// ---------- TryCastElement: source → Decimal ----------

macro_rules! try_cast_element_to_decimal_from_int {
    ($($T:ty),+ $(,)?) => {
        $(
            impl TryCastElement<Decimal> for $T {
                #[inline]
                fn try_cast_element(self) -> Option<Decimal> {
                    Some(Decimal::from(self))
                }
            }
        )+
    };
}

try_cast_element_to_decimal_from_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

// Floats → Decimal via upstream `TryFrom<f*>`: returns `None` for
// NaN/±INF AND for out-of-range finite floats (e.g. `1e30_f64` is
// finite but exceeds `Decimal::MAX`). This is the key difference from
// `BigDecimal`, which is unbounded and accepts every finite float.
impl TryCastElement<Decimal> for f32 {
    #[inline]
    fn try_cast_element(self) -> Option<Decimal> {
        Decimal::try_from(self).ok()
    }
}

impl TryCastElement<Decimal> for f64 {
    #[inline]
    fn try_cast_element(self) -> Option<Decimal> {
        Decimal::try_from(self).ok()
    }
}

impl TryCastElement<Decimal> for Decimal {
    #[inline]
    fn try_cast_element(self) -> Option<Decimal> {
        Some(self)
    }
}

// ---------- TryCastElement: Decimal → primitive ----------

impl<U> TryCastElement<U> for Decimal
where
    U: NumCast + crate::cast::Primitive,
{
    #[inline]
    fn try_cast_element(self) -> Option<U> {
        <U as NumCast>::from(self)
    }
}

// ---------- LossyCastElement: Decimal → primitive ----------

impl<U> LossyCastElement<U> for Decimal
where
    U: NumCast + Bounded + crate::cast::Primitive,
{
    fn lossy_cast_element(self) -> U {
        let is_negative = self.is_sign_negative();
        NumCast::from(self).unwrap_or_else(|| {
            if is_negative {
                U::min_value()
            } else {
                U::max_value()
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_decimal_interval() {
        let interval = EnumInterval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));

        assert_eq!(interval.measure().finite(), Decimal::new(798, 2));
    }

    #[test]
    fn test_midpoint_decimal() {
        // integer-valued, exact
        let mid = Decimal::new(10, 0).midpoint(Decimal::new(20, 0)).unwrap();
        assert_eq!(mid, Decimal::new(15, 0));

        // mixed-scale: 2.02 and 10.0 -> 6.01
        let mid = Decimal::new(202, 2).midpoint(Decimal::new(100, 1)).unwrap();
        assert_eq!(mid, Decimal::new(601, 2));

        // negative midpoint -- Decimal is continuous, so the result is
        // the exact mean (-3.5), not truncation toward zero.
        let mid = Decimal::new(-7, 0).midpoint(Decimal::new(0, 0)).unwrap();
        assert_eq!(mid, Decimal::new(-35, 1));

        // commutativity
        let a = Decimal::new(1_000_001, 3);
        let b = Decimal::new(-3, 0);
        assert_eq!(a.midpoint(b).unwrap(), b.midpoint(a).unwrap());

        // Half-then-add accepts pairs near MAX whose direct sum would
        // overflow. Result may round within 0.5 ULP of the true midpoint;
        // we assert boundedness rather than the exact rounded value.
        let high = Decimal::MAX;
        let lower = Decimal::MAX - Decimal::ONE;
        let mid = high.midpoint(lower).unwrap();
        assert!(mid >= lower && mid <= high);

        // The extreme pair -- both at MAX -- still fails because each
        // half rounds up, pushing the sum above MAX.
        assert_eq!(Decimal::MAX.midpoint(Decimal::MAX), Err(MathError::Range));
        assert_eq!(Decimal::MIN.midpoint(Decimal::MIN), Err(MathError::Range));
    }

    // === Cast trait coverage ===

    mod cast {
        use rust_decimal::Decimal;

        use crate::cast::{Cast, LossyCast, TryCast};
        use crate::error::Error;
        use crate::factory::FiniteFactory;
        use crate::sets::FiniteInterval;

        // ---------- Cast (int → Decimal lossless) ----------

        #[test]
        fn cast_int_to_decimal() {
            let x = FiniteInterval::closed(0_i32, 10);
            let y: FiniteInterval<Decimal> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(Decimal::from(0), Decimal::from(10))
            );
        }

        #[test]
        fn cast_i128_to_decimal() {
            // i128 fits losslessly in Decimal's 96-bit mantissa for
            // values up to roughly ±2^96.
            let x = FiniteInterval::closed(-1_000_000_000_000_i128, 1_000_000_000_000);
            let y: FiniteInterval<Decimal> = x.cast();
            assert_eq!(
                y,
                FiniteInterval::closed(
                    Decimal::from(-1_000_000_000_000_i128),
                    Decimal::from(1_000_000_000_000_i128)
                )
            );
        }

        // ---------- TryCast: f → Decimal (in-range succeeds, overflow errors) ----------

        #[test]
        fn try_cast_f64_to_decimal_in_range() {
            let x = FiniteInterval::closed(0.5_f64, 10.25);
            let y: FiniteInterval<Decimal> = x.try_cast().unwrap();
            assert_eq!(
                y,
                FiniteInterval::closed(
                    Decimal::try_from(0.5_f64).unwrap(),
                    Decimal::try_from(10.25_f64).unwrap(),
                )
            );
        }

        #[test]
        fn try_cast_f64_to_decimal_overflow_errors() {
            // Finite, but well beyond Decimal::MAX (≈ 7.92e28).
            let x = FiniteInterval::closed(0.0_f64, 1e30_f64);
            let y: Result<FiniteInterval<Decimal>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        // ---------- TryCast: Decimal → primitive ----------

        #[test]
        fn try_cast_decimal_to_i64() {
            let x = FiniteInterval::closed(Decimal::from(-100), Decimal::from(100));
            let y: FiniteInterval<i64> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(-100_i64, 100));
        }

        #[test]
        fn try_cast_decimal_to_i32_overflow_errors() {
            // Decimal::MAX is way beyond i32::MAX.
            let x = FiniteInterval::closed(Decimal::ZERO, Decimal::MAX);
            let y: Result<FiniteInterval<i32>, _> = x.try_cast();
            assert!(matches!(y, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn try_cast_decimal_to_f64() {
            let x = FiniteInterval::closed(Decimal::from(0), Decimal::from(10));
            let y: FiniteInterval<f64> = x.try_cast().unwrap();
            assert_eq!(y, FiniteInterval::closed(0.0_f64, 10.0));
        }

        // ---------- LossyCast (Decimal → primitive saturating) ----------

        #[test]
        fn lossy_cast_decimal_to_i32_clamps_positive() {
            let x = FiniteInterval::closed(Decimal::from(0), Decimal::MAX);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_i32, i32::MAX));
        }

        #[test]
        fn lossy_cast_decimal_to_i32_clamps_negative() {
            let x = FiniteInterval::closed(Decimal::MIN, Decimal::MAX);
            let y: FiniteInterval<i32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(i32::MIN, i32::MAX));
        }

        #[test]
        fn lossy_cast_decimal_to_u32_negative_clamps_to_zero() {
            let x = FiniteInterval::closed(Decimal::from(-100), Decimal::from(100));
            let y: FiniteInterval<u32> = x.lossy_cast();
            assert_eq!(y, FiniteInterval::closed(0_u32, 100));
        }
    }

    #[test]
    fn test_try_ops_decimal() {
        use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

        let a = Decimal::new(10, 0);
        let b = Decimal::new(3, 0);

        assert_eq!(a.try_add(b).unwrap(), Decimal::new(13, 0));
        assert_eq!(a.try_sub(b).unwrap(), Decimal::new(7, 0));
        assert_eq!(a.try_mul(b).unwrap(), Decimal::new(30, 0));
        assert_eq!(
            Decimal::new(10, 0).try_div(Decimal::new(2, 0)).unwrap(),
            Decimal::new(5, 0)
        );

        // Domain on /0
        assert_eq!(a.try_div(Decimal::ZERO), Err(MathError::Domain));

        // Range on overflow (MAX + MAX)
        assert_eq!(Decimal::MAX.try_add(Decimal::MAX), Err(MathError::Range));
        assert_eq!(Decimal::MAX.try_mul(Decimal::MAX), Err(MathError::Range));
    }
}

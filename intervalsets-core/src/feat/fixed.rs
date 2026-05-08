// lint doesn't detect usage inside macro
#[allow(unused_imports)]
use num_traits::{CheckedAdd, CheckedSub, Zero};

use crate::error::MathError;
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

/// private macro for Midpoint on fixed crate types.
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
            impl<N: typenum::Unsigned> crate::numeric::Midpoint for $t {
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

#[cfg(test)]
mod tests {
    use fixed::types::{I6F2, U6F2};

    use crate::bound::Side::*;
    use crate::numeric::{Element, Midpoint};

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

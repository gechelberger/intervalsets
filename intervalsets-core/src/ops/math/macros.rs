//! Internal macros that implement the [`TryAdd`](super::TryAdd) /
//! [`TrySub`](super::TrySub) / [`TryMul`](super::TryMul) /
//! [`TryDiv`](super::TryDiv) traits for value-level primitive types.
//!
//! Two lineups:
//!
//! - `impl_try_*_checked!` — for types with a `checked_*` API (signed
//!   and unsigned integer primitives, plus feature-crate types like
//!   `Decimal` and `Fixed*`). Uses the `checked_*` family directly.
//! - `impl_try_*_float_finite!` — for IEEE-754 floats. Performs the op
//!   raw and reports any non-finite result as
//!   [`MathError::Domain`](crate::error::MathError::Domain). The single
//!   `is_finite()` check classifies both `INF` and `NaN` as `Domain`;
//!   see `MathError::Domain`'s docs for the rationale.
//!
//! These macros are `pub(crate)`. Production primitive instantiations
//! live in the per-op `add.rs` / `sub.rs` / `mul.rs` / `div.rs` files
//! (and per-feature `feat/<crate>.rs` files); only the macro
//! definitions and their unit tests live here.
//!
//! Each generated impl returns [`MathError`](crate::error::MathError):
//! - `Range` — `checked_*` returned `None` (integer overflow, including
//!   signed `MIN / -1` for division).
//! - `Domain` — divisor is zero (integer division), or the result is
//!   non-finite (floats).

macro_rules! impl_try_add_checked {
    ($t:ty) => {
        impl $crate::ops::math::TryAdd for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_add(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                self.checked_add(rhs).ok_or($crate::error::MathError::Range)
            }
        }
    };
}
pub(crate) use impl_try_add_checked;

macro_rules! impl_try_sub_checked {
    ($t:ty) => {
        impl $crate::ops::math::TrySub for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_sub(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                self.checked_sub(rhs).ok_or($crate::error::MathError::Range)
            }
        }
    };
}
pub(crate) use impl_try_sub_checked;

macro_rules! impl_try_mul_checked {
    ($t:ty) => {
        impl $crate::ops::math::TryMul for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_mul(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                self.checked_mul(rhs).ok_or($crate::error::MathError::Range)
            }
        }
    };
}
pub(crate) use impl_try_mul_checked;

macro_rules! impl_try_div_checked {
    ($t:ty) => {
        impl $crate::ops::math::TryDiv for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_div(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                if rhs == 0 {
                    return ::core::result::Result::Err($crate::error::MathError::Domain);
                }
                self.checked_div(rhs).ok_or($crate::error::MathError::Range)
            }
        }
    };
}
pub(crate) use impl_try_div_checked;

// IEEE-754 float macros. Each performs the op raw and reports any
// non-finite result (INF or NaN) as `MathError::Domain`. Division
// inherits the same treatment — `1.0 / 0.0 = INF` and `0.0 / 0.0 = NaN`
// both surface as `Domain` without an explicit zero pre-check.

macro_rules! impl_try_add_float_finite {
    ($t:ty) => {
        impl $crate::ops::math::TryAdd for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_add(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                let r = self + rhs;
                if r.is_finite() {
                    ::core::result::Result::Ok(r)
                } else {
                    ::core::result::Result::Err($crate::error::MathError::Domain)
                }
            }
        }
    };
}
pub(crate) use impl_try_add_float_finite;

macro_rules! impl_try_sub_float_finite {
    ($t:ty) => {
        impl $crate::ops::math::TrySub for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_sub(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                let r = self - rhs;
                if r.is_finite() {
                    ::core::result::Result::Ok(r)
                } else {
                    ::core::result::Result::Err($crate::error::MathError::Domain)
                }
            }
        }
    };
}
pub(crate) use impl_try_sub_float_finite;

macro_rules! impl_try_mul_float_finite {
    ($t:ty) => {
        impl $crate::ops::math::TryMul for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_mul(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                let r = self * rhs;
                if r.is_finite() {
                    ::core::result::Result::Ok(r)
                } else {
                    ::core::result::Result::Err($crate::error::MathError::Domain)
                }
            }
        }
    };
}
pub(crate) use impl_try_mul_float_finite;

macro_rules! impl_try_div_float_finite {
    ($t:ty) => {
        impl $crate::ops::math::TryDiv for $t {
            type Output = $t;
            type Error = $crate::error::MathError;

            #[inline]
            fn try_div(self, rhs: $t) -> ::core::result::Result<$t, $crate::error::MathError> {
                let r = self / rhs;
                if r.is_finite() {
                    ::core::result::Result::Ok(r)
                } else {
                    ::core::result::Result::Err($crate::error::MathError::Domain)
                }
            }
        }
    };
}
pub(crate) use impl_try_div_float_finite;

#[cfg(test)]
mod tests {
    use crate::error::MathError;
    use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

    // Crate-local newtype so the macro-generated impls don't collide with any
    // future production-surface impls on bare primitives (E2). `i8` keeps the
    // overflow probes cheap.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct I8(i8);

    impl I8 {
        #[inline]
        fn checked_add(self, rhs: Self) -> Option<Self> {
            self.0.checked_add(rhs.0).map(Self)
        }
        #[inline]
        fn checked_sub(self, rhs: Self) -> Option<Self> {
            self.0.checked_sub(rhs.0).map(Self)
        }
        #[inline]
        fn checked_mul(self, rhs: Self) -> Option<Self> {
            self.0.checked_mul(rhs.0).map(Self)
        }
        #[inline]
        fn checked_div(self, rhs: Self) -> Option<Self> {
            self.0.checked_div(rhs.0).map(Self)
        }
    }

    impl PartialEq<i8> for I8 {
        #[inline]
        fn eq(&self, other: &i8) -> bool {
            self.0 == *other
        }
    }

    super::impl_try_add_checked!(I8);
    super::impl_try_sub_checked!(I8);
    super::impl_try_mul_checked!(I8);
    super::impl_try_div_checked!(I8);

    #[test]
    fn add_ok() {
        assert_eq!(I8(1).try_add(I8(2)), Ok(I8(3)));
    }

    #[test]
    fn add_range_overflow() {
        assert_eq!(I8(i8::MAX).try_add(I8(1)), Err(MathError::Range));
        assert_eq!(I8(i8::MIN).try_add(I8(-1)), Err(MathError::Range));
    }

    #[test]
    fn sub_ok() {
        assert_eq!(I8(5).try_sub(I8(3)), Ok(I8(2)));
    }

    #[test]
    fn sub_range_overflow() {
        assert_eq!(I8(i8::MIN).try_sub(I8(1)), Err(MathError::Range));
    }

    #[test]
    fn mul_ok() {
        assert_eq!(I8(6).try_mul(I8(7)), Ok(I8(42)));
    }

    #[test]
    fn mul_range_overflow() {
        assert_eq!(I8(64).try_mul(I8(2)), Err(MathError::Range));
    }

    #[test]
    fn div_ok() {
        assert_eq!(I8(10).try_div(I8(2)), Ok(I8(5)));
    }

    #[test]
    fn div_domain_on_zero() {
        assert_eq!(I8(1).try_div(I8(0)), Err(MathError::Domain));
        assert_eq!(I8(0).try_div(I8(0)), Err(MathError::Domain));
    }

    #[test]
    fn div_range_on_min_neg_one() {
        assert_eq!(I8(i8::MIN).try_div(I8(-1)), Err(MathError::Range));
    }

    // Crate-local newtype around `f64` so the float-finite macro tests
    // exercise the macros without colliding with any production-surface
    // impls on bare primitives.
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct F64(f64);

    impl ::core::ops::Add for F64 {
        type Output = F64;
        #[inline]
        fn add(self, rhs: Self) -> Self {
            F64(self.0 + rhs.0)
        }
    }
    impl ::core::ops::Sub for F64 {
        type Output = F64;
        #[inline]
        fn sub(self, rhs: Self) -> Self {
            F64(self.0 - rhs.0)
        }
    }
    impl ::core::ops::Mul for F64 {
        type Output = F64;
        #[inline]
        fn mul(self, rhs: Self) -> Self {
            F64(self.0 * rhs.0)
        }
    }
    impl ::core::ops::Div for F64 {
        type Output = F64;
        #[inline]
        fn div(self, rhs: Self) -> Self {
            F64(self.0 / rhs.0)
        }
    }
    impl F64 {
        #[inline]
        fn is_finite(self) -> bool {
            self.0.is_finite()
        }
    }

    super::impl_try_add_float_finite!(F64);
    super::impl_try_sub_float_finite!(F64);
    super::impl_try_mul_float_finite!(F64);
    super::impl_try_div_float_finite!(F64);

    #[test]
    fn float_add_ok() {
        assert_eq!(F64(1.0).try_add(F64(2.0)), Ok(F64(3.0)));
    }

    #[test]
    fn float_add_overflow_to_inf_is_domain() {
        assert_eq!(F64(f64::MAX).try_add(F64(f64::MAX)), Err(MathError::Domain));
    }

    #[test]
    fn float_add_inf_minus_inf_is_domain() {
        assert_eq!(
            F64(f64::INFINITY).try_add(F64(f64::NEG_INFINITY)),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn float_sub_ok() {
        assert_eq!(F64(5.0).try_sub(F64(3.0)), Ok(F64(2.0)));
    }

    #[test]
    fn float_sub_inf_minus_inf_is_domain() {
        assert_eq!(
            F64(f64::INFINITY).try_sub(F64(f64::INFINITY)),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn float_mul_ok() {
        assert_eq!(F64(6.0).try_mul(F64(7.0)), Ok(F64(42.0)));
    }

    #[test]
    fn float_mul_overflow_is_domain() {
        assert_eq!(F64(f64::MAX).try_mul(F64(2.0)), Err(MathError::Domain));
    }

    #[test]
    fn float_div_ok() {
        assert_eq!(F64(10.0).try_div(F64(2.0)), Ok(F64(5.0)));
    }

    #[test]
    fn float_div_by_zero_is_domain() {
        // 1.0 / 0.0 = INF; 0.0 / 0.0 = NaN. Both are non-finite → Domain.
        assert_eq!(F64(1.0).try_div(F64(0.0)), Err(MathError::Domain));
        assert_eq!(F64(0.0).try_div(F64(0.0)), Err(MathError::Domain));
    }
}

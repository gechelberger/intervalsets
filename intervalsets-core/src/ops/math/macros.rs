//! Internal macros that implement the [`TryAdd`](super::TryAdd) /
//! [`TrySub`](super::TrySub) / [`TryMul`](super::TryMul) /
//! [`TryDiv`](super::TryDiv) traits for primitive integer types via the
//! standard `checked_*` API.
//!
//! These macros are `pub(crate)` and intended to be invoked by the
//! per-feature-crate primitive instantiations (E2/E3 of the math
//! recontract). Production primitive instantiations are not in this
//! module — only the macro definitions and their unit tests live here.
//!
//! Each generated impl returns [`MathError`](crate::error::MathError):
//! - `Range` — `checked_*` returned `None` (integer overflow, including
//!   signed `MIN / -1` for division).
//! - `Domain` — divisor is zero (division only).

// E1 lands the macros without any production-code invocation; E2/E3 add the
// callsites. Until then, the macro definitions and their `pub(crate) use`
// re-exports look dead to the non-test build — silence those warnings here
// rather than peppering each macro with its own `#[allow(...)]`.
#![allow(unused_macros, unused_imports)]

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
}

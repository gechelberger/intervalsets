use num_traits::{CheckedAdd, CheckedSub};

use crate::ival::Side;

/// Defines a fallible finite Add operation.
///
/// This operations is very similar to [`num_traits::CheckedAdd`].
/// A pass through macro implementation is provided for types that
/// already implement `CheckedAdd`.
///
/// It has slight semantic differences however for types that are
/// infinite aware. Specifically f32 and f64 which can saturate to
/// +/- infinity. The expected behavior for this trait is to treat
/// such a saturation as an overflow and return None instead.
pub trait TryFiniteAdd<Rhs = Self> {
    type Output;

    fn try_finite_add(&self, rhs: &Self) -> Option<Self::Output>;
}

macro_rules! try_finite_add_float_impl {
    ($t:ty) => {
        impl TryFiniteAdd<Self> for $t {
            type Output = Self;

            fn try_finite_add(&self, rhs: &Self) -> Option<Self::Output> {
                if self.is_nan() || rhs.is_nan() {
                    return None;
                }

                match self + rhs {
                    Self::INFINITY | Self::NEG_INFINITY => None,
                    result => Some(result),
                }
            }
        }
    };
}

try_finite_add_float_impl!(f32);
try_finite_add_float_impl!(f64);

/// A macro adapting [`num_traits::CheckedAdd`] to [`TryFiniteAdd`].
///
/// These must be explicitly adapted rather that a blanket
/// implementation that could conflict if the upstream provides
/// additional implementations of `CheckedAdd`.
#[macro_export]
macro_rules! try_finite_add_checked_impl {
    ($t:ty) => {
        impl TryFiniteAdd<Self> for $t {
            type Output = Self;

            fn try_finite_add(&self, rhs: &Self) -> Option<Self::Output> {
                self.checked_add(rhs)
            }
        }
    };
}

try_finite_add_checked_impl!(u8);
try_finite_add_checked_impl!(u16);
try_finite_add_checked_impl!(u32);
try_finite_add_checked_impl!(u64);
try_finite_add_checked_impl!(u128);
try_finite_add_checked_impl!(usize);

try_finite_add_checked_impl!(i8);
try_finite_add_checked_impl!(i16);
try_finite_add_checked_impl!(i32);
try_finite_add_checked_impl!(i64);
try_finite_add_checked_impl!(i128);
try_finite_add_checked_impl!(isize);

/// Defines a fallible finite Sub operation.
///
/// This operations is very similar to [`num_traits::CheckedSub`].
/// A pass through implementation is provided for types that
/// already implement `CheckedSub`.
///
/// It has slight semantic differences however for types that are
/// infinite aware. Specifically f32 and f64 which can saturate to
/// +/- infinity. The expected behavior for this trait is to treat
/// such a saturation as an overflow and return None instead.
pub trait TryFiniteSub<Rhs = Self> {
    type Output;

    fn try_finite_sub(&self, rhs: &Self) -> Option<Self::Output>;
}

macro_rules! try_finite_sub_float_impl {
    ($t:ty) => {
        impl TryFiniteSub for $t {
            type Output = Self;

            fn try_finite_sub(&self, rhs: &Self) -> Option<Self> {
                if self.is_nan() || rhs.is_nan() {
                    return None;
                }

                match self - rhs {
                    Self::INFINITY | Self::NEG_INFINITY => None,
                    result => Some(result),
                }
            }
        }
    };
}

try_finite_sub_float_impl!(f32);
try_finite_sub_float_impl!(f64);

#[macro_export]
macro_rules! try_finite_sub_checked_impl {
    ($t:ty) => {
        impl TryFiniteSub for $t {
            type Output = Self;

            fn try_finite_sub(&self, rhs: &Self) -> Option<Self> {
                self.checked_sub(rhs)
            }
        }
    };
}

try_finite_sub_checked_impl!(u8);
try_finite_sub_checked_impl!(u16);
try_finite_sub_checked_impl!(u32);
try_finite_sub_checked_impl!(u64);
try_finite_sub_checked_impl!(u128);
try_finite_sub_checked_impl!(usize);

try_finite_sub_checked_impl!(i8);
try_finite_sub_checked_impl!(i16);
try_finite_sub_checked_impl!(i32);
try_finite_sub_checked_impl!(i64);
try_finite_sub_checked_impl!(i128);
try_finite_sub_checked_impl!(isize);

pub trait Domain: Sized + Clone + PartialOrd + PartialEq {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

#[macro_export]
macro_rules! continuous_domain_impl {
    ($t:ty) => {
        impl Domain for $t {
            #[inline]
            fn try_adjacent(&self, side: Side) -> Option<Self> {
                None
            }
        }
    };
}

continuous_domain_impl!(f32);
continuous_domain_impl!(f64);

macro_rules! integer_domain_impl {
    ($t:ty) => {
        impl Domain for $t {
            #[inline]
            fn try_adjacent(&self, side: Side) -> Option<Self> {
                match side {
                    Side::Right => <$t>::checked_add(*self, 1),
                    Side::Left => <$t>::checked_sub(*self, 1),
                }
            }
        }
    };
}

integer_domain_impl!(usize);
integer_domain_impl!(u8);
integer_domain_impl!(u16);
integer_domain_impl!(u32);
integer_domain_impl!(u64);
integer_domain_impl!(u128);

integer_domain_impl!(isize);
integer_domain_impl!(i8);
integer_domain_impl!(i16);
integer_domain_impl!(i32);
integer_domain_impl!(i64);
integer_domain_impl!(i128);

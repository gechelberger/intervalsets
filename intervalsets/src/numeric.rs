//! Control behavior of data types that comprise the elements of a set.

use crate::bound::Side;

pub use num_traits::Zero;

/// Defines the data types whose elements make up a Set.
///
/// `try_adjacent` determines whether the elements are
/// treated as continuous or discrete data.
pub trait Domain: Sized + Clone + PartialOrd + PartialEq {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

/// Automatically implements [`Domain`] for a type.
///
/// [`Interval`](crate::Interval) and [`IntervalSet`](crate::IntervalSet)
/// expect their generic types to implement
/// the [`Domain`] trait. It's primary function is to help normalize **disrete**
/// data types.
///
/// For **continuous** data types, normalization is a **noop**, but the trait
/// still needs to be implemented to meet the trait bounds for Set types.
///
/// This macro provides a default impl for **continuous** types;
///
/// # Example
///
/// ```
/// use intervalsets::continuous_domain_impl;
/// use intervalsets::{Interval, Factory};
/// use intervalsets::ops::Contains;
///
/// #[derive(Clone, PartialEq, PartialOrd)]
/// struct MyFloat(f64);
///
/// continuous_domain_impl!(MyFloat);
///
/// let x = Interval::closed(MyFloat(0.0), MyFloat(10.0));
/// assert_eq!(x.contains(&MyFloat(5.0)), true);
/// ```
#[macro_export]
macro_rules! continuous_domain_impl {
    ($t:ty) => {
        impl $crate::numeric::Domain for $t {
            #[inline]
            fn try_adjacent(&self, side: $crate::Side) -> Option<Self> {
                None
            }
        }
    };
}

continuous_domain_impl!(f32);
continuous_domain_impl!(f64);

macro_rules! integer_domain_impl {
    ($t:ty) => {
        impl $crate::numeric::Domain for $t {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros() {}
}

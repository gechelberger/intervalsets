use crate::bound::Side;

/// Defines the zero value for a type.
///
/// This is intended to work identically to [`num_traits::Zero`].
/// The trait is duplicated in order to work with external
/// types.
pub trait LibZero {
    fn new_zero() -> Self;
}

/// Create [`LibZero`] impl(s) that delegate to [`num_traits::Zero`]
///
/// Example
/// ```
/// use intervalsets::numeric::LibZero;
/// use intervalsets::adapt_num_traits_zero_impl;
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct MyInt(u8);
///
/// impl core::ops::Add for MyInt {
///     type Output = Self;
///     fn add(self, rhs: Self) -> Self {
///         MyInt(self.0 + rhs.0)
///     }
/// }
///
/// impl num_traits::Zero for MyInt {
///     fn zero() -> Self {
///         MyInt(0)
///     }
///
///     fn is_zero(&self) -> bool {
///         self.0 == 0
///     }
/// }
///
/// adapt_num_traits_zero_impl!(MyInt);
/// assert_eq!(MyInt::new_zero(), MyInt(0));
/// ```
#[macro_export]
macro_rules! adapt_num_traits_zero_impl {
    ( $($t:ty), +) => {
        $(
            impl $crate::numeric::LibZero for $t {
                fn new_zero() -> Self {
                    <$t as num_traits::Zero>::zero()
                }
            }
        )*
    };
}

adapt_num_traits_zero_impl!(u8, u16, u32, u64, u128, usize);
adapt_num_traits_zero_impl!(i8, i16, i32, i64, i128, isize);
adapt_num_traits_zero_impl!(f32, f64);

/// Defines the data types whose elements make up a Set.
///
/// `try_adjacent` determines whether the elements are
/// treated as continuous or discrete data.
pub trait Domain: Sized + Clone + PartialOrd + PartialEq {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

/// Automatically implements [`Domain`] for a type.
///
/// [`Interval`] and [`IntervalSet`] expect their generic types to implement
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
/// use intervalsets::Interval;
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

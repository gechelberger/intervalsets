pub use num_traits::Zero;

//use ordered_float::{NotNan, OrderedFloat};
use crate::bound::Side;

/// Defines the data types whose elements make up a Set.
///
/// `try_adjacent` determines whether the elements are
/// treated as continuous or discrete data.
pub trait Domain: Sized + PartialEq + PartialOrd {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

/// Automatically implements [`Domain`] for a type.
///
/// Interval/Set types require generic storage types to implement
/// the [`Domain`] trait. It's primary function is to normalize **disrete**
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
/// use intervalsets_core::continuous_domain_impl;
/// use intervalsets_core::prelude::*;
///
/// #[derive(Clone, PartialEq, PartialOrd)]
/// struct MyFloat(f64);
///
/// continuous_domain_impl!(MyFloat);
///
/// //todo: num_traits::Zero required
///
/// //let x = FiniteInterval::closed(MyFloat(0.0), MyFloat(10.0));
/// //assert_eq!(x.contains(&MyFloat(5.0)), true);
/// ```
#[macro_export]
macro_rules! continuous_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Domain for $t {
                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }
            }
        )+
    }
}

continuous_domain_impl!(f32, f64);
//continuous_domain_impl!(NotNan<f32>, NotNan<f64>);
//continuous_domain_impl!(OrderedFloat<f32>, OrderedFloat<f64>);

macro_rules! integer_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Domain for $t {
                #[inline]
                fn try_adjacent(&self, side: $crate::bound::Side) -> Option<Self> {
                    match side {
                        Side::Right => <$t>::checked_add(*self, 1),
                        Side::Left => <$t>::checked_sub(*self, 1),
                    }
                }
            }
        )+
    }
}

integer_domain_impl!(u8, u16, u32, u64, u128, usize);
integer_domain_impl!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacent() {
        assert_eq!(10.try_adjacent(Side::Right).unwrap(), 11);
        assert_eq!(11.try_adjacent(Side::Left).unwrap(), 10);
    }
}

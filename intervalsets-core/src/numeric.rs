//! # Custom Data Types
//!
//! If you have a data type that is not currently supported
//! out of the box, there are a few traits that need to be implemented in order
//! to get started.
//!
//! Firstly, does your codebase own the type you want to use?
//! [Yes? Great! Skip Ahead](#required-custom-type-traits)
//!
//! ## External Type Conflicts
//!
//! Rust doesn't allow us to implement traits for types if we don't own at least
//! one of them for fear that a future upstream change will introduce ambiguity.
//! The solution to which is the
//! [New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html).
//!
//! So, we need to proxy whatever type we want to work with.
//!
//! ```ignore
//! use chrono::{DateTime, Utc};
//! pub struct MyDateTime(DateTime<Utc>);
//! // implement all the things...
//! ```
//!
//! ## Required Custom Type Traits
//!
//! Interval types use a handful of traits to fully define interval and set
//! behavior.
//!
//! * [`Domain`]
//! > The `Domain` trait serves one purpose -- to distinguish between types
//! > that represent **discrete** vs **continuous** data.
//! >
//! > This allows us to do two important things:
//! > 1. Normalize discrete sets so that there is only a single valid
//! >    representations of each possible `Set`.
//! >    eg. **[1, 9]** == (0, 10) == (0, 9] == [1, 10).
//! > 2. Properly test the adjacency of sets (union / merge).
//! >
//! > The method [`try_adjacent`](Domain::try_adjacent) is the
//! > mechanism by which both of these goals is accomplished. Implementations
//! > for **continuous** types should simply return None.
//! >
//! > The macro [`continuous_domain_impl`](crate::continuous_domain_impl) exists for exactly this purpose.
//!
//! * [`Zero`]
//! > The `Zero` trait is necessary for the [`measure`](crate::measure) module,
//! > specifically in handling the empty set. It is just a re-export from [`num_traits`].
//!
//! * [`Countable`](crate::measure::Countable)
//! > The `Countable` trait is only relevant to **discrete** data types. It is
//! > the mechanism by which a data type can say how many distinct values fall
//! > between some bounds of that type. There is a macro
//! > [`default_countable_impl`](crate::default_countable_impl) which uses [`try_adjacent`](Domain).
//!
//! * [`Add`](core::ops::Add) and [`Sub`](core::ops::Sub)
//! > The `Add` and `Sub` traits are used by the [`measure`](crate::measure) module, and could
//! > be used elsewhere in the future. Presumably these are already implemented
//! > for most types that one would want to use as bounds of a Set.
//!
//! ## Putting it all together
//!
//! ```
//! use core::ops::{Add, Sub};
//! use intervalsets_core::numeric::{Domain, Zero};
//! use intervalsets_core::measure::Countable;
//! use intervalsets_core::bound::Side;
//!
//! // minimum required is: Clone, PartialEq, PartialOrd
//! #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! pub struct MyInt(i32);
//!
//! impl Domain for MyInt {
//!     fn try_adjacent(&self, side: Side) -> Option<Self> {
//!         Some(match side {
//!             Side::Left => Self(self.0 - 1),
//!             Side::Right => Self(self.0 + 1),
//!         })
//!     }
//! }
//!
//! impl Zero for MyInt {
//!     fn zero() -> Self {
//!         Self(0)
//!     }
//!
//!     fn is_zero(&self) -> bool {
//!         self.0 == 0
//!     }
//! }
//!
//! // The default_countable_impl macro would work here just fine.
//! // This would be omitted for a continuous data type.
//! impl Countable for MyInt {
//!     type Output = Self;
//!     fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
//!         Some(Self(right.0 - left.0 + 1))
//!     }
//! }
//!
//! impl Add for MyInt {
//!     type Output = Self;
//!     fn add(self, rhs: Self) -> Self {
//!         Self(self.0 + rhs.0)
//!     }
//! }
//!
//! impl Sub for MyInt {
//!     type Output = Self;
//!     fn sub(self, rhs: Self) -> Self {
//!         Self(self.0 - rhs.0)
//!     }
//! }
//! ```

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

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
//! * [`Element`]
//! > The `Element` trait serves one purpose -- to distinguish between types
//! > that represent **discrete** vs **continuous** data.
//! >
//! > This allows us to do two important things:
//! > 1. Normalize discrete sets so that there is only a single valid
//! >    representations of each possible `Set`.
//! >    eg. **[1, 9]** == (0, 10) == (0, 9] == [1, 10).
//! > 2. Properly test the adjacency of sets (union / merge).
//! >
//! > The method [`try_adjacent`](Element::try_adjacent) is the
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
//! > [`default_countable_impl`](crate::default_countable_impl) which uses [`try_adjacent`](Element).
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
//! use intervalsets_core::numeric::{Element, Zero};
//! use intervalsets_core::measure::Countable;
//! use intervalsets_core::bound::Side;
//!
//! // minimum required is: Clone, PartialEq, PartialOrd
//! #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! pub struct MyInt(i32);
//!
//! impl Element for MyInt {
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

pub use num_traits::{CheckedSub, Zero};

//use ordered_float::{NotNan, OrderedFloat};
use crate::bound::Side;

/// Defines the data types whose elements make up a Set.
///
/// `try_adjacent` determines whether the elements are
/// treated as continuous or discrete data.
///
/// # Design: `PartialOrd`, not `Ord`
///
/// `Element` deliberately requires only `PartialOrd`, **not** `Ord`.
/// Tightening this bound to `Ord` would exclude `f32` and `f64` (which
/// are `!Ord` because of NaN), and float support is one of the crate's
/// core value propositions — much of the crate's complexity exists to
/// keep floats in the domain. Don't tighten this.
///
/// The crate handles NaN at runtime via [`TryCmp`](crate::try_cmp::TryCmp)
/// (a blanket impl over `T: PartialOrd` that returns
/// [`TotalOrderError`](crate::error::TotalOrderError) when
/// `partial_cmp` returns `None`). Validating constructors (`new`,
/// `try_new`, `Deserialize`) call `try_cmp` and reject NaN. Operations
/// that benefit from a stronger guarantee (set-op traits like `Union`)
/// add `T: Ord` as a separate per-trait bound, so callers using
/// integer-only types pay no NaN-checking cost while float users still
/// get a working API. The verbose `T: Element + Ord + Clone + Zero`-style
/// bounds elsewhere are deliberate; that's the cost of the split.
pub trait Element: Sized + PartialEq + PartialOrd {
    fn try_adjacent(&self, side: Side) -> Option<Self>;
}

/// Automatically implements [`Element`] for a type.
///
/// Interval/Set types require generic storage types to implement
/// the [`Element`] trait. It's primary function is to normalize **disrete**
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
/// ```
#[macro_export]
macro_rules! continuous_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Element for $t {
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
            impl $crate::numeric::Element for $t {
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

/// Computes the midpoint (average) of two values.
///
/// The midpoint is the value equidistant from both inputs. For numeric
/// types this is conceptually `(self + other) / 2`, computed in a way
/// that does not overflow at the bounds of the type.
///
/// # Contract
///
/// Implementations should uphold the following:
///
/// 1. **No overflow.** The computation must not overflow even when
///    `self + other` would. Conceptually, evaluate as if in a
///    sufficiently-large arithmetic domain, then map back into `Self`.
/// 2. **Commutativity.** `a.midpoint(b) == b.midpoint(a)`.
/// 3. **Boundedness.** When inputs are comparable, the result lies
///    between them: `min(a, b) <= a.midpoint(b) <= max(a, b)`.
///
/// # Rounding
///
/// For std primitives this trait delegates to the inherent
/// `<T>::midpoint` method, so rounding behavior matches std exactly:
///
/// - **Integers** — rounded toward zero.
/// - **Floats** (`f32`, `f64`) — computed as if in extended precision,
///   then rounded to the nearest representable value
///   (round-to-nearest-even).
///
/// Custom types choose the rounding semantics appropriate to their
/// domain; this trait does not prescribe one. Downstream impls are
/// expected to document their own rounding, error, and edge-case
/// behavior.
///
/// # Errors
///
/// The associated [`Error`](Midpoint::Error) type lets implementations
/// surface failure modes specific to the type:
///
/// - For total-ordered types where every pair has a well-defined
///   midpoint (e.g. integers), `Error` is typically
///   [`core::convert::Infallible`] and the impl never returns `Err`.
/// - For partial-ordered types (e.g. `f32`/`f64`), the impl may reject
///   inputs that are degenerate as midpoint endpoints. The float impls
///   in this crate return [`MidpointError`](crate::error::MidpointError)
///   when either input is non-finite (NaN, +∞, or −∞) — note that ±∞
///   are excluded even though they are comparable, because they have
///   no well-defined midpoint.
pub trait Midpoint: Sized {
    type Error;
    fn midpoint(self, other: Self) -> Result<Self, Self::Error>;
}

macro_rules! integer_midpoint_delegate_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Midpoint for $t {
                type Error = ::core::convert::Infallible;

                /// Infallible: std's inherent integer `midpoint` is
                /// defined for every value in the type's range and
                /// cannot overflow.
                #[inline]
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
                    Ok(<$t>::midpoint(self, other))
                }
            }
        )+
    }
}

integer_midpoint_delegate_impl!(u8, u16, u32, u64, u128, usize);
integer_midpoint_delegate_impl!(i8, i16, i32, i64, i128, isize);

macro_rules! float_midpoint_delegate_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Midpoint for $t {
                type Error = $crate::error::MidpointError;

                /// Delegates to the inherent float `midpoint` method,
                /// which avoids spurious overflow/underflow at extremes.
                ///
                /// # Errors
                ///
                /// Returns [`MidpointError`](crate::error::MidpointError)
                /// when either input is non-finite — NaN, +∞, or −∞.
                /// Infinities are rejected even though they are
                /// comparable, because their midpoint is not
                /// well-defined as a finite endpoint.
                #[inline]
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
                    if !self.is_finite() || !other.is_finite() {
                        return Err($crate::error::MidpointError);
                    }
                    Ok(<$t>::midpoint(self, other))
                }
            }
        )+
    }
}

float_midpoint_delegate_impl!(f32, f64);

#[cfg(test)]
mod tests {
    use super::*;
    //use quickcheck_macros::quickcheck;

    #[test]
    fn test_adjacent() {
        assert_eq!(10.try_adjacent(Side::Right).unwrap(), 11);
        assert_eq!(11.try_adjacent(Side::Left).unwrap(), 10);
    }

    // force resolution through trait
    fn get_midpoint<T: Midpoint>(a: T, b: T) -> Result<T, T::Error> {
        a.midpoint(b)
    }

    #[quickcheck]
    fn quickcheck_midpoint_i32(a: i32, b: i32) {
        let expected = (((a as i64) + (b as i64)) / 2) as i32;
        assert_eq!(get_midpoint(a, b).unwrap(), expected);
    }

    #[quickcheck]
    fn quickcheck_midpoint_f32(a: f32, b: f32) {
        if !a.is_finite() || !b.is_finite() {
            return;
        }
        // widen so the reference sum can't overflow f32 range
        let expected = (((a as f64) + (b as f64)) / 2.0) as f32;
        assert_eq!(get_midpoint(a, b).unwrap(), expected);
    }
}

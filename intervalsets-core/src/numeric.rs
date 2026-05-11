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
//! > The `Element` trait serves two purposes:
//! >
//! > 1. **Distinguish discrete vs continuous data** via
//! >    [`try_adjacent`](Element::try_adjacent). This lets the crate
//! >    normalize discrete sets to a canonical bit-pattern (e.g.
//! >    `[1, 9]` == `(0, 10)` == `(0, 9]` == `[1, 10)`) and test
//! >    adjacency for union/merge. Continuous-type impls return `None`.
//! >
//! > 2. **Validate (and optionally normalize) bound limits** via
//! >    [`validate`](Element::validate). Library float types reject
//! >    `±INF` and `NaN` here so that a `FiniteBound<f*>` is always a
//! >    finite real number; user types override to enforce their own
//! >    predicate. The default impl rejects only NaN-style
//! >    `partial_cmp(&self) == None` values, which is correct for
//! >    every discrete type with no intrinsic infinities.
//! >
//! > The macros [`continuous_domain_impl`](crate::continuous_domain_impl)
//! > and the integer-domain helpers wire `try_adjacent` for the common
//! > shapes; `validate` is overridden separately when needed (see the
//! > `Extended<T>` worked example below).
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
//!     // `validate` defaults to `partial_cmp(&self).is_some()` — fine for
//!     // a discrete `i32`-backed type with no intrinsic infinities.
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
//!
//! ## Worked example: `validate` override for a type with intrinsic infinities
//!
//! User types that carry their own `±INF` (or any other "comparable but
//! not a valid finite-bound limit" value) override `validate` to reject
//! them. Construction sites that funnel through
//! [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new) then
//! surface the rejection as
//! [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit).
//!
//! ```
//! use intervalsets_core::numeric::Element;
//! use intervalsets_core::bound::Side;
//!
//! /// A toy "extended real" type that admits ±∞ as comparable values.
//! #[derive(Copy, Clone, PartialEq, PartialOrd)]
//! enum Extended {
//!     NegInf,
//!     Finite(i32),
//!     PosInf,
//! }
//!
//! impl Element for Extended {
//!     fn try_adjacent(&self, side: Side) -> Option<Self> {
//!         match (self, side) {
//!             (Extended::Finite(n), Side::Left) => Some(Extended::Finite(n - 1)),
//!             (Extended::Finite(n), Side::Right) => Some(Extended::Finite(n + 1)),
//!             // ±∞ have no adjacent finite element in this model.
//!             _ => None,
//!         }
//!     }
//!
//!     /// Reject the intrinsic infinities — they're comparable, but
//!     /// they're not valid finite-bound limits.
//!     fn validate(self) -> Option<Self> {
//!         match self {
//!             Extended::NegInf | Extended::PosInf => None,
//!             Extended::Finite(_) => Some(self),
//!         }
//!     }
//! }
//! ```

pub use num_traits::{Bounded, CheckedSub, NumCast, ToPrimitive, Zero};

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

    /// Validate (and optionally normalize) `self` as a `FiniteBound` value.
    ///
    /// Returns `Some(v)` to accept `self` (where `v` is the canonical
    /// form to store — possibly the same value), or `None` to reject.
    /// A `None` return collapses to
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)
    /// at construction sites that funnel through
    /// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new).
    ///
    /// # Default behavior
    ///
    /// The default impl delegates to `self.partial_cmp(&self)`, which
    /// rejects values that are incomparable to themselves — i.e. NaN.
    /// This preserves the historical NaN-rejection behavior the crate's
    /// `try_cmp`-based validators relied on.
    ///
    /// # When to override
    ///
    /// Override when the type carries values that are comparable but
    /// not valid finite bound limits — most commonly intrinsic
    /// infinities. Library floats (`f32`, `f64`, `OrderedFloat<f*>`,
    /// `NotNan<f*>`) override to reject non-finite values via
    /// `is_finite()`. Discrete types (integers, `BigInt`/`BigUint`,
    /// `Decimal`, `BigDecimal`, `Fixed*`) keep the default — none have
    /// intrinsic infinities, and NaN is either nonexistent or already
    /// covered by `partial_cmp`.
    fn validate(self) -> Option<Self> {
        self.partial_cmp(&self).map(|_| self)
    }
}

/// Automatically implements [`Element`] for a type.
///
/// Interval/Set types require generic storage types to implement
/// the [`Element`] trait. It's primary function is to normalize **discrete**
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

// Native floats override `validate` to reject non-finite (NaN, ±INF).
// `continuous_domain_impl!` is reserved for types whose default
// `validate` is already correct (e.g. `BigDecimal`).
macro_rules! float_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Element for $t {
                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }

                #[inline]
                fn validate(self) -> Option<Self> {
                    self.is_finite().then_some(self)
                }
            }
        )+
    }
}

float_domain_impl!(f32, f64);

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
/// The associated [`Error`](Midpoint::Error) type is a user-extension
/// hook. Every library-provided impl uses
/// [`core::convert::Infallible`] except [`Decimal`](rust_decimal::Decimal),
/// whose bounded precision means rounding the two halves can push their
/// sum out of range; that impl returns [`MathError::Range`](crate::error::MathError::Range).
///
/// In-tree contract: values stored in any in-tree set type
/// (`FiniteInterval`, `HalfInterval`, `EnumInterval`, `Interval`) are
/// validated finite at construction (see [`Element::validate`]). When
/// `T::midpoint` is reached through the set-level
/// [`midpoint`](crate::sets::FiniteInterval::midpoint) accessors, the
/// inputs are guaranteed finite, so library impls succeed.
///
/// Direct callers passing arbitrary values (e.g. `f32::midpoint(NAN, 0.0)`)
/// inherit std's `f*::midpoint` semantics — typically a NaN-tainted
/// result rather than an error.
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
                type Error = ::core::convert::Infallible;

                /// Infallible by contract: values stored in any in-tree
                /// set type are validated finite at construction
                /// ([`Element::validate`](crate::numeric::Element::validate)
                /// rejects `±INF`/`NaN`), so the set-level
                /// [`midpoint`](crate::sets::FiniteInterval::midpoint)
                /// accessors only ever reach this impl with finite
                /// inputs. Delegates to the inherent float `midpoint`,
                /// which avoids spurious overflow/underflow at extremes.
                ///
                /// Direct callers passing non-finite values bypass the
                /// in-tree contract and inherit std's `f*::midpoint`
                /// semantics (typically a NaN-tainted result).
                #[inline]
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
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

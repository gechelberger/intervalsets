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
//! > The `Element` trait declares **what category** of element the type
//! > is (via [`Kind`](Element::Kind) — [`DiscreteKind`] or
//! > [`ContinuousKind`]), **what its natural measure is** (via
//! > [`Measure`](Element::Measure)), and the primitives needed to
//! > compute that measure ([`try_adjacent`](Element::try_adjacent),
//! > [`try_measure_finite`](Element::try_measure_finite)). It also
//! > carries [`validate`](Element::validate), the gatekeeper for
//! > construction-time bound validity.
//!
//! * [`Zero`]
//! > The `Zero` trait is necessary for the [`measure`](crate::measure)
//! > module, specifically in handling the empty set. It is just a
//! > re-export from [`num_traits`].
//!
//! * [`Add`](core::ops::Add) and [`Sub`](core::ops::Sub)
//! > Used by some operations. Presumably already implemented for most
//! > types that one would want to use as bounds.
//!
//! ## Putting it all together
//!
//! ```
//! use core::ops::{Add, Sub};
//! use intervalsets_core::bound::Side;
//! use intervalsets_core::numeric::{
//!     default_discrete_count_inclusive, DiscreteKind, Element, Zero,
//! };
//!
//! // minimum required: Clone, PartialEq, PartialOrd
//! #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! pub struct MyInt(i32);
//!
//! impl num_traits::CheckedSub for MyInt {
//!     fn checked_sub(&self, rhs: &Self) -> Option<Self> {
//!         self.0.checked_sub(rhs.0).map(MyInt)
//!     }
//! }
//!
//! impl Element for MyInt {
//!     type Kind = DiscreteKind;
//!     type Measure = MyInt;
//!
//!     fn try_adjacent(&self, side: Side) -> Option<Self> {
//!         Some(match side {
//!             Side::Left => Self(self.0 - 1),
//!             Side::Right => Self(self.0 + 1),
//!         })
//!     }
//!
//!     fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
//!         default_discrete_count_inclusive(left, right)
//!     }
//! }
//!
//! impl Zero for MyInt {
//!     fn zero() -> Self { Self(0) }
//!     fn is_zero(&self) -> bool { self.0 == 0 }
//! }
//!
//! impl Add for MyInt {
//!     type Output = Self;
//!     fn add(self, rhs: Self) -> Self { Self(self.0 + rhs.0) }
//! }
//!
//! impl Sub for MyInt {
//!     type Output = Self;
//!     fn sub(self, rhs: Self) -> Self { Self(self.0 - rhs.0) }
//! }
//!
//! impl intervalsets_core::ops::math::TryAdd for MyInt {
//!     type Output = Self;
//!     type Error = intervalsets_core::error::MathError;
//!     fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
//!         self.0.checked_add(rhs.0)
//!             .map(MyInt)
//!             .ok_or(intervalsets_core::error::MathError::Range)
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
//! use intervalsets_core::bound::Side;
//! use intervalsets_core::numeric::{DiscreteKind, Element};
//! use intervalsets_core::ops::math::TryAdd;
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
//!     type Kind = DiscreteKind;
//!     type Measure = u64;
//!
//!     fn try_adjacent(&self, side: Side) -> Option<Self> {
//!         match (self, side) {
//!             (Extended::Finite(n), Side::Left) => Some(Extended::Finite(n - 1)),
//!             (Extended::Finite(n), Side::Right) => Some(Extended::Finite(n + 1)),
//!             // ±∞ have no adjacent finite element in this model.
//!             _ => None,
//!         }
//!     }
//!
//!     fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
//!         match (left, right) {
//!             (Extended::Finite(l), Extended::Finite(r)) => {
//!                 i32::try_measure_finite(l, r)
//!             }
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

mod element;
mod midpoint;
mod saturating;

pub use element::{
    default_discrete_count_inclusive, ContinuousElement, ContinuousKind, DiscreteElement,
    DiscreteKind, Element,
};
pub use midpoint::Midpointable;

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

pub mod element;
pub mod midpoint;
pub mod saturating;

pub use element::Element;
pub use midpoint::Midpoint;

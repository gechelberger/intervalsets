//! intervalsets-core is a no-std/no-alloc library for working with intervals as sets.
//!
//! # Overview
//!
//! intervalsets-core provides basic intervals and set operations
//! for a no-alloc environment.
//! [intervalsets](https://docs.rs/intervalsets/latest/intervalsets/) builds upon
//! this crate for additional functionality in an std environment.
//!
//! An **interval** is defined as a connected set of numbers with no gaps, of which
//! there are 5 general cases for the Reals.
//!
//! | Case          | Base Implementation | Set Notation (Closed) | [`Range`](core::ops::Range) Equivalents |
//! |---------------|---------------------|-----------------------|------------------|
//! | Empty Set     | `FiniteInterval`    | {}                    | `Range` |
//! | Fully Bounded | `FiniteInterval`    | { x \| a <= x <= b }  | `Range`, `RangeInclusive` |
//! | Left Bounded  | `HalfInterval`      | { x \| a <= x }       | `RangeFrom` |
//! | Right Bounded | `HalfInterval`      | { x \| x <= b }       | `RangeTo`, `RangeToInclusive` |
//! | Unbounded     | `EnumInterval`      | { x \| x ∈ DataType } | `RangeFull` |
//!
//! Some performance gains can be achieved by working directly with the base
//! implementations, though it is often simpler to work with [`EnumInterval`]
//! which, as the name implies, is an enum of the base implementations and covers
//! all five cases. Any **set operation** on an `EnumInterval` should return another
//! `EnumInterval`. This is not the case for operations on [`FiniteInterval`] or
//! [`HalfInterval`] which will return the most narrow base implementation possible.
//!
//! Operations that might return disjoint sets (such as Union and Complement)
//! are provided in the main
//! [intervalsets](https://docs.rs/intervalsets/latest/intervalsets/) crate.
//!
//! # Limitations
//!
//! This family of crates is intended to provide robust, general implementations
//! of intervals with a convenient `Set` based api, and is pluggable with
//! user provided data-types. While attention has been given to performance,
//! there are many optimizations that can not be included without a loss of generality.
//!
//! Currently [interval arithmetic](https://en.wikipedia.org/wiki/Interval_arithmetic)
//! is not supported, and while it may be in the future, it will never be as
//! performant as a specialized library like [inari](https://docs.rs/inari/latest/inari/).
//!
//! # Getting Started
//!
//! **Intervals** are generally constructed via the a [`factory`]` trait which are
//! implemented for each base interval type. Manipulation is provided via
//! [set operations](ops).
//!
//! ```
//! use intervalsets_core::prelude::*;
//!
//! let a = EnumInterval::closed(0, 10);
//!
//! assert!(a.contains(&5));
//! assert!(a.contains(&a));
//! assert!(a.contains(&EnumInterval::closed(4, 6)));
//! assert!(!a.contains(&EnumInterval::closed_unbound(5)));
//!
//! assert!(a.intersects(&EnumInterval::closed(10, 20)));
//! assert!(!a.intersects(&EnumInterval::closed(11, 20)));
//!
//! let merged = a.try_merge(EnumInterval::closed(5, 20)).unwrap();
//! assert_eq!(merged, EnumInterval::closed(0, 20));
//!
//! let merge_failed = a.try_merge(EnumInterval::closed(15, 20));
//! assert_eq!(merge_failed, None);
//!
//! let intersected = a.intersection(EnumInterval::closed(-5, 5));
//! assert_eq!(intersected, EnumInterval::closed(0, 5));
//!
//! let rebound_left = a.with_left_closed(7);
//! assert_eq!(rebound_left, EnumInterval::closed(7, 10));
//!
//! let rebound_right = a.with_right_closed(3);
//! assert_eq!(rebound_right, EnumInterval::closed(0, 3));
//!
//! let hull = EnumInterval::try_hull([10, 8, 0, 6, 4, 2]).unwrap();
//! assert_eq!(hull, a);
//!
//! let empty = a.intersection(EnumInterval::closed(20, 30));
//! assert!(empty.is_empty());
//!
//! let (left, right) = a.split(5, Side::Left);
//! assert_eq!(left, EnumInterval::closed(0, 5));
//! assert_eq!(right, EnumInterval::closed(6, 10));
//!
//! let width: Measurement<_> = a.width();
//! assert_eq!(width.finite(), 10);
//!
//! let count: Measurement<_> = a.count();
//! assert_eq!(count.finite(), 11);
//!
//! assert_eq!(format!("{}", a), "[0, 10]");
//!
//! // intervals have a total order as long as T: Ord:
//! let a = EnumInterval::empty();
//! let b = EnumInterval::unbound_closed(10);
//! let c = EnumInterval::closed(0, 10);
//! let d = EnumInterval::closed_unbound(0);
//! assert!(a < b && b < c && c < d);
//! ```
//!
//! # Invariants
//!
//! For an interval or set to be valid it must satisfy certain invariants.
//! These are validated on construction and panic if violated (or return an
//! error from the fallible (try_*) api).
//!
//! 1. Discrete types are always normalized to closed form so that there is only
//!    a single valid bit-pattern for each possible `Set`.
//! 2. lhs <= rhs. All non-empty sets have a left and right hand side, though
//!    they may be implicit and/or unbounded.
//! 3. A FiniteBound's limit value must be a member of some set S ⊆ T where
//!    T is the set of the underlying data-type which may be partially ordered,
//!    but S has a strict total order. (S is a chain)
//!
//! # Foot guns
//!
//! ## Normalized Conversions
//!
//! Most of the time normalization is transparent to the user, but it is a
//! potential source of error, particularly when converting types that have
//! implicit open bounds.
//!
//! ```
//! use intervalsets_core::prelude::*;
//!
//! let discrete = EnumInterval::open(0, 10);
//! assert_eq!(discrete.lval(), Some(&1));
//! assert_eq!(discrete.rval(), Some(&9));
//! assert_eq!(discrete, (0, 10).into());
//! assert_eq!(discrete, [1, 9].into());
//! ```
//!
//! ## Floating Point Types
//!
//! Making [`Ord`] a trait bound for most of this crate's APIs would
//! elimenate a whole class of errors (invariant #3), but floats come with a
//! whole host of complexities regardless.
//!
//! * `NAN` is not part of the default ordering, though there is a `total_cmp`
//!   available now.
//! * rounding errors can cause issues with testing values near a set bound.
//! * `FiniteBound(f32::INFINITY)` and `FiniteBound(f32::NEG_INFINITY)`are both
//!   valid syntax, though all manner of headache semantically speaking.
//!
//! Sometimes, floats are still the right tool for the job, and it is left to the
//! user to choose the right approach for the given problem. Fixed precision
//! decimal types like `rust_decimal` do side step some pitfalls.
//!
//! ## Fallibility
//!
//! Every fallible operation has both a panicking and a `try_*` form.
//! The panicking forms exist for ergonomics on properly-ordered types;
//! the `try_*` forms return `Result<_, Error>` and never panic.
//!
//! | Concern | Panicking | Fallible |
//! |---|---|---|
//! | Constructing an interval | `FiniteInterval::new`, factory methods like `closed`/`open` | [`FiniteInterval::try_new`], `try_closed`/`try_open` |
//! | Arithmetic operators | `+` `-` `*` `/` (require `T: Ord`) | [`ops::math::TryAdd`], [`ops::math::TrySub`], [`ops::math::TryMul`], [`ops::math::TryDiv`] |
//! | Convex hull | [`ops::ConvexHull::hull`] | [`ops::ConvexHull::try_hull`] |
//! | Splitting | [`ops::Split::split`] | [`ops::Split::try_split`] |
//! | Rebounding | [`ops::Rebound::with_left`]/[`ops::Rebound::with_right`] | [`ops::Rebound::try_with_left`]/[`ops::Rebound::try_with_right`] |
//! | Counting | [`measure::Count::count`] | [`measure::Count::try_count`] |
//! | Categorizing | [`FiniteInterval::category`] | [`FiniteInterval::try_category`] |
//!
//! ```
//! use intervalsets_core::prelude::*;
//! use intervalsets_core::bound::FiniteBound;
//!
//! // The factory `open` / `try_open` are coercive: crossed bounds and
//! // empty-by-construction inputs both produce empty silently.
//! let x = FiniteInterval::open(1.0, 0.0);
//! assert_eq!(x, FiniteInterval::empty());
//!
//! let x = FiniteInterval::try_open(1.0, 0.0).unwrap();
//! assert_eq!(x, FiniteInterval::empty());
//!
//! // NaN panics on the panicking path, errors on the fallible path.
//! let result = std::panic::catch_unwind(|| {
//!     FiniteInterval::open(f32::NAN, 0.0);
//! });
//! assert!(result.is_err());
//!
//! let x = FiniteInterval::try_open(f32::NAN, 0.0);
//! assert!(x.is_err());
//!
//! // The strict primitives (FiniteInterval::new / try_new) reject
//! // crossed bounds outright. Use them when malformed input should be
//! // a hard error rather than coerced.
//! let x = FiniteInterval::try_new(
//!     FiniteBound::closed(10),
//!     FiniteBound::closed(0),
//! );
//! assert!(x.is_err());
//! ```
//!
//! Silent failures can still occur in operations that have a defensible
//! "empty set" interpretation for crossed bounds (e.g. `with_left`).
//! These are documented per-operation:
//!
//! ```
//! use intervalsets_core::prelude::*;
//! let interval = EnumInterval::closed(0, 10);
//!
//! let oops = interval
//!     .with_left_closed(20) // empty here (bounds violation + silent failure)
//!     .with_right(None);
//! assert_ne!(oops, EnumInterval::closed_unbound(20));
//! assert_eq!(oops, EnumInterval::empty());
//!
//! let fixed = interval.with_right(None).with_left_closed(20);
//! assert_eq!(fixed, EnumInterval::closed_unbound(20));
//! ```
//!
//! # Construction at boundaries
//!
//! Construction comes in two flavors at the user-facing API surface:
//! the **factory methods** (`Interval::open`, `try_closed`, etc.) are
//! **coercive** — they treat crossed bounds and empty-by-construction
//! inputs as `Empty`, only surfacing NaN as an error / panic. The
//! **primitive constructors** ([`FiniteInterval::new`],
//! [`FiniteInterval::try_new`]) are **strict** — they reject crossed
//! bounds outright with `InvalidBoundPair`. The strict primitives are
//! what `Deserialize` routes through (a well-formed serializer never
//! legitimately produces crossed bounds, so an error signals malformed
//! input); the coercive factories preserve ergonomics for the common
//! "give me the interval these bounds describe; empty if they don't
//! describe one" case.
//!
//! In full, the constructors form a layered escape hatch:
//!
//! - **Strict primitives** ([`FiniteInterval::new`],
//!   [`FiniteInterval::try_new`]): reject anything that isn't a
//!   well-formed `Bounded`. NaN → `Err(TotalOrderError)` (or panic).
//!   Crossed bounds → `Err(InvalidBoundPair)` (or panic). Used by
//!   `Deserialize`.
//!
//! - **Coercive factories**
//!   (`Interval::open`/`closed`/etc, `try_open`/`try_closed`/etc,
//!   [`FiniteInterval::try_new_or_empty`]): crossed bounds and
//!   empty-by-construction inputs → `Empty`, NaN still propagates.
//!   This is the user-facing default and the right choice for code
//!   producing bounds via computation (Range conversions, rebound,
//!   split-at-boundary).
//!
//! - **Pre-normalized** ([`FiniteInterval::new_assume_normed`],
//!   [`FiniteInterval::try_new_assume_normed`]): caller has already
//!   normalized; collapse-or-build. Used internally by intersection
//!   and other ops where bounds are computed from already-valid
//!   inputs and may legitimately cross.
//!
//! - **Bypass** ([`FiniteInterval::new_assume_valid`]): caller
//!   asserts the preconditions; no checking. `forbid(unsafe_code)`
//!   means a violation produces incorrect results, never undefined
//!   behavior.
//!
//! The two-tier split (strict primitives + coercive factories) keeps
//! the strict semantic available where it matters — at trust
//! boundaries — without forcing every ergonomic factory call to plumb
//! a `Result` to handle "the bounds describe an empty set."
//!
//! # Deserialization
//!
//! `Deserialize` impls route through the strict constructor path: a
//! well-formed serializer never emits a swapped-order `Bounded`,
//! a non-canonical discrete interval, an empty stored in an
//! `IntervalSet`, an unsorted set, or connecting intervals in a set.
//! Any such payload did not come from us, and silently accepting it
//! would mask producer bugs. NaN, swapped-order `Bounded`,
//! stored-empty, unsorted-set, and connecting-set inputs all return
//! `Err`.
//!
//! The deserialization boundary is a trust boundary and gets the
//! strict treatment unconditionally. Callers that want crossed →
//! empty coercion can opt in via the appropriate constructor at the
//! callsite (see "Construction at boundaries" above).
//!
//! # Features
//! intervalsets-core has several feature flags that modify capabilities. By
//! default, none are enabled.
//!
//! ## testing
//!
//! * arbitrary: implement the [`Arbitrary`](::arbitrary::Arbitrary) trait
//! * quickcheck: implement the [`Arbitrary`](::quickcheck::Arbitrary) trait
//!
//! ## storage
//!
//! * ordered-float: wrappers that provide a total order for floating point types.
//! * rust_decimal: fixed precision total ordered decimals
//! * bigdecimal: arbitrary precision total ordered decimals
//! * num-bigint: arbitrary sized integers
//!
//! ## serialization
//! * serde: implement [`Serialize`](::serde::Serialize), [`Deserialize`](::serde::Deserialize).
//!   `Deserialize` requires `T: Element` and rejects NaN, swapped-order
//!   `Bounded` pairs, and other invariant violations on the interval types
//!   ([`FiniteInterval`],
//!   [`HalfInterval`],
//!   [`EnumInterval`]). The `bound::ord::*` helper
//!   types do not derive serde traits.
//!
//! # Diving Deeper
//! * [Implement custom storage data types](numeric)
//! * [Adapt unsupported data types with factory converters](factory::Converter)
//!
#![no_std]
#![deny(bad_style)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![forbid(unsafe_code)]
//#![deny(unused)]

//#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
//#![warn(missing_docs)]

pub mod bound;
pub mod numeric;

pub mod error;

mod feat;
pub mod sets;
pub use sets::{EnumInterval, FiniteInterval, HalfInterval};

pub mod disjoint;

pub mod ops;

pub mod factory;

pub mod category;
pub mod measure;
pub mod try_cmp;

mod display;
mod from;

mod empty;
pub use empty::MaybeEmpty;

/// commonly used imports
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::bound::{BoundType, FiniteBound, SetBounds, Side};
    pub use crate::empty::MaybeEmpty;
    //pub use crate::error::Error;
    pub use crate::factory::traits::*;
    pub use crate::measure::{Count, Measurement, Width};
    pub use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};
    pub use crate::ops::*;
    pub use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};
}

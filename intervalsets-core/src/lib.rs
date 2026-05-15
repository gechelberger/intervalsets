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
//! let merged = a.merge_connected(EnumInterval::closed(5, 20)).unwrap();
//! assert_eq!(merged, EnumInterval::closed(0, 20));
//!
//! let merge_failed = a.merge_connected(EnumInterval::closed(15, 20));
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
//! // `.measure()` returns the natural measure of the type — cardinality
//! // for discrete T, Lebesgue width for continuous T. For `i32`, the
//! // measure type widens to `u64` (stepwise widening).
//! let m: Extent<u64> = a.measure();
//! assert_eq!(m.finite(), 11);
//!
//! // For diameter `sup − inf` on any T (in T's native type), use Span.
//! assert_eq!(a.span().unwrap().finite(), 10_i32);
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
//! For an interval or set to be valid it must satisfy four invariants,
//! which split into two pairs:
//!
//! ## Canonicalization (one bit-pattern per logical set)
//!
//! 1. **Empty-canonical** — the empty set is represented only by the
//!    `Empty` discriminant; a `Bounded(lhs, rhs)` is never used to
//!    encode it. Inputs that describe an empty set are either
//!    rejected (strict constructors) or collapsed to `Empty`
//!    (coercive constructors).
//!
//! 2. **Discrete-normalized** — for discrete `T` (those with
//!    [`Element::try_adjacent`](crate::numeric::Element::try_adjacent)
//!    returning `Some(_)`), bounds are stored in **closed** form.
//!    `(0, 10)` for `i32` normalizes to `[1, 9]` so the same set has
//!    a single valid bit-pattern.
//!
//! ## Well-formedness (the contained pair makes sense)
//!
//! 3. **Limit-valid** — each [`FiniteBound`](crate::bound::FiniteBound)'s
//!    limit value must lie in some totally-ordered subdomain `S ⊆ T`.
//!    `T` may be only partially ordered (`f32`, user types), but `S`
//!    is a chain. Library float types reject `NaN` and `±INF`; user
//!    types enforce their own predicate via
//!    [`Element::validate`](crate::numeric::Element::validate).
//!
//! 4. **Ordered** — for any non-empty `Bounded(lhs, rhs)`,
//!    `lhs.value() <= rhs.value()`, with equality requiring both
//!    bounds to be `Closed` (an open-open pair at the same point
//!    describes the empty set and collapses to `Empty` per (1)).
//!
//! Invariants are enforced on construction. Every public entry
//! point is **strict by default**: malformed input — crossed bounds,
//! invalid limit — produces `Err` on the fallible path and panic on
//! the panicking sibling. Coercive ("crossed → `Empty`") behavior is
//! reachable explicitly via the
//! [`SatisfyFiniteInterval`](crate::factory::SatisfyFiniteInterval) /
//! [`TrySatisfyFiniteInterval`](crate::factory::TrySatisfyFiniteInterval)
//! trait family at the `FiniteBound`-taking layer. See
//! "[Construction at boundaries](#construction-at-boundaries)" below
//! for the full layered model.
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
//! * `±INF` and `NaN` are rejected at
//!   [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new)
//!   (and the `try_closed` / `try_open` aliases) — the only
//!   construction path. The panicking convenience ctors
//!   `FiniteBound::new` / `closed` / `open` delegate to `try_new`
//!   and panic on rejection. There is no validation bypass.
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
//! | Arithmetic operators | `+` `-` `*` `/` (panicking sugar over `try_*`) | [`ops::math::TryAdd`], [`ops::math::TrySub`], [`ops::math::TryMul`], [`ops::math::TryDiv`] |
//! | Convex hull | [`ops::ConvexHull::hull`] | [`ops::ConvexHull::try_hull`] |
//! | Splitting | [`ops::Split::split`] | [`ops::Split::try_split`] |
//! | Rebounding | [`ops::Rebound::with_left`]/[`ops::Rebound::with_right`] | [`ops::Rebound::try_with_left`]/[`ops::Rebound::try_with_right`] |
//! | Measure (cardinality / width) | [`measure::Measure::measure`] | [`measure::Measure::try_measure`] |
//! | Categorizing | [`FiniteInterval::category`] | [`FiniteInterval::try_category`] |
//!
//! ```
//! use intervalsets_core::prelude::*;
//! use intervalsets_core::bound::FiniteBound;
//!
//! // Factory methods are strict: crossed bounds error / panic.
//! let x = FiniteInterval::try_open(1.0, 0.0);
//! assert!(x.is_err());
//!
//! let result = std::panic::catch_unwind(|| {
//!     FiniteInterval::open(1.0, 0.0);
//! });
//! assert!(result.is_err());
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
//! // For coercive (crossed → Empty) semantics, use the
//! // SatisfyFiniteInterval trait family at the FiniteBound layer:
//! let x = FiniteInterval::satisfy_bounds(
//!     FiniteBound::open(1.0),
//!     FiniteBound::open(0.0),
//! );
//! assert_eq!(x, FiniteInterval::empty());
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
//! Construction is **strict by default at every public entry point**.
//! `Interval::closed(10, 0)` panics; `Interval::try_closed(10, 0)`
//! returns `Err(InvalidBoundPair)`. A typed value pair the user wrote
//! is treated as a producer assertion that those values describe a
//! real interval; silently degrading to `Empty` would mask producer
//! bugs.
//!
//! Coercive ("crossed → `Empty`") behavior is still available, but
//! only through explicit opt-in:
//!
//! - The
//!   [`SatisfyFiniteInterval`](crate::factory::SatisfyFiniteInterval)
//!   /
//!   [`TrySatisfyFiniteInterval`](crate::factory::TrySatisfyFiniteInterval)
//!   trait family exposes `satisfy_bounds(FiniteBound, FiniteBound)`
//!   / `try_satisfy_bounds(...)`. There is no value-taking surface;
//!   the caller constructs `FiniteBound`s explicitly. This matches
//!   the cases where coercion is the right answer (intersection-shape
//!   ops, computed bound pairs that may legitimately have no
//!   solution).
//!
//! - The `From<Range*>` family (`Range`, `RangeInclusive`, etc.)
//!   keeps coercive semantics because Rust's standard library Range
//!   types natively encode "empty when start ≥ end."
//!
//! In full, construction forms a layered model:
//!
//! - **Strict primitives** ([`FiniteInterval::new`],
//!   [`FiniteInterval::try_new`], [`HalfInterval::new`],
//!   [`HalfInterval::try_new`]): reject anything that isn't a
//!   well-formed pair. NaN / ±INF →
//!   `Err(InvalidElement)` (or panic). Crossed bounds →
//!   `Err(InvalidBoundPair)` (or panic). Used by `Deserialize`.
//!
//! - **Strict factories** (Family A:
//!   [`Factory`](crate::factory::Factory) /
//!   [`TryFiniteFactory`](crate::factory::TryFiniteFactory) /
//!   [`FiniteFactory`](crate::factory::FiniteFactory) and the
//!   half-bounded factory traits): the user-facing surface for
//!   ergonomic construction. `closed`, `open`, `try_closed`,
//!   `try_open`, `fully_bounded`, `try_fully_bounded`, etc. All
//!   strict. Built on top of the strict primitives.
//!
//! - **Coercive factory** (Family B:
//!   [`SatisfyFiniteInterval`](crate::factory::SatisfyFiniteInterval) /
//!   [`TrySatisfyFiniteInterval`](crate::factory::TrySatisfyFiniteInterval)):
//!   one operation only — `satisfy_bounds` / `try_satisfy_bounds`
//!   — at the `FiniteBound`-taking layer. Crossed bounds collapse
//!   to `Empty`; only error path is invalid limit.
//!
//! - **Bypass** ([`FiniteInterval::new_assume_valid`],
//!   [`HalfInterval::new_assume_valid`]): `#[doc(hidden)]`. Caller
//!   asserts all preconditions; no checking in release.
//!   `forbid(unsafe_code)` means a violation produces incorrect
//!   results, never undefined behavior.
//!
//! Tier-3 ("pre-normalized": per-bound trusted, pair satisfiability
//! is the question) has no public surface. Today's only consumer is
//! intersection, which keeps a private free `fn` for the operation.
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

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod bound;
pub mod cast;
pub mod numeric;

/// Compile-time-checked literal macro for [`EnumInterval`]. Parses a
/// string literal at expansion time in the same grammar as the runtime
/// [`FromStr`](crate::sets::EnumInterval#impl-FromStr-for-EnumInterval%3CT%3E)
/// impl; malformed input fails to build instead of panicking at
/// runtime.
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x: EnumInterval<i32> = enum_interval!("[0, 10]");
/// assert_eq!(x, EnumInterval::closed(0, 10));
///
/// let y: EnumInterval<f64> = enum_interval!("(.., 10.0)");
/// assert_eq!(y, EnumInterval::unbound_open(10.0));
///
/// let z: EnumInterval<i32> = enum_interval!("(.., ..)");
/// assert_eq!(z, EnumInterval::unbounded());
///
/// let e: EnumInterval<i32> = enum_interval!("{}");
/// assert_eq!(e, EnumInterval::empty());
///
/// // An optional second argument supplies a storage-type hint
/// // (emitted as a turbofish on the constructor call):
/// let u = enum_interval!("(.., ..)", i32);
/// assert_eq!(u, EnumInterval::<i32>::unbounded());
/// ```
pub use intervalsets_macros::enum_interval;

pub mod error;

mod feat;
pub mod sets;
pub use sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

pub mod ops;

pub mod factory;

pub mod category;
pub mod measure;
pub mod try_cmp;

mod display;
mod from;
mod parse;

mod empty;
pub use empty::MaybeEmpty;

/// commonly used imports
#[allow(unused_imports)]
pub mod prelude {
    pub use crate::bound::{BoundType, FiniteBound, SetBounds, Side};
    pub use crate::cast::{Cast, LossyCast, TryCast};
    pub use crate::empty::MaybeEmpty;
    pub use crate::enum_interval;
    pub use crate::factory::traits::*;
    pub use crate::measure::{Extent, Measure};
    pub use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};
    pub use crate::ops::*;
    pub use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};
}

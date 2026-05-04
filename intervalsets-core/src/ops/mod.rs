//! Set-algebra and arithmetic operations on intervals.
//!
//! Each submodule defines a single trait, implemented for
//! [`FiniteInterval`](crate::sets::FiniteInterval),
//! [`HalfInterval`](crate::sets::HalfInterval), and
//! [`EnumInterval`](crate::sets::EnumInterval) along with the
//! cross-type combinations needed for ergonomic chaining.
//!
//! # Output shape
//!
//! A set operation between two intervals can produce up to two
//! disjoint pieces (e.g. `[0, 5] ∪ [10, 15]`,
//! `[0, 10]'`). This crate is `no_std` and avoids allocation, so
//! multi-piece results are returned as
//! [`MaybeDisjoint`](crate::disjoint::MaybeDisjoint) rather than an
//! allocating set. The outer `intervalsets` crate layers an
//! arbitrary-piece `IntervalSet` on top for use cases that need it.
//!
//! # Panicking and fallible forms
//!
//! Operations that can fail at runtime (NaN bounds, arithmetic
//! overflow, invariant violations on `_assume_valid` inputs) are
//! offered in two flavors:
//!
//! - The panicking method is the default, ergonomic form.
//! - A `try_*` companion returns `Result<_, Error>` and never panics.
//!
//! Arithmetic gets dedicated traits for the fallible form
//! ([`TryAdd`], [`TrySub`], [`TryMul`], [`TryDiv`]) since the
//! panicking form is exposed through the `core::ops` operator
//! overloads. [`TryMerge`] is the fallible companion to interval
//! merging.

mod complement;
pub use complement::Complement;
mod connects;
pub use connects::{are_bounds_connected, Connects};
mod contains;
pub use contains::Contains;
mod difference;
pub use difference::Difference;
mod intersects;
pub use intersects::Intersects;

mod hull;
pub use hull::{convex_hull_into_ord_bound_impl, convex_hull_ord_bounded_impl, ConvexHull};
mod intersection;
pub use intersection::{Intersection, SetSetIntersection};
mod merged;
pub use merged::{MergeSortedByRef, MergeSortedByValue, TryMerge};
mod rebound;
pub use rebound::Rebound;
mod split;
pub use split::Split;
mod union;
pub use union::Union;

mod finite;
pub use finite::IntoFinite;

#[doc(hidden)]
pub mod math;
pub use math::{TryAdd, TryDiv, TryMul, TrySub};

mod util;

//! Set-algebra operations on intervals.
//!
//! The traits in this module operate on intervals as set-theoretic
//! values: complement, intersection, union, difference, convex hull,
//! containment, connectivity. Each submodule defines a single trait,
//! implemented for [`FiniteInterval`](crate::sets::FiniteInterval),
//! [`HalfInterval`](crate::sets::HalfInterval), and
//! [`EnumInterval`](crate::sets::EnumInterval) along with the
//! cross-type combinations needed for ergonomic chaining.
//!
//! Arithmetic on intervals lives in the [`math`] submodule.
//!
//! # Output shape
//!
//! A set operation between two intervals can produce up to two
//! disjoint pieces (e.g. `[0, 5] ∪ [10, 15]`, `[0, 10]'`). This crate
//! is `no_std` and avoids allocation, so multi-piece results are
//! returned as [`MaybeDisjoint`](crate::disjoint::MaybeDisjoint)
//! rather than an allocating set. The outer `intervalsets` crate
//! layers an arbitrary-piece `IntervalSet` on top for use cases that
//! need it.
//!
//! # Fallibility (TODO)
//!
//! The set-arithmetic side of [`math`] has a clear contract: the
//! infix operators are panicking and the `Try*` traits are their
//! `Result`-returning counterparts. The set-algebra traits in this
//! module are less consistent — some have `try_*` variants
//! ([`TryMerge`]; [`ConvexHull::try_hull`], [`Split::try_split`],
//! [`Rebound::try_with_left`]) while others don't, and the
//! conditions under which the panicking forms actually panic are
//! not uniformly documented. We need to settle on a contract and
//! audit the impls; until then, treat per-trait rustdoc as the
//! source of truth.

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

pub mod math;

mod util;

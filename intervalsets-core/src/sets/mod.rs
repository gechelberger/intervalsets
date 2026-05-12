//! The set types in this crate's hierarchy.
//!
//! - [`FiniteInterval`] — empty or `[lhs, rhs]`/`(lhs, rhs)`/mixed
//!   bounded on both sides.
//! - [`HalfInterval`] — bounded on exactly one side.
//! - [`EnumInterval`] — sum of the above plus `Unbounded`.
//! - [`MaybeDisjoint`] — at most two disjoint pieces; the
//!   set-valued result of operations like `Union` and complement
//!   that can produce a non-connected set without allocating.

mod disjoint;
mod enum_interval;
mod finite;
mod half;

pub use disjoint::MaybeDisjoint;
pub use enum_interval::EnumInterval;
pub use finite::FiniteInterval;
pub use half::HalfInterval;

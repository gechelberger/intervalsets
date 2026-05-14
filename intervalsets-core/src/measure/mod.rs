//! A Measure is a function of a set that gives a comparable size between sets.
//!
//! They must obey the following invariants:
//!
//! ```text
//! Let m(S) be our measure.
//!
//! 1) Monotonicity:
//!     If A is subset of B then m(A) <= m(B)
//!
//! 2) Subadditivity:
//!     If A0, A1, .. An is a countable set of possibly intersecting sets:
//!         m(A0 U A1 .. An) <= Sum { m(Ai) for i in 0..n }
//! ```
//!
//! The unified [`Measure`] trait returns the natural additive measure
//! of a set: cardinality on discrete element types, Lebesgue width on
//! continuous element types. The kind-projection lives at the
//! [`Element`](crate::numeric::Element) layer (`Output = T::Measure`).
//!
//! For "diameter on any T" (`sup − inf` regardless of category), see
//! [`Span`](crate::ops::Span) in the `ops/` module — Span is not a
//! measure (it fails subadditivity on disjoint sets).

mod extent;
pub use extent::Extent;
#[allow(clippy::module_inception)]
mod measure;
pub use measure::Measure;

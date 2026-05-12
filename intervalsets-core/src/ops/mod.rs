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
//! # Contract
//!
//! Set operations fall into four tiers, ordered from strongest to
//! weakest guarantee. Every public trait in this module names its
//! tier in its own rustdoc.
//!
//! This is the parallel to the four-tier *constructor* contract
//! documented under "Construction at boundaries" in the crate root,
//! and to the panicking/`Try*` split spelled out in [`math`].
//!
//! ## Tier 1 — Truly infallible
//!
//! Cannot panic, cannot error, cannot return wrong answers — *for
//! any input*, including pathological ones. The shape of the return
//! absorbs failure modes (e.g. predicates collapse incomparability
//! into `false`).
//!
//! Members: [`Contains`], [`Intersects`], [`Connects`]. Storage-type
//! casts at this tier — [`Cast`](crate::cast::Cast),
//! [`LossyCast`](crate::cast::LossyCast) — live in [`crate::cast`].
//!
//! ## Tier 2 — Infallible when closed over the invariants
//!
//! Cannot panic and cannot error *given inputs satisfying their type
//! invariants*. The type system prevents validating-API callers from
//! constructing invariant-violating inputs, so from the validating-API
//! caller's seat this tier is also infallible. These traits have no
//! `try_*` variant because the operation introduces no logical
//! violation of its own — there is nothing to surface.
//!
//! Internally, every `*_assume_*` checkpoint reachable from a Tier 2
//! op carries a `debug_assert!` that verifies the precondition it
//! relies on. In debug builds, an upstream invariant violation
//! (typically Tier 4 misuse) panics at the first checkpoint reached;
//! in release the asserts are compiled out and misuse propagates to
//! a wrong answer (no UB).
//!
//! Tier 2 is **fundamentally different from a panicking sugar
//! wrapper (Tier 3)**: a Tier 3 panic is an intentional `unwrap()`
//! on a documented `Err` from user-supplied `T` and is part of the
//! contract; a Tier 2 debug-mode panic is a tripwire on broken
//! invariants and is not reachable from validating-API usage.
//!
//! Members: [`Complement`], [`Intersection`], [`Union`],
//! [`Difference`], [`IntoFiniteInterval`], [`IntoElementIterator`], plus
//! [`MergeConnected`] (the `Option` is a domain answer — "operands
//! disconnected" — not an error). The bound on each impl varies; bound
//! choice is independent of fallibility.
//!
//! ## Tier 3 — `try_*` + panicking sugar
//!
//! Accepts user-supplied `T` that may break a logical constraint
//! (NaN, integer overflow, divide-by-zero, etc.). Tier 3 splits into
//! two halves with different guarantees:
//!
//! ### Tier 3a — `try_*` (total, panic-free)
//!
//! The `try_*` form returns `Result<_, Self::Error>` and **never
//! panics in release**. `Err` covers user-supplied breakage:
//! incomparable bounds (NaN), integer overflow / signed `MIN / -1`
//! (`MathError::Range`), integer divide-by-zero / non-finite float
//! result (`MathError::Domain`), or any user-defined `Error` from a
//! custom `T`. Per-impl `Error: core::error::Error`; some impls are
//! `Infallible` for already-validated inputs (the
//! [`ConvexHull::try_hull`]`<&FiniteInterval<T>>` precedent). The
//! `core-panic-canary` crate verifies the panic-free contract
//! against an external Kani prover.
//!
//! Members: [`Split::try_split`], [`Rebound::try_with_left`] /
//! [`Rebound::try_with_right`], [`ConvexHull::try_hull`], plus
//! [`math::TryAdd`] / [`math::TrySub`] / [`math::TryMul`] /
//! [`math::TryDiv`]. Storage-type cast at this tier —
//! [`TryCast`](crate::cast::TryCast) — lives in [`crate::cast`].
//!
//! ### Tier 3b — infix `+ - * /` and the non-`try_*` ops (panicking sugar)
//!
//! Defined as `lhs.try_op(rhs).unwrap()`. **May panic in release
//! and debug** when the underlying `try_*` would have returned
//! `Err`. The panic site is part of the documented contract — there
//! is no `T: Ord`-style "infallibility" claim. Use the `try_*`
//! sibling for the panic-free contract.
//!
//! Members: infix [`core::ops::Add`] / [`core::ops::Sub`] /
//! [`core::ops::Mul`] / [`core::ops::Div`] for set types, plus
//! [`Split::split`], [`Rebound::with_left`] / [`Rebound::with_right`],
//! [`ConvexHull::hull`].
//!
//! ## Tier 4 — `*_assume_valid` (bypass)
//!
//! Bypasses the validating surface. Caller asserts the precondition;
//! misuse produces a wrong answer (no UB, since the crate is
//! `forbid(unsafe_code)`). Public only because the outer crate needs
//! them for performance reasons. User code probably shouldn't reach
//! for these.
//!
//! Members are spread across `bound`, `sets`, etc. (not specific to
//! ops): `*_assume_valid` and `*_assume_normed` functions.

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
pub use merged::{MergeConnected, MergeSortedByRef, MergeSortedByValue};
mod rebound;
pub use rebound::Rebound;
mod split;
pub use split::Split;
mod union;
pub use union::Union;

mod finite;
pub use finite::IntoFiniteInterval;
mod span;
pub use span::Span;

mod elem_iter;
pub use elem_iter::{DisjointElements, Elements, IntoElementIterator};

pub mod math;

mod util;

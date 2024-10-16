//! # intervalsets: Intervals as Sets in Rust
//!
//! Intervalsets intends to provide full functionality of sets for
//! interval data.
//!
//! * The [`Interval`] type is a Set implementation representing a
//!   contiguous set of values.
//!     * It is generic over any type that implements the [`Domain`] trait
//!       which is intended to make sure elements are comparable.
//!
//! # Overview
//!
//! # Features
//!    
//! * rust_decimal
//! * num-bigint
//! * num-rational

#![allow(unused_variables)] // for now

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

// concrete types
//pub mod concrete;
pub mod ival;
mod sets;

// invariant traits
mod bounds;
mod display;
mod empty;
mod from;
mod numeric;
mod partial_ord;

// measures
pub mod measure;

// operation traits
pub mod op;

// predicate traits
pub(crate) mod pred;

pub(crate) mod util;

// reexports / public APIs
pub(crate) use sets::FiniteInterval;
pub(crate) use sets::HalfBounded;

pub use sets::Interval;
pub use sets::IntervalSet;

pub use ival::{Bound, Side};

pub use bounds::Bounds;
pub use empty::MaybeEmpty;
pub use measure::count::{Count, Countable};
pub use measure::width::Width;
pub use numeric::Domain;

pub use pred::contains::Contains;
pub use pred::intersects::Intersects;

pub use op::complement::Complement;
pub use op::difference::{Difference, SymmetricDifference};
pub use op::hull::ConvexHull;
pub use op::intersection::Intersection;
pub use op::merged::Merged;
//pub use op::padded::Padded;
//pub use op::shifted::Shifted;
pub use op::union::Union;

pub mod feats;

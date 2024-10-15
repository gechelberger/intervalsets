#![allow(unused_variables)] // for now

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

// concrete types
pub mod concrete;
pub mod ival;

// invariant traits
mod bounds;
mod display;
mod empty;
mod from;
mod normalize;
mod numeric;
mod partial_ord;
mod sizable;

// operation traits
pub mod op;

// predicate traits
pub(crate) mod pred;

pub(crate) mod util;

// reexports / public APIs
pub(crate) use concrete::finite::FiniteInterval;
pub(crate) use concrete::half::HalfInterval;

pub use concrete::interval::Interval;
pub use concrete::set::IntervalSet;

pub use ival::{Bound, Side};

pub use bounds::Bounds;
pub use empty::MaybeEmpty;
pub use numeric::Domain;
pub use sizable::{ISize, Sizable};

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

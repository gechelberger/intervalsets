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
pub mod bounds;
pub mod empty;
pub mod from;
mod normalize;
pub mod numeric;
pub mod partial_ord;
pub mod sizeable;

// operation traits
pub mod complement;
pub mod intersection;
pub mod merged;
pub mod union;

pub mod padded;
pub mod shifted;

// predicate traits
pub mod contains;
pub mod intersects;

pub(crate) mod util;

// reexports / public APIs
pub use concrete::finite::FiniteInterval;
pub use concrete::half::HalfInterval;
pub use concrete::interval::Interval;
pub use concrete::set::IntervalSet;

pub use normalize::Normalize;

#[cfg(feature = "decimal")]
pub mod decimal;

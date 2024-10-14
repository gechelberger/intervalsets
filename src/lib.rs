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
pub mod display;
pub mod empty;
pub mod from;
mod normalize;
pub mod numeric;
pub mod partial_ord;
pub mod sizeable;

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

pub use pred::contains::Contains;
pub use pred::intersects::Intersects;

pub use normalize::Normalize;

#[cfg(feature = "decimal")]
pub mod decimal;

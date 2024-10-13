#![feature(trait_alias)]
#![allow(unused_variables)] // for now

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod finite;
pub mod half;
pub mod infinite;
pub mod ival;
pub(crate) mod util;

// traits
pub mod empty;
pub mod from;
mod normalize;

pub mod bounds;
pub mod complement;
pub mod contains;
pub mod intersection;
pub mod intersects;
pub mod merged;
pub mod numeric;
pub mod partial_ord;
pub mod sizeable;
pub mod union;

pub mod padded;
pub mod shifted;

pub use finite::FiniteInterval;
pub use half::HalfInterval;
pub use infinite::Interval;

pub use normalize::Normalize;

#[cfg(feature = "decimal")]
pub mod decimal;

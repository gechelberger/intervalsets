#![feature(trait_alias)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod finite;
pub mod half;
pub mod infinite;
pub mod ival;

// traits
pub mod into;
mod normalize;
pub mod intersects;
pub mod intersection; 
pub mod union;
pub mod contiguous;
pub mod complement;
pub mod contains;
pub mod sizeable;
pub mod bounds;
pub mod numeric;

pub mod shifted;
pub mod padded;

pub use finite::FiniteInterval;
pub use half::HalfInterval;
pub use infinite::Interval;

pub use normalize::Normalize;
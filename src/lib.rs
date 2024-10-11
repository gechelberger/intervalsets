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
mod normalize;
pub mod union;
pub mod intersection; 
//pub mod complement;
pub mod contains;
pub mod sizeable;
pub mod bounds;

pub use finite::FiniteInterval;
pub use infinite::Interval;

pub use normalize::Normalize;
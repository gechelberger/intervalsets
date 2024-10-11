#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod finite;
pub mod infinite;
pub mod ival;

pub use finite::FiniteInterval;
pub use infinite::Interval;
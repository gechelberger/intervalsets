//! intervalsets-core
//! -----------------
//!
//!
#![no_std]
//#![deny(bad_style)]
//#![deny(missing_docs)]
//#![deny(future_incompatible)]
//#![deny(nonstandard_style)]
//#![deny(unused)]

pub mod bound;
pub mod numeric;

//pub mod error;
pub mod feat;
pub mod sets;
pub use sets::EnumInterval;

pub mod ops;

pub mod factory;
pub use factory::Factory;

pub mod measure;

pub mod try_cmp;

mod display;
mod from;

mod empty;
pub use empty::MaybeEmpty;

#[allow(unused_imports)]
pub mod prelude {
    #[cfg(feature = "rkyv")]
    pub use crate::bound::{ArchivedBoundType, ArchivedFiniteBound, ArchivedSide};
    pub use crate::bound::{BoundType, FiniteBound, SetBounds, Side};
    pub use crate::empty::MaybeEmpty;
    //pub use crate::error::Error;
    pub use crate::factory::Factory;
    pub use crate::measure::{Count, Measurement, Width};
    pub use crate::ops::*;
    #[cfg(feature = "rkyv")]
    pub use crate::sets::{ArchivedEnumInterval, ArchivedFiniteInterval, ArchivedHalfInterval};
    pub use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};
}

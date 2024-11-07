//! intervalsets-core
//! -----------------
//!
//!
#![no_std]

pub mod bound;
pub mod numeric;

pub mod error;
pub mod sets;
pub use sets::EnumInterval;

pub mod ops;

pub mod factory;
pub use factory::Factory;

mod from;

mod traits;

mod empty;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::bound::{BoundType, FiniteBound, SetBounds, Side};
    pub use crate::empty::MaybeEmpty;
    pub use crate::error::Error;
    pub use crate::factory::Factory;
    pub use crate::ops::*;
    pub use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};
}

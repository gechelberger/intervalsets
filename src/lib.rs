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

mod numeric;
pub use numeric::Domain;

mod bound;
pub use bound::{Bound, Side};

mod traits;
pub use traits::contains::Contains;
pub use traits::intersects::Intersects;
pub use traits::intersection::Intersection;
pub use traits::bounding::Bounding;
pub use traits::complement::Complement;
pub use traits::empty::MaybeEmpty;
pub use traits::merged::Merged;

mod detail;

mod sets;
pub use sets::{Interval, IntervalSet};

mod util;

//! # intervalsets: Intervals as Sets in Rust
//!
//! Intervalsets intends to provide full functionality of sets for
//! interval data.
//!
//! * The [`Interval`] type is a Set implementation representing a
//!   contiguous set of values.
//!     * It is generic over any type that implements the [`Domain`] trait
//!       which is intended to make sure elements are comparable and allows
//!       us to differentiate between discrete and continuous data types.
//!
//! * The [`IntervalSet`] type is a Set of disjoint, normalized `Intervals`
//!   maintained in sorted order.
//!
//! # Overview
//!
//! # Getting Started
//! ```
//!
//! ```
//!
//! # Optional Features
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

pub mod numeric;

mod bound;
pub use bound::{Bound, Side};

mod traits;
pub use traits::bounding::Bounding;
pub use traits::empty::MaybeEmpty;
pub use traits::hull::ConvexHull;

/// Operations on Set types.
pub mod ops {
    pub use crate::traits::contains::Contains;
    pub use crate::traits::intersects::Intersects;

    pub use crate::traits::complement::Complement;
    pub use crate::traits::difference::{Difference, SymmetricDifference};
    pub use crate::traits::intersection::Intersection;
    pub use crate::traits::merged::Merged;
    pub use crate::traits::union::Union;
}

mod detail;

pub mod measure;

mod sets;
pub use sets::{Interval, IntervalSet};

mod display;

mod feat;

mod util;

pub mod prelude {
    pub use crate::ops::*;
    pub use crate::sets::{Interval, IntervalSet};
}

//! # intervalsets: Intervals as Sets in Rust
//!
//! Intervalsets intends to provide the full functionality of sets for
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
//! [`Interval`] and [`IntervalSet`] are both intended to be simple, versatile,
//! and correct. If you find any bugs, please open an issue on the repository.
//!
//! They are **immutable** and can be easily be used in a multi-threaded environment,
//! or as keys in hash-structures as long as the underlying generic type is `Hash`.
//!
//! The vast majority of interactions with these `Set` types is governed by
//! trait implementations.
//!
//! # Getting Started
//!
//! ## Building Intervals and Sets
//!
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::closed(0, 10);
//! assert_eq!(x.contains(&5), true);
//! assert_eq!(x.count().finite(), 11);
//!
//! let y = Interval::closed(0.0, 10.0);
//! assert_eq!(y.contains(&5.0), true);
//! assert_eq!(y.width().finite(), 10.0);
//!
//! let z = Interval::open(100.0, 110.0);
//!
//! let set = IntervalSet::new(vec![y, z]);
//! assert_eq!(set.width().finite(), 20.0);
//! ```
//!
//! ## Set Operations
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::closed(0.0, 100.0);
//! let y = Interval::closed(1000.0, 1100.0);
//! let z = Interval::open(2000.0, 2100.0);
//!
//! let a = Interval::<f64>::unbounded()
//!     .difference(&x)
//!     .difference(&y)
//!     .difference(&z);
//!
//! assert_eq!(a.contains(&50.0), false);
//!
//! let b = x.union(&y).union(&z).complement();
//! assert_eq!(a, b);
//! ```
//!
//! ## Measure of a Set
//!
//! Two [measures](measure) are provided.
//!
//! They each return a [`measure::Measurement`] which may be infinite.
//!
//! ### [`measure::Width`] for continuous data types
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::open(0.0, 10.0);
//! assert_eq!(x.width().finite(), 10.0);
//!
//! let x = Interval::closed(0.0, 10.0);
//! assert_eq!(x.width().finite(), 10.0);
//!
//! let x = Interval::closed_unbound(0.0);
//! assert_eq!(x.width().is_finite(), false);
//! ```
//!
//! ### [`measure::Count`] for discrete data types
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::closed(0, 10);
//! assert_eq!(x.count().finite(), 11);
//!
//! let x = Interval::closed_unbound(0);
//! assert_eq!(x.count().is_finite(), false);
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
    //pub use crate::traits::adjacent::Adjacent;
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
mod hash;

mod feat;

mod util;

/// Common operations & traits
pub mod prelude {
    pub use crate::measure::{Count, Width};
    pub use crate::ops::*;
    pub use crate::sets::{Interval, IntervalSet};
    pub use crate::{Bounding, ConvexHull, MaybeEmpty};
}

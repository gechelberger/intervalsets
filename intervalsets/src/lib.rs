#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]

//! # intervalsets: Intervals as Sets in Rust
//!
//! Intervalsets intends to provide the full functionality of sets for
//! interval data.
//!
//! * The [`Interval`] type is a Set implementation representing a
//!   contiguous set of values.
//!     * It is generic over any type that implements the [`Element`] trait
//!       which makes sure elements are comparable and differentiates
//!       between discrete and continuous data types.
//!
//! * The [`IntervalSet`] type is a Set of disjoint, normalized `Intervals`
//!   maintained in sorted order.
//!
//! # Overview
//!
//! [`Interval`] and [`IntervalSet`] are intended to be simple, versatile,
//! and correct. If you find any bugs, please
//! [open an issue](https://github.com/gechelberger/intervalsets/issues/new)
//! or create a pull request.
//!
//! The vast majority of interactions with these `Set` types are governed by
//! trait implementations in the [`ops`] module.
//!
//! # Limitations
//!
//! Neither [`Interval`] nor [`IntervalSet`] are `Copy`.
//!
//! # Getting Started
//!
//! ## Constructing Sets
//!
//! ```
//! use intervalsets::prelude::*;
//! use intervalsets::bound::{FiniteBound, Side};
//!
//! let x = Interval::closed(0, 10);
//! assert_eq!(x.is_empty(), false);
//! assert_eq!(x.is_finite(), true);
//! assert_eq!(x.is_fully_bounded(), true);
//! assert_eq!(*x.right().unwrap(), FiniteBound::closed(10));
//! assert_eq!(*x.rval().unwrap(), 10);
//! assert_eq!(format!("x = {}", x), "x = [0, 10]");
//!
//! let x = Interval::closed_unbound(0.0);
//! assert_eq!(x.right(), None);
//! assert_eq!(x.is_half_bounded(), true);
//! assert_eq!(x.is_half_bounded_on(Side::Left), true);
//!
//! let x = Interval::closed(-100.0, -50.0);
//! let y = Interval::hull([5.0, 10.0, 23.0, -3.0, 22.0, 9.0, 99.9]);
//! assert_eq!(y, Interval::closed(-3.0, 99.9));
//!
//! let iset = IntervalSet::from_iter([x, y]);
//! assert_eq!(iset.slice().len(), 2);
//!
//! // closed intervals can also be converted from tuples
//! let iset2 = IntervalSet::from_iter([[-100.0, -50.0], [-3.0, 99.9]]);
//! assert_eq!(iset, iset2);
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
//!     .difference(x.clone())
//!     .difference(y.clone())
//!     .difference(z.clone());
//!
//! assert_eq!(a.contains(&50.0), false);
//!
//! let b = x.union(y).union(z).complement();
//! assert_eq!(a, b);
//!
//! let c = a.sym_difference(b).expect_interval();
//! assert_eq!(c, Interval::<f64>::empty());
//! ```
//!
//! ## Measure of a Set
//!
//! Two [measures](measure) are provided.
//!
//! They each return an [`Extent`](measure::Extent) which may be infinite.
//!
//! ### [`Width`](measure::Width) for continuous data types
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
//! ### [`Cardinality`](measure::Cardinality) for discrete data types
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::closed(0, 10);
//! assert_eq!(x.cardinality().finite(), 11u128);
//!
//! let x = Interval::closed_unbound(0);
//! assert_eq!(x.cardinality().is_finite(), false);
//! ```
//!
//! # Optional Features
//!
//! `intervalsets` has multiple Cargo features for controlling the underlying
//! data types used by [`Interval`] and [`IntervalSet`]. None are enabled by
//! default
//!
//! * rust_decimal
//! * num-bigint
//! * chrono
//! * uom
//!
#![deny(bad_style)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
//#![deny(unused)]

//#![warn(missing_docs)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub use intervalsets_core::bound::ord::OrdBounded;
pub use intervalsets_core::bound::{SetBounds, Side};
pub use intervalsets_core::numeric::Element;
pub use intervalsets_core::{bound, continuous_domain_impl, default_countable_impl, numeric};

pub mod error;
pub mod factory;

pub use intervalsets_core::MaybeEmpty;

pub mod measure;
pub mod ops;

mod sets;
pub use sets::{Interval, IntervalSet};

mod cast;
mod display;
mod feat;
mod from;
//mod util;

/// Common operations & traits
pub mod prelude {
    pub use intervalsets_core::cast::{Cast, LossyCast, TryCast};
    pub use intervalsets_core::factory::traits::*;

    pub use crate::measure::{Cardinality, Width};
    pub use crate::ops::*;
    pub use crate::sets::{Interval, IntervalSet};
    pub use crate::{Element, MaybeEmpty, OrdBounded, SetBounds, Side};
}

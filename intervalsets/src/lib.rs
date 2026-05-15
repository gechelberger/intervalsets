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
//! ## Compile-time interval literals
//!
//! The [`interval!`] macro parses a string literal at expansion time
//! in the same grammar as the runtime `FromStr` impl. Malformed input
//! — bad syntax, closed bracket on an unbounded side, crossed
//! numeric-literal bounds — fails to build instead of panicking at
//! runtime. Bound bodies are tokenized as Rust expressions, not just
//! literals.
//!
//! ```
//! use intervalsets::prelude::*;
//!
//! let x: Interval<i32> = interval!("[0, 10)");
//! let y: Interval<f64> = interval!("(.., 10.5]");
//! let z: Interval<i32> = interval!("(.., ..)");
//!
//! let n = 5_i32;
//! let from_expr: Interval<i32> = interval!("[n, n + 10]");
//! assert_eq!(from_expr, Interval::closed(5, 15));
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
//! The unified [`Measure`](measure::Measure) trait returns the natural
//! additive measure of a set — Lebesgue width on continuous T,
//! cardinality on discrete T. The result is an
//! [`Extent`](measure::Extent) which may be infinite.
//!
//! ### Continuous (Lebesgue width)
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::open(0.0, 10.0);
//! assert_eq!(x.measure().finite(), 10.0);
//!
//! let x = Interval::closed(0.0, 10.0);
//! assert_eq!(x.measure().finite(), 10.0);
//!
//! let x = Interval::closed_unbound(0.0);
//! assert_eq!(x.measure().is_finite(), false);
//! ```
//!
//! ### Discrete (cardinality)
//! ```
//! use intervalsets::prelude::*;
//!
//! // i32::Measure = u64 under stepwise widening.
//! let x = Interval::closed(0_i32, 10);
//! assert_eq!(x.measure().finite(), 11_u64);
//!
//! let x = Interval::closed_unbound(0_i32);
//! assert_eq!(x.measure().is_finite(), false);
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
pub use intervalsets_core::{bound, default_continuous_element_impl, numeric};
/// Compile-time-checked literal macro for [`Interval`]. Parses a
/// string literal at expansion time in the same grammar as the runtime
/// [`FromStr`](crate::Interval) impl; malformed input fails to build
/// instead of panicking at runtime.
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x: Interval<i32> = interval!("[0, 10]");
/// assert_eq!(x, Interval::closed(0, 10));
///
/// let y: Interval<f64> = interval!("[0.0, 10.0)");
/// assert_eq!(y, Interval::closed_open(0.0, 10.0));
///
/// let z: Interval<i32> = interval!("[0, ..)");
/// assert_eq!(z, Interval::closed_unbound(0));
///
/// let u: Interval<i32> = interval!("(.., ..)");
/// assert_eq!(u, Interval::unbounded());
///
/// let e: Interval<i32> = interval!("{}");
/// assert_eq!(e, Interval::empty());
///
/// // Bound bodies are arbitrary Rust expressions, not just literals:
/// let n = 5_i32;
/// let v: Interval<i32> = interval!("[n, n + 10]");
/// assert_eq!(v, Interval::closed(5, 15));
/// ```
pub use intervalsets_macros::interval;

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
mod parse;
//mod util;

/// Common operations & traits
pub mod prelude {
    pub use intervalsets_core::cast::{Cast, LossyCast, TryCast};
    pub use intervalsets_core::factory::traits::*;

    pub use crate::measure::Measure;
    pub use crate::ops::*;
    pub use crate::sets::{Interval, IntervalSet};
    pub use crate::{interval, Element, MaybeEmpty, OrdBounded, SetBounds, Side};
}

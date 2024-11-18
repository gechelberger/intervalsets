//! intervalsets-core is a no-std/no-alloc library for working with intervals as sets.
//!
//! # Overview
//!
//! intervalsets-core provides basic intervals and set operations
//! for a no-alloc environment.
//! [intervalsets](https://docs.rs/intervalsets/latest/intervalsets/) builds upon
//! this crate for additional functionality in an std environment.
//!
//! An **interval** is defined as a connected set of numbers with no gaps, of which
//! there are 5 general cases for the Reals. (todo: footnote about unsigned/whole
//! numbers but don't distract with that diatribe here...)
//!
//! | Case          | Base Implementation | Set Notation (Closed) | [`Range`](core::ops::Range) Equivalents |
//! |---------------|---------------------|-----------------------|------------------|
//! | Empty Set     | `FiniteInterval`    | {}                    | `Range` |
//! | Fully Bounded | `FiniteInterval`    | { x \| a <= x <= b }  | `Range`, `RangeInclusive` |
//! | Left Bounded  | `HalfInterval`      | { x \| a <= x }       | `RangeFrom` |
//! | Right Bounded | `HalfInterval`      | { x \| x <= b }       | `RangeTo`, `RangeToInclusive` |
//! | Unbounded     | `EnumInterval`      | { x \| x ∈ DataType } | `RangeFull` |
//!
//! Some performance gains can be achieved by working directly with the base
//! implementations, though it is often simpler to work with [`EnumInterval`]
//! which, as the name implies, is an enum of the base implementations and covers
//! all five cases. Any **set operation** on an `EnumInterval` should return another
//! `EnumInterval`. This is not the case for operations on [`FiniteInterval`] or
//! [`HalfInterval`] which will return the most narrow base implementation possible.
//!
//! Operations that might return disjoint sets (such as Union and Complement)
//! are provided in the main
//! [intervalsets](https://docs.rs/intervalsets/latest/intervalsets/) crate.
//!
//! # Limitations
//!
//! This family of crates is intended to provide robust, general implementations
//! of intervals with a convenient `Set` based api, and is pluggable with
//! user provided data-types. While attention has been given to performance,
//! there are many optimizations that can not be included without a loss of generality.
//!
//! Currently [interval arithmetic](https://en.wikipedia.org/wiki/Interval_arithmetic)
//! is not supported, and while it may be in the future, it will never be as
//! performant as a specialized library like [inari](https://docs.rs/inari/latest/inari/).
//!
//! # Getting Started
//!
//! **Intervals** are generally constructed via the [`Factory`] trait which is
//! implemented for each base interval type. Manipulation is provided via
//! [set operations](ops).
//!
//! ```
//! use intervalsets_core::prelude::*;
//!
//! let a = EnumInterval::closed(0, 10);
//!
//! assert!(a.contains(&5));
//! assert!(a.contains(&a));
//! assert!(a.contains(&EnumInterval::closed(4, 6)));
//! assert!(!a.contains(&EnumInterval::closed_unbound(5)));
//!
//! assert!(a.intersects(&EnumInterval::closed(10, 20)));
//! assert!(!a.intersects(&EnumInterval::closed(11, 20)));
//!
//! let merged = a.try_merge(EnumInterval::closed(5, 20)).unwrap();
//! assert_eq!(merged, EnumInterval::closed(0, 20));
//!
//! let merge_failed = a.try_merge(EnumInterval::closed(15, 20));
//! assert_eq!(merge_failed, None);
//!
//! let intersected = a.intersection(EnumInterval::closed(-5, 5));
//! assert_eq!(intersected, EnumInterval::closed(0, 5));
//!
//! let rebound_left = a.with_left_closed(7);
//! assert_eq!(rebound_left, EnumInterval::closed(7, 10));
//!
//! let rebound_right = a.with_right_closed(3);
//! assert_eq!(rebound_right, EnumInterval::closed(0, 3));
//!
//! let hull = EnumInterval::convex_hull([10, 8, 0, 6, 4, 2]).unwrap();
//! assert_eq!(hull, a);
//!
//! let empty = a.intersection(EnumInterval::closed(20, 30));
//! assert!(empty.is_empty());
//!
//! let (left, right) = a.split(5, Side::Left);
//! assert_eq!(left, EnumInterval::closed(0, 5));
//! assert_eq!(right, EnumInterval::closed(6, 10));
//!
//! let (left, right) = a.split(5, Side::Right);
//! assert_eq!(left, EnumInterval::closed(0, 4));
//! assert_eq!(right, EnumInterval::closed(5, 10));
//!
//! let width: Measurement<_> = a.width();
//! assert_eq!(width.finite(), 10);
//!
//! let count: Measurement<_> = a.count();
//! assert_eq!(count.finite(), 11);
//!
//! assert_eq!(format!("{}", a), "[0, 10]");
//! ```
//!
//! # Features
//! intervalsets-core has several feature flags that modify capabilities. By
//! default, none are enabled.
//!
//! ## testing
//!
//! * arbitrary: implement the [`Arbitrary`](arbitrary::Arbitrary) trait
//! * todo:
//!     * quickcheck?
//!     * proptest?
//!
//! ## storage
//!
//! * ordered-float: wrappers that provide a total order for floating point types.
//! * rust_decimal: fixed precision total ordered decimals
//! * bigdecimal: arbitrary precision total ordered decimals
//! * num-bigint: arbitrary sized integers
//!
//! ## serialization
//! * serde: implement `Serialize`, `Deserialize`
//! * rkyv: implement `Archive`, `Serialize`, `Deserialize`
//!
//! # Diving Deeper
//! * [Implement custom storage data types](numeric)
//! * [Adapt unsupported data types with factory converters](factory::Converter)
//!
#![no_std]
#![deny(bad_style)]
#![warn(missing_docs)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(unused)]

pub mod bound;
pub mod numeric;

//pub mod error;
mod feat;
pub mod sets;
pub use sets::{EnumInterval, FiniteInterval, HalfInterval};

pub mod ops;

pub mod factory;
pub use factory::Factory;

pub mod measure;

pub mod try_cmp;

mod display;
mod from;

mod empty;
pub use empty::MaybeEmpty;

/// commonly used imports
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

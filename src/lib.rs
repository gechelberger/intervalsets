//! # intervalsets: Intervals as Sets in Rust
//!
//! Intervalsets intends to provide the full functionality of sets for
//! interval data.
//!
//! * The [`Interval`] type is a Set implementation representing a
//!   contiguous set of values.
//!     * It is generic over any type that implements the [`Domain`](numeric::Domain) trait
//!       which is intended to make sure elements are comparable and allows
//!       us to differentiate between discrete and continuous data types.
//!
//! * The [`IntervalSet`] type is a Set of disjoint, normalized `Intervals`
//!   maintained in sorted order.
//!
//! # Overview
//!
//! [`Interval`] and [`IntervalSet`] are intended to be simple, versatile,
//! and correct. If you find any bugs, please open an issue on the repository
//! or open a pull request.
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
//! use intervalsets::{Bound, Side};
//!
//! let x = Interval::closed(0, 10);
//! assert_eq!(x.is_empty(), false);
//! assert_eq!(x.is_finite(), true);
//! assert_eq!(x.is_fully_bounded(), true);
//! assert_eq!(*x.right().unwrap(), Bound::closed(10));
//! assert_eq!(*x.rval().unwrap(), 10);
//! assert_eq!(format!("x = {}", x), "x = [0, 10]");
//!
//! let x = Interval::closed_unbound(0.0);
//! assert_eq!(x.right(), None);
//! assert_eq!(x.is_half_bounded(), true);
//! assert_eq!(x.is_half_bounded_on(Side::Left), true);
//!
//! let x = Interval::closed_open(-100.0, -50.0);
//! assert_eq!(*x.right().unwrap(), Bound::open(-50.0));
//!
//! let y = Interval::convex_hull([5.0, 10.0, 23.0, -3.0, 22.0, 9.0, 99.9]);
//! assert_eq!(y, Interval::closed(-3.0, 99.9));
//!
//! let iset = IntervalSet::from_iter([x, y]);
//! assert_eq!(iset.slice().len(), 2);
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
//!     .ref_difference(&x)
//!     .ref_difference(&y)
//!     .ref_difference(&z);
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
//! ## General Mapping
//!
//! ```
//! use intervalsets::prelude::*;
//!
//! let x = Interval::closed(1, 5);
//! let y = x.flat_map_finite(|left, right| {
//!     Interval::new_finite(left.clone().map(|v| 10 * v), right.clone().map(|v| 20 * v))
//! });
//! assert_eq!(y, Interval::closed(10, 100));
//!
//! let z = IntervalSet::from_iter([x.clone(), y.clone()]);
//! let z = z.collect_map(|mut sets, subset| {
//!     let mirror_image = subset.flat_map_finite(|left, right| {
//!         Interval::new_finite(right.clone().map(|v| -v), left.clone().map(|v| -v))
//!     });
//!     sets.push(mirror_image);
//!     sets.push(subset.clone());
//! });
//! assert_eq!(z, IntervalSet::from_iter([
//!     x,
//!     y,
//!     Interval::closed(-5, -1),
//!     Interval::closed(-100, -10),
//! ]));
//! ```
//!
//! ## Measure of a Set
//!
//! Two [measures](measure) are provided.
//!
//! They each return a [`Measurement`](measure::Measurement) which may be infinite.
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
//! ### [`Count`](measure::Count) for discrete data types
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
//! `intervalsets` has multiple Cargo features for controlling the underlying
//! data types used by [`Interval`] and [`IntervalSet`]. None are enabled by
//! default
//!    
//! * rust_decimal
//! * num-bigint
//! * chrono
//! * uom
//!
//! # Custom Data Types
//!
//! If you have a data type that is not currently supported by `intervalsets`
//! out of the box, there are a few traits that need to be implemented in order
//! to get started.
//!
//! Firstly, does your library or application own the type you want to use?
//! [Yes? Great! Skip Ahead](#required-custom-type-traits)
//!
//! ## External Type Conflicts
//!
//! Rust doesn't allow us to implement traits for types if we don't own at least
//! one of them for fear that a future upstream change will introduce ambiguity.
//! The solution to which is the
//! [New Type Idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html).
//!
//! So, we need to proxy whatever type we want to work with.
//!
//! ```ignore
//! use num_bigint::BigInt;
//! pub struct MyBigInt(BigInt);
//!
//! // implement all the traits...
//! ```
//!
//! ## Required Custom Type Traits
//!
//! `intervalsets` uses a handful of traits to fully define interval and set
//! behavior.
//!
//! * [`Domain`](numeric::Domain)
//! > The `Domain` trait serves one purpose -- to distinguish between types
//! > that represent **discrete** vs **continuous** data.
//! >
//! > This allows us to do two important things:
//! > 1. Normalize discrete sets so that there is only a single valid
//! >    representations of each possible `Set`.
//! >    eg. **[1, 9]** == (0, 10) == (0, 9] == [1, 10).
//! > 2. Properly test the adjacency of sets.
//! >
//! > The method [`try_adjacent`](numeric::Domain::try_adjacent) is the
//! > mechanism by which both of these goals is accomplished. Implementations
//! > for **continuous** types should simply return None.
//! >
//! > The macro [`continuous_domain_impl`] exists for exactly this purpose.
//!
//! * [`LibZero`](numeric::LibZero)
//! > The LibZero trait is a direct knock-off of [`Zero`](num_traits::Zero).
//! > However, it is necessary to resolve issues of external traits and
//! > types. If a type already implements `Zero` the macro
//! > [`adapt_num_traits_zero_impl`] can be used directly.
//! >
//! > The `LibZero` trait is necessary for the [`measure`] module,
//! > specifically in regards to the empty set.
//!
//! * [`Countable`](measure::Countable)
//! > The `Countable` trait is only relevant to **discrete** data types. It is
//! > the mechanism by which a data type can say how many distinct values fall
//! > between some bounds of that type. There is a macro
//! > [`default_countable_impl`] which uses [`try_adjacent`](numeric::Domain).
//!
//! * [`Add`](core::ops::Add) and [`Sub`](core::ops::Sub)
//! > The `Add` and `Sub` traits are used by the [`measure`] module, and could
//! > be used elsewhere in the future. Presumably these are already implemented
//! > for most types that one would want to use as bounds of a Set.
//!
//! ## Putting it all together
//!
//! ```
//! use core::ops::{Add, Sub};
//! use intervalsets::numeric::{Domain, LibZero};
//! use intervalsets::measure::Countable;
//! use intervalsets::Side;
//!
//! // minimum required is: Clone, PartialEq, PartialOrd
//! #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! pub struct MyInt(i32);
//!
//! impl Domain for MyInt {
//!     fn try_adjacent(&self, side: Side) -> Option<Self> {
//!         Some(match side {
//!             Side::Left => Self(self.0 - 1),
//!             Side::Right => Self(self.0 + 1),
//!         })
//!     }
//! }
//!
//! // MyInt does not already implement num_traits::Zero
//! // so the adapt_num_traits_zero_impl doesn't help us here,
//! // even though i32 does.
//! impl LibZero for MyInt {
//!     fn new_zero() -> Self {
//!         Self(0)
//!     }
//! }
//!
//! // The default_countable_impl macro would work here just fine.
//! // This would be omitted for a continuous data type.
//! impl Countable for MyInt {
//!     type Output = Self;
//!     fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
//!         Some(Self(right.0 - left.0 + 1))
//!     }
//! }
//!
//! impl Add for MyInt {
//!     type Output = Self;
//!     fn add(self, rhs: Self) -> Self {
//!         Self(self.0 + rhs.0)
//!     }
//! }
//!
//! impl Sub for MyInt {
//!     type Output = Self;
//!     fn sub(self, rhs: Self) -> Self {
//!         Self(self.0 - rhs.0)
//!     }
//! }
//! ```

#![allow(unused_variables)] // for now

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod numeric;

mod bound;
pub use bound::{Bound, BoundType, Side};

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
    pub use crate::traits::difference::{
        Difference, RefDifference, RefSymmetricDifference, SymmetricDifference,
    };
    pub use crate::traits::intersection::{Intersection, RefIntersection};
    pub use crate::traits::merged::{Merged, RefMerged};
    pub use crate::traits::split::{RefSplit, Split};
    pub use crate::traits::union::{RefUnion, Union};
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

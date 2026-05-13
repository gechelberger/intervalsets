//! Set-algebra and arithmetic operations on intervals and interval sets.
//!
//! Re-exports the trait set from [`intervalsets_core::ops`] and adds
//! implementations for the allocating
//! [`IntervalSet`](crate::IntervalSet), so set operations can return
//! arbitrary-piece results. The core crate's implementations cap output
//! at two pieces via
//! [`MaybeDisjoint`](intervalsets_core::sets::MaybeDisjoint).
//!
//! [`SymDifference`] is added at this layer — it composes naturally
//! over `IntervalSet` inputs that the core crate doesn't model.
//!
//! For panic-free variants of fallible operations, see the `Try*`
//! traits ([`TryAdd`], [`TrySub`], [`TryMul`], [`TryDiv`]).

mod connects;
pub use connects::Connects;

mod contains;
pub use contains::Contains;

mod complement;
pub use intervalsets_core::ops::Complement;

mod intersects;
pub use intersects::Intersects;

mod hull;
pub use hull::ConvexHull;

mod split;
pub use split::Split;

mod bisect;
pub use bisect::{Bisect, Bisection};

mod intersection;
pub use intersection::Intersection;

mod union;
pub use union::Union;

mod merged;
pub use merged::MergeConnected;

mod difference;
pub use difference::{Difference, SymDifference};

mod rebound;
pub use rebound::Rebound;

mod finite;
pub use finite::IntoFiniteInterval;

mod elem_iter;
pub use elem_iter::{DisjointElements, Elements, IntoElementIterator, SetElements};

mod math;
pub use math::{TryAdd, TryDiv, TryMul, TrySub};

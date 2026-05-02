//! traits for set operations.

mod complement;
pub use complement::Complement;
mod connects;
pub use connects::{are_bounds_connected, Connects};
mod contains;
pub use contains::Contains;
mod difference;
pub use difference::Difference;
mod intersects;
pub use intersects::Intersects;

mod hull;
pub use hull::{convex_hull_into_ord_bound_impl, convex_hull_ord_bounded_impl, ConvexHull};
mod intersection;
pub use intersection::{Intersection, SetSetIntersection};
mod merged;
pub use merged::{MergeSortedByRef, MergeSortedByValue, TryMerge};
mod rebound;
pub use rebound::Rebound;
mod split;
pub use split::Split;
mod union;
pub use union::Union;

mod finite;
pub use finite::IntoFinite;

#[doc(hidden)]
pub mod math;
pub use math::{TryAdd, TryDiv, TryMul, TrySub};

mod util;

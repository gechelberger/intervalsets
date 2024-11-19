//! traits for set operations.

mod adjacent;
pub use adjacent::Adjacent;
mod contains;
pub use contains::Contains;
mod intersects;
pub use intersects::Intersects;

mod hull; // todo: ord impls
pub use hull::{convex_hull_into_ord_bound_impl, convex_hull_ord_bounded_impl, ConvexHull};
mod intersection; // todo: SetSetIntersection
pub use intersection::{Intersection, SetSetIntersection};
mod merged; // todo: MergeSorted
pub use merged::{mergeable, MergeSorted, TryMerge};
mod rebound;
pub use rebound::Rebound;
mod split;
pub use split::Split;

mod util;

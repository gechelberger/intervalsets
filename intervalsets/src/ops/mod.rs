mod connects;
pub use connects::Connects;

mod contains;
pub use contains::Contains;

mod complement;
pub use complement::Complement;

mod intersects;
pub use intersects::Intersects;

mod hull;
pub use hull::ConvexHull;

mod split;
pub use split::Split;

mod intersection;
pub use intersection::Intersection;

mod union;
pub use union::Union;

mod merged;
pub use merged::TryMerge;

mod difference;
pub use difference::{Difference, SymDifference};

mod rebound;
pub use rebound::Rebound;

mod finite;
pub use finite::IntoFinite;

mod math;

//! todo...

use crate::MaybeEmpty;

mod adjacent;
pub use adjacent::Adjacent;
mod contains;
mod intersects;

pub mod hull; // todo: ord impls
pub mod intersection; // todo: SetSetIntersection
pub mod merged; // todo: MergeSorted
mod rebound;
pub use rebound::Rebound;
mod split;

mod util;

/// Defines the minimal contiguous Interval
/// which fully contains every provided item.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// // from points on the number line
/// let hull = FiniteInterval::convex_hull([5, 3, -120, 44, 100, -100]);
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// let items = vec![5, 3, -120, 44, 100, -100];
/// let hull = FiniteInterval::convex_hull(&items);
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// // from intervals
/// let intervals = [
///     EnumInterval::open(30, 50),
///     EnumInterval::closed(20, 40),
///     EnumInterval::closed(1000, 2000),
///     EnumInterval::unbound_open(0),
/// ];
/// let hull = EnumInterval::convex_hull(intervals);
/// assert_eq!(hull, EnumInterval::unbound_closed(2000));
/// ```
pub trait ConvexHull<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self;
}

pub trait Contains<T> {
    fn contains(&self, rhs: T) -> bool;
}

pub trait Intersects<T> {
    fn intersects(&self, rhs: T) -> bool;

    fn is_disjoint_from(&self, rhs: T) -> bool {
        !self.intersects(rhs)
    }
}

pub trait TryMerge<Rhs = Self> {
    type Output;

    fn try_merge(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait Intersection<Rhs = Self> {
    type Output;
    fn intersection(self, rhs: Rhs) -> Self::Output;
}

/// Split a Set into two disjoint subsets, fully covering the original.
///
/// `at` provides the new bound where the set should be split.
///
/// # Example
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// let (left, right) = x.split(5, Side::Left);
/// assert_eq!(left, FiniteInterval::closed(0, 5));
/// assert_eq!(right, FiniteInterval::closed(6, 10));
/// ```
pub trait Split<T> {
    type Output: Sized;
    fn split(self, at: T, closed: crate::bound::Side) -> (Self::Output, Self::Output);
}

#[inline]
pub fn mergeable<'a, A, B>(a: &'a A, b: &'a B) -> bool
where
    A: MaybeEmpty + Intersects<&'a B> + Adjacent<&'a B>,
    B: MaybeEmpty,
{
    a.intersects(b) || a.is_adjacent_to(b) || a.is_empty() || b.is_empty()
}

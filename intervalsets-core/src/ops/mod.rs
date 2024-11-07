//! todo...

use adjacent::Adjacent;

pub mod adjacent;
pub mod contains;
pub mod intersects;

pub mod complement;

pub mod difference;
pub mod hull;
pub mod intersection;
pub mod merged;
pub mod split;
pub mod union;

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
///
///
/// // from sets
/// let sets = [
///     EnumInterval::closed(0, 10).union(EnumInterval::closed(1000, 1010)),
///     EnumInterval::closed(-1000, 10).into(),
///     EnumInterval::closed(-500, 500).union(EnumInterval::closed_unbound(800))
/// ];
/// let hull = EnumInterval::convex_hull(sets);
/// assert_eq!(hull, EnumInterval::closed_unbound(-1000))
/// ```
pub trait ConvexHull<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self;
}

pub trait Contains<T> {
    fn contains(&self, rhs: &T) -> bool;
}

pub trait Intersects<T> {
    fn intersects(&self, rhs: &T) -> bool;

    fn is_disjoint_from(&self, rhs: &T) -> bool {
        !self.intersects(rhs)
    }
}

/// Defines the complement of a Set.
///
/// ```text
/// Let A  = { x | P(x) } =>
///     A' = { x | x ∉ A } = { x | !P(x) }
/// ```
pub trait Complement {
    type Output;

    fn complement(self) -> Self::Output;
}

pub trait Merged<Rhs = Self> {
    type Output;
    fn merged(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait Union<Rhs = Self> {
    type Output;
    fn union(self, rhs: Rhs) -> Self::Output;
}

pub trait Intersection<Rhs = Self> {
    type Output;
    fn intersection(self, rhs: Rhs) -> Self::Output;
}

/// Defines the difference of sets A - B.
///
/// ```text
/// Let A ⊆ T, B ⊆ T:
///
/// { x | x ∈ A && x ∉ B }
/// ```
///
/// Difference is not commutative.
///
/// # Example
///
/// ```
/// use intervalsets_core::{EnumInterval, Factory};
/// use intervalsets_core::ops::{Difference, Union};
///
/// let a = EnumInterval::closed(0, 100);
/// let b = EnumInterval::closed(50, 150);
/// assert_eq!(
///     a.clone().difference(b.clone()),
///     EnumInterval::closed_open(0, 50).into()
/// );
/// assert_eq!(
///     b.difference(a),
///     EnumInterval::open_closed(100, 150).into()
/// );
/// ```
pub trait Difference<Rhs = Self> {
    type Output;
    fn difference(self, rhs: Rhs) -> Self::Output;
}

/// Defines the symmetric difference (A ⊕ B). A and B are consumed.
///
/// ```text
/// Let A ⊆ T, B ⊆ T:
///
/// {x | x ∈ (A ∪ B) && x ∉ (A ∩ B) }
/// ```
///
/// Symmetric difference is commutative.
///
/// Example:
/// ```
/// use intervalsets_core::{EnumInterval, Factory};
/// use intervalsets_core::ops::{SymDifference, Union};
///
/// let a = EnumInterval::closed(0, 10);
/// let b = EnumInterval::closed(5, 15);
/// let expected = EnumInterval::closed_open(0, 5)
///     .union(EnumInterval::open_closed(10, 15));
/// assert_eq!(a.clone().sym_difference(b.clone()), expected);
/// assert_eq!(b.clone().sym_difference(a.clone()), expected);
/// assert_eq!(a.clone().sym_difference(a), EnumInterval::empty().into())
/// ```
pub trait SymDifference<Rhs = Self> {
    type Output;
    fn sym_difference(self, rhs: Rhs) -> Self::Output;
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

pub fn mergeable<A, B>(a: &A, b: &B) -> bool
where
    A: Intersects<B> + Adjacent<B>,
{
    a.intersects(b) || a.is_adjacent_to(b)
}

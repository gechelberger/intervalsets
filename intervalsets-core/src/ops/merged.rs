use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::numeric::Element;
use crate::ops::{Connects, Contains};
use crate::sets::EnumInterval::{self, *};
use crate::sets::{FiniteInterval, HalfInterval, MaybeDisjoint};
use crate::MaybeEmpty;

/// The union of two intervals if and only if [connected](`Connects`) else `None``.
///
/// ```text
/// {x | x ∈ A ∨ x ∈ B } ⇔ {x} is an interval
/// ```
///
/// # Note
///
/// > Types subject to rounding errors (floats) may have unexpected results.
/// > When testing adjacency PartialEq is used directly. Handling
/// > edge cases is left to the end user. A fixed precision decimal
/// > type may be preferred in some cases.
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). Cannot panic
/// or error given inputs satisfying their type invariants. The
/// returned `Option` is a domain answer — `None` means the operands
/// are disconnected and have no single-piece fusion — not an error
/// signal. See [`crate::ops`] for the full tier model.
///
/// # Examples
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0.0, 0.3);
/// let y = FiniteInterval::closed(0.1 + 0.2, 1.0);
///
/// assert_eq!(x.merge_connected(y), None);
/// ```
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
///
/// let y = FiniteInterval::closed(11, 20);
/// assert_eq!(x.merge_connected(y).unwrap(), FiniteInterval::closed(0, 20));
///
/// let y = FiniteInterval::closed(20, 30);
/// assert_eq!(x.merge_connected(y), None);
///
/// let y = FiniteInterval::<i32>::empty();
/// assert_eq!(x.merge_connected(y).unwrap(), x);
/// assert_eq!(y.merge_connected(x).unwrap(), x);
///
/// let x = FiniteInterval::<i32>::empty();
/// assert_eq!(x.merge_connected(y).unwrap(), FiniteInterval::empty());
/// ```
pub trait MergeConnected<Rhs = Self> {
    /// The type of interval to return when mergeable.
    type Output;

    /// Tries to merge two intervals into a single interval.
    fn merge_connected(self, rhs: Rhs) -> Option<Self::Output>;
}

impl<T: Element> MergeConnected<Self> for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if self.connects(&rhs) {
            let Some((lhs_min, lhs_max)) = self.into_raw() else {
                // intersects is false, but the empty set
                // is trivially connected to all other sets.
                return Some(rhs);
            };

            let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
                // repacking
                let lhs = Self::new_assume_valid(lhs_min, lhs_max);
                return Some(lhs);
            };

            // lhs and rhs satisfy invariants -> bounds are normalized, comparable,
            // and min(left, right) <= max(left, right)
            let merged = FiniteInterval::new_assume_valid(
                FiniteBound::take_min_assume_valid(Side::Left, lhs_min, rhs_min),
                FiniteBound::take_max_assume_valid(Side::Right, lhs_max, rhs_max),
            );

            Some(merged)
        } else {
            None
        }
    }
}

impl<T: Element + Clone> MergeConnected<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if self.connects(rhs) {
            let Some((lhs_min, lhs_max)) = self.view_raw() else {
                return Some(rhs.clone());
            };

            let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
                // just putting it back together
                let lhs = FiniteInterval::new_assume_valid(lhs_min.clone(), lhs_max.clone());
                return Some(lhs);
            };

            // lhs and rhs satisfy invariants -> bounds are normalized, comparable,
            // and min(left, right) <= max(left, right)
            let merged = FiniteInterval::new_assume_valid(
                FiniteBound::min_assume_valid(Side::Left, lhs_min, rhs_min).clone(),
                FiniteBound::max_assume_valid(Side::Right, lhs_max, rhs_max).clone(),
            );

            Some(merged)
        } else {
            None
        }
    }
}

impl<T: Element> MergeConnected<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                Some(self.into())
            } else {
                Some(rhs.into())
            }
        } else if self.connects(&rhs) {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            Some(EnumInterval::Unbounded)
        } else {
            None
        }
    }
}

impl<T: Element + Clone> MergeConnected<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else if self.connects(rhs) {
            Some(EnumInterval::Unbounded)
        } else {
            None
        }
    }
}

impl<T: Element> MergeConnected<FiniteInterval<T>> for HalfInterval<T> {
    type Output = HalfInterval<T>;

    #[inline(always)]
    fn merge_connected(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        if self.connects(&rhs) {
            let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
                return Some(self); // identity: merge with empty
            };

            let left_contained = self.side() == Left && self.contains(rhs_min.finite_ord(Left));
            let right_contained = self.side() == Right && self.contains(rhs_max.finite_ord(Right));
            if left_contained || right_contained {
                Some(self)
            } else {
                let bound = self.side().select(rhs_min, rhs_max);
                // bound is stolen from existing FiniteInterval -> already comparable
                Some(HalfInterval::new_assume_valid(self.side(), bound))
            }
        } else {
            None
        }
    }
}

impl<T: Element + Clone> MergeConnected<&FiniteInterval<T>> for &HalfInterval<T> {
    type Output = HalfInterval<T>;

    #[inline(always)]
    fn merge_connected(self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        if self.connects(rhs) {
            let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
                return Some(self.clone());
            };

            let left_contained = self.side() == Left && self.contains(rhs_min.finite_ord(Left));
            let right_contained = self.side() == Right && self.contains(rhs_max.finite_ord(Right));
            if left_contained || right_contained {
                Some(self.clone())
            } else {
                let bound = self.side().select(rhs_min, rhs_max).clone();
                // bound is taken from existing FiniteInterval -> already comparable
                Some(HalfInterval::new_assume_valid(self.side(), bound))
            }
        } else {
            None
        }
    }
}

macro_rules! dispatch_merge_connected_impl {
    ($t_rhs:ty) => {
        impl<T: $crate::numeric::Element> MergeConnected<$t_rhs> for EnumInterval<T> {
            type Output = EnumInterval<T>;
            #[inline(always)]
            fn merge_connected(self, rhs: $t_rhs) -> Option<Self::Output> {
                match self {
                    Finite(lhs) => lhs.merge_connected(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.merge_connected(rhs).map(EnumInterval::from),
                    Unbounded => Some(Unbounded),
                }
            }
        }
        impl<T: $crate::numeric::Element + Clone> MergeConnected<&$t_rhs> for &EnumInterval<T> {
            type Output = EnumInterval<T>;
            #[inline(always)]
            fn merge_connected(self, rhs: &$t_rhs) -> Option<Self::Output> {
                match self {
                    Finite(lhs) => lhs.merge_connected(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.merge_connected(rhs).map(EnumInterval::from),
                    Unbounded => Some(Unbounded),
                }
            }
        }
    };
}

dispatch_merge_connected_impl!(FiniteInterval<T>);
dispatch_merge_connected_impl!(HalfInterval<T>);
dispatch_merge_connected_impl!(EnumInterval<T>);

macro_rules! commutative_merge_connected_impl {
    ($t_lhs:ty, $t_rhs:ty, $t_ret:ty) => {
        impl<T: $crate::numeric::Element> MergeConnected<$t_rhs> for $t_lhs {
            type Output = $t_ret;
            #[inline(always)]
            fn merge_connected(self, rhs: $t_rhs) -> Option<Self::Output> {
                rhs.merge_connected(self)
            }
        }

        impl<T: $crate::numeric::Element + Clone> MergeConnected<&$t_rhs> for &$t_lhs {
            type Output = $t_ret;
            #[inline(always)]
            fn merge_connected(self, rhs: &$t_rhs) -> Option<Self::Output> {
                rhs.merge_connected(self)
            }
        }
    };
}

commutative_merge_connected_impl!(FiniteInterval<T>, HalfInterval<T>, HalfInterval<T>);
commutative_merge_connected_impl!(FiniteInterval<T>, EnumInterval<T>, EnumInterval<T>);
commutative_merge_connected_impl!(HalfInterval<T>, EnumInterval<T>, EnumInterval<T>);

// ===== MaybeDisjoint =====
//
// Cardinality is trivially bounded: the trait returns `Option<single
// piece>` regardless of input cardinality. Tied to the `Connects`
// contract: `connects(rhs) ⇒ merge_connected(rhs).is_some()`.
//
// Strategy: pre-check `connects`; if true, the merged result is the
// convex hull of `self ∪ rhs`, which equals `hull(self_hull ∪
// rhs_hull)` — i.e., merging the two single-piece hulls. The
// pre-check is mandatory because hull-of-self may include elements
// not in self (the gap), so hull-merging without the check would
// produce a wrong `Some(...)` answer for an rhs that sits in self's
// gap.

macro_rules! md_merge_connected_single_impl {
    ($t_rhs:ty) => {
        impl<T: $crate::numeric::Element> MergeConnected<$t_rhs> for MaybeDisjoint<T> {
            type Output = EnumInterval<T>;

            #[inline]
            fn merge_connected(self, rhs: $t_rhs) -> Option<Self::Output> {
                if !self.connects(&rhs) {
                    return None;
                }
                self.into_hull().merge_connected(rhs)
            }
        }

        impl<T: $crate::numeric::Element + Clone> MergeConnected<&$t_rhs> for &MaybeDisjoint<T> {
            type Output = EnumInterval<T>;

            #[inline]
            fn merge_connected(self, rhs: &$t_rhs) -> Option<Self::Output> {
                if !self.connects(rhs) {
                    return None;
                }
                self.hull().merge_connected(rhs.clone())
            }
        }
    };
}

md_merge_connected_single_impl!(FiniteInterval<T>);
md_merge_connected_single_impl!(HalfInterval<T>);
md_merge_connected_single_impl!(EnumInterval<T>);

impl<T: Element> MergeConnected<Self> for MaybeDisjoint<T> {
    type Output = EnumInterval<T>;

    #[inline]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if !self.connects(&rhs) {
            return None;
        }
        self.into_hull().merge_connected(rhs.into_hull())
    }
}

impl<T: Element + Clone> MergeConnected<Self> for &MaybeDisjoint<T> {
    type Output = EnumInterval<T>;

    #[inline]
    fn merge_connected(self, rhs: Self) -> Option<Self::Output> {
        if !self.connects(rhs) {
            return None;
        }
        self.hull().merge_connected(rhs.hull())
    }
}

// Commutative wrappers. Can't use `commutative_merge_connected_impl!`
// because the MD-side by-ref impl requires `T: Element + Clone` matching,
// which the macro provides — but writing inline for clarity / locality
// with the rest of the MD impls.
macro_rules! md_merge_connected_commutative_impl {
    ($t_lhs:ty) => {
        impl<T: $crate::numeric::Element> MergeConnected<MaybeDisjoint<T>> for $t_lhs {
            type Output = EnumInterval<T>;

            #[inline(always)]
            fn merge_connected(self, rhs: MaybeDisjoint<T>) -> Option<Self::Output> {
                rhs.merge_connected(self)
            }
        }

        impl<T: $crate::numeric::Element + Clone> MergeConnected<&MaybeDisjoint<T>> for &$t_lhs {
            type Output = EnumInterval<T>;

            #[inline(always)]
            fn merge_connected(self, rhs: &MaybeDisjoint<T>) -> Option<Self::Output> {
                rhs.merge_connected(self)
            }
        }
    };
}

md_merge_connected_commutative_impl!(FiniteInterval<T>);
md_merge_connected_commutative_impl!(HalfInterval<T>);
md_merge_connected_commutative_impl!(EnumInterval<T>);

/// MergeSorted merges intersecting intervals and returns disjoint ones.
///
/// As an `Iterator` is should return disjoint intervals from the sorted
/// input in order, omitting empty sets.
pub struct MergeSortedByValue<S, I: Iterator<Item = S>> {
    sorted: core::iter::Peekable<I>,
}

impl<S, I> MergeSortedByValue<S, I>
where
    S: MaybeEmpty,
    I: Iterator<Item = S>,
{
    /// Creates a new `MergeSorted` Iterator
    ///
    /// If the input is not sorted, behavior is undefined.
    pub fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = S, IntoIter = I>,
    {
        let mut sorted = sorted.into_iter().peekable();

        // discard all empty sets from the list
        while let Some(head) = sorted.peek() {
            if head.is_empty() {
                sorted.next();
            } else {
                break;
            }
        }

        Self { sorted }
    }
}

impl<S, I> Iterator for MergeSortedByValue<S, I>
where
    S: MergeConnected + for<'a> Connects<&'a S>,
    S: From<<S as MergeConnected>::Output>,
    I: Iterator<Item = S>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?;

        while let Some(peek) = self.sorted.peek() {
            if !current.connects(peek) {
                break;
            }

            let candidate = self.sorted.next().unwrap();
            current = match current.merge_connected(candidate) {
                Some(merged) => S::from(merged),
                None => {
                    // Connects/MergeConnected contract: connects(rhs) ⇒ merge_connected(rhs).is_some().
                    // Reaching this arm means an upstream invariant has been
                    // violated (e.g. via a Tier 4 *_assume_valid bypass). In
                    // debug builds, panic loudly; in release, end the iterator
                    // gracefully rather than diverge.
                    debug_assert!(
                        false,
                        "Connects/MergeConnected contract violation: connected but not mergeable"
                    );
                    return None;
                }
            };
        }

        Some(current)
    }
}

/// MergeSorted merges intersecting intervals and returns disjoint ones.
pub struct MergeSortedByRef<'a, T: 'a, I: Iterator<Item = &'a EnumInterval<T>>> {
    sorted: itertools::PutBack<I>,
}

impl<'a, T, I> MergeSortedByRef<'a, T, I>
where
    I: Iterator<Item = &'a EnumInterval<T>>,
{
    /// Creates a new `MergeSorted` Iterator
    ///
    /// If the input is not sorted, behavior is undefined.
    #[allow(unused)]
    pub fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = &'a EnumInterval<T>, IntoIter = I>,
    {
        Self {
            sorted: itertools::put_back(sorted),
        }
    }
}

impl<'a, T, I> Iterator for MergeSortedByRef<'a, T, I>
where
    T: Clone + Element,
    I: Iterator<Item = &'a EnumInterval<T>>,
{
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?.clone();

        while let Some(cand) = self.sorted.next() {
            current = match (&current).merge_connected(cand) {
                Some(merged) => merged,
                None => {
                    self.sorted.put_back(cand);
                    break;
                }
            };
        }

        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_finite_finite() {
        let a = EnumInterval::closed(0, 10);
        let b = EnumInterval::closed(5, 15);
        let c = EnumInterval::closed(20, 30);

        let expected = Some(EnumInterval::closed(0, 15));
        assert_eq!((&a).merge_connected(&b), expected);
        assert_eq!(a.merge_connected(b), expected);

        let expected = None;
        assert_eq!((&a).merge_connected(&c), expected);
        assert_eq!(a.merge_connected(c), expected);

        let empty = EnumInterval::empty();

        assert_eq!((&a).merge_connected(&empty), Some(a));
        assert_eq!(a.merge_connected(empty), Some(a));

        assert_eq!((&empty).merge_connected(&a), Some(a));
        assert_eq!(empty.merge_connected(a), Some(a));
    }

    #[test]
    fn test_half_half() {
        let a = EnumInterval::unbound_closed(-10);
        let b = EnumInterval::closed_unbound(10);

        assert_eq!((&a).merge_connected(&b), None);
        assert_eq!(a.merge_connected(b), None);

        let c = EnumInterval::unbound_closed(20);
        let expected = Some(EnumInterval::unbounded());
        assert_eq!((&b).merge_connected(&c), expected);
        assert_eq!(b.merge_connected(c), expected);

        assert_eq!((&a).merge_connected(&c), Some(c));
        assert_eq!(a.merge_connected(c), Some(c));
    }

    #[test]
    fn test_finite_half() {
        let a = EnumInterval::closed(0, 10);

        let b = EnumInterval::unbound_closed(5);
        let expected = Some(EnumInterval::unbound_closed(10));
        assert_eq!((&a).merge_connected(&b), expected);
        assert_eq!(a.merge_connected(b), expected);

        let b = EnumInterval::closed_unbound(5);
        let expected = Some(EnumInterval::closed_unbound(0));
        assert_eq!((&a).merge_connected(&b), expected);
        assert_eq!(a.merge_connected(b), expected);

        let b = EnumInterval::closed_unbound(0);
        assert_eq!((&a).merge_connected(&b), Some(b));
        assert_eq!(a.merge_connected(b), Some(b));

        let b = EnumInterval::closed_unbound(15);
        assert_eq!((&a).merge_connected(&b), None);
        assert_eq!(a.merge_connected(b), None);
    }

    extern crate std;

    #[test]
    fn test_merge_sorted_by_value() {
        let mut empty_by_val = MergeSortedByValue::new([FiniteInterval::<u8>::empty()]);
        assert_eq!(empty_by_val.next(), None);

        let finite = [
            FiniteInterval::empty(),
            FiniteInterval::closed(0, 10),
            FiniteInterval::closed(5, 15),
            FiniteInterval::closed(50, 60),
            FiniteInterval::closed(55, 65),
            FiniteInterval::closed(60, 70),
            FiniteInterval::closed(90, 100),
        ];

        let mut finite_by_val = MergeSortedByValue::new(finite);
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(0, 15)));
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(50, 70)));
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(90, 100)));
        assert_eq!(finite_by_val.next(), None);

        let enums = [
            EnumInterval::closed(0, 10),
            EnumInterval::closed(5, 15),
            EnumInterval::closed(50, 60),
            EnumInterval::closed(55, 65),
            EnumInterval::closed(60, 70),
            EnumInterval::closed(90, 100),
        ];

        let mut finite_by_val = MergeSortedByValue::new(enums);
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(0, 15)));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(50, 70)));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(90, 100)));
        assert_eq!(finite_by_val.next(), None);
    }

    #[test]
    fn test_merge_sorted_by_value_merge() {
        let a = std::vec![
            EnumInterval::unbound_closed(-100),
            EnumInterval::closed(0, 10),
            EnumInterval::closed_unbound(100),
        ];

        let b = std::vec![
            EnumInterval::closed(-500, -400),
            EnumInterval::closed(-350, -300),
            EnumInterval::closed(-150, 150),
            EnumInterval::closed(300, 500),
        ];

        let mut finite_by_val = MergeSortedByValue::new(itertools::merge(a, b));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::unbounded()));
        assert_eq!(finite_by_val.next(), None);
    }

    #[test]
    fn test_merge_sorted_by_ref() {
        let enums = [
            EnumInterval::closed(0, 10),
            EnumInterval::closed(5, 15),
            EnumInterval::closed(50, 60),
            EnumInterval::closed(55, 65),
            EnumInterval::closed(60, 70),
            EnumInterval::closed(90, 100),
        ];

        let mut finite_by_ref = MergeSortedByRef::new(enums.iter());
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(0, 15)));
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(50, 70)));
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(90, 100)));
        assert_eq!(finite_by_ref.next(), None);
    }

    // ===== MaybeDisjoint =====

    fn md_pair(a: EnumInterval<i32>, b: EnumInterval<i32>) -> MaybeDisjoint<i32> {
        MaybeDisjoint::from_pair(a, b)
    }

    // ---- MD merge_connected single piece ----

    #[test]
    fn md_disjoint_merges_with_bridging_single() {
        // [0,5] ∪ [10,15] merge [3, 12] = [0, 15]
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let bridge = EnumInterval::closed(3, 12);
        assert_eq!(
            md.merge_connected(bridge),
            Some(EnumInterval::closed(0, 15))
        );
    }

    #[test]
    fn md_disjoint_does_not_merge_with_single_in_gap() {
        // The hull-without-precheck trap: rhs sits in the gap, hull
        // would falsely report a successful merge. The `connects`
        // precheck correctly returns None.
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let in_gap = EnumInterval::closed(7, 8);
        assert_eq!(md.merge_connected(in_gap), None);
    }

    #[test]
    fn md_disjoint_does_not_merge_when_only_one_piece_touches() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let touches_first_only = EnumInterval::closed(3, 7);
        assert_eq!(md.merge_connected(touches_first_only), None);
    }

    #[test]
    fn md_connected_delegates_to_inner() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_i32, 10));
        assert_eq!(
            md.merge_connected(EnumInterval::closed(11, 20)),
            Some(EnumInterval::closed(0, 20))
        );
    }

    // ---- MD merge_connected MD ----

    #[test]
    fn md_md_merge_3_edge_bipartite() {
        // The motivating case from Connects: 3 of 4 bipartite edges, the
        // 4-piece union merges to one interval [0, 20].
        let lhs = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let rhs = md_pair(EnumInterval::closed(3, 12), EnumInterval::closed(13, 20));
        assert_eq!(lhs.merge_connected(rhs), Some(EnumInterval::closed(0, 20)));
    }

    #[test]
    fn md_md_merge_4_edge_bipartite() {
        // Complete bipartite. The 4-piece hull is [-5, 35].
        let lhs = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let rhs = md_pair(EnumInterval::closed(-5, 25), EnumInterval::closed(8, 35));
        assert_eq!(lhs.merge_connected(rhs), Some(EnumInterval::closed(-5, 35)));
    }

    #[test]
    fn md_md_does_not_merge_when_disconnected() {
        // Two-edge non-sharing case from Connects tests: {a,c} and {b,d}
        // form separate components.
        let lhs = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(100, 110));
        let rhs = md_pair(EnumInterval::closed(3, 8), EnumInterval::closed(105, 115));
        assert_eq!(lhs.merge_connected(rhs), None);
    }

    // ---- Commutative ----

    #[test]
    fn single_merge_connected_md_matches_md_merge_connected_single() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let bridge = EnumInterval::closed(3, 12);
        let forward = md.clone().merge_connected(bridge);
        let backward = bridge.merge_connected(md);
        assert_eq!(forward, backward);
        assert_eq!(forward, Some(EnumInterval::closed(0, 15)));
    }

    // ---- Empty handling ----

    #[test]
    fn md_disjoint_merge_with_empty_returns_none() {
        // Per the Connects contract: Disjoint ∪ empty has multiple components.
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        let empty = EnumInterval::empty();
        assert_eq!(md.merge_connected(empty), None);
    }

    #[test]
    fn md_empty_merge_with_single_returns_single() {
        // Connected(empty) ∪ single = single, both connected and merge-able.
        let md = MaybeDisjoint::<i32>::empty();
        let iv = EnumInterval::closed(0, 10);
        assert_eq!(md.merge_connected(iv), Some(EnumInterval::closed(0, 10)));
    }

    // ---- By-ref form matches by-value ----

    #[test]
    fn by_ref_merge_matches_by_value() {
        let md = md_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        let bridge = EnumInterval::closed(3, 12);
        let by_value = md.clone().merge_connected(bridge);
        let by_ref = (&md).merge_connected(&bridge);
        assert_eq!(by_value, by_ref);
    }

    #[test]
    fn by_ref_md_md_merge_matches_by_value() {
        let lhs = md_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        let rhs = md_pair(EnumInterval::closed(3, 12), EnumInterval::closed(13, 20));
        let by_value = lhs.clone().merge_connected(rhs.clone());
        let by_ref = (&lhs).merge_connected(&rhs);
        assert_eq!(by_value, by_ref);
    }

    // ---- Contract verification: connects ⇒ merge_connected.is_some() ----

    #[test]
    fn connects_implies_merge_connected_is_some() {
        let md = md_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        let bridge = EnumInterval::closed(3, 12);
        let in_gap = EnumInterval::closed(7, 8);

        // Positive case
        assert!(md.connects(&bridge));
        assert!(md.clone().merge_connected(bridge).is_some());

        // Negative case — contract doesn't require this direction, but
        // we want it for consistency.
        assert!(!md.connects(&in_gap));
        assert!(md.merge_connected(in_gap).is_none());
    }
}

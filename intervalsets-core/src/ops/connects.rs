use super::util::commutative_predicate_impl;
use super::Intersects;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Test if two sets are connected.
///
/// Sets are connected if the elements of both form an ordered chain of
/// values with no gaps. The empty set is trivially connected to
/// any other interval since no elements exist between the two.
///
/// More formally, two sets are connected if their union can not be
/// partitioned into two or more disjoint non-empty open subsets.
///
/// All intersecting intervals are therefore connected.
///
/// # Contract
///
/// Tier 1 (truly infallible). Must not panic. Predicate-shaped
/// return absorbs incomparability into `false`. See [`crate::ops`]
/// for the full tier model.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(1, 5);
/// let y = FiniteInterval::closed(6, 10);
/// assert_eq!(x.connects(&y), true);
///
/// let x = FiniteInterval::closed(0.0, 5.0);
/// let y = FiniteInterval::closed(6.0, 10.0);
/// assert_eq!(x.connects(&y), false);
///
/// let y = FiniteInterval::open(5.0, 10.0);
/// assert_eq!(x.connects(&y), true);
///
/// let z = FiniteInterval::open(0.0, 5.0);
/// assert_eq!(y.connects(&z), false);
/// ```
pub trait Connects<Rhs = Self> {
    #[allow(missing_docs)]
    fn connects(&self, rhs: Rhs) -> bool;
}

/// Treating each bound as a singleton set, are they connected?
///
/// <--][----> | yes
/// <--)[----> | yes
/// <--](----> | yes
/// <--)(----> | no
/// <--]  [--> | no
/// <--[--]--> | no
#[inline(always)]
pub fn are_bounds_connected<T: Element>(right: &FiniteBound<T>, left: &FiniteBound<T>) -> bool {
    let right_up = right.value().try_adjacent(Side::Right);
    let left_down = left.value().try_adjacent(Side::Left);

    match (right_up, left_down) {
        (None, None) => right.value() == left.value() && (left.is_closed() || right.is_closed()),
        (None, Some(_)) => {
            // Open(T::MAX) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            right.is_closed() && right.value() == left.value()
        }
        (Some(_), None) => {
            // Open(T::MIN) **is** the normalized form since there is no valid
            // closed bit-pattern but the operation needs to be reversible.
            left.is_closed() && right.value() == left.value()
        }
        (Some(right_up), Some(left_down)) => {
            // std normalized comparison. a discrete bound may only be open if
            // it is the MIN/MAX of the data type which is handled by the prev
            // two cases. ie. connects([0, MAX-1], (MAX, ->)) -> should be false.
            right_up == *left.value()
                && left_down == *right.value()
                && left.is_closed()
                && right.is_closed()
        }
    }
}

impl<T: Element> Connects<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        if self.intersects(rhs) {
            return true;
        }

        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return true;
        };

        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return true;
        };

        are_bounds_connected(lhs_max, rhs_min) || are_bounds_connected(rhs_max, lhs_min)
    }
}

impl<T: Element> Connects<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        self.intersects(rhs)
            || match (self.side(), rhs.side()) {
                (Side::Left, Side::Right) => {
                    are_bounds_connected(rhs.finite_bound(), self.finite_bound())
                }
                (Side::Right, Side::Left) => {
                    are_bounds_connected(self.finite_bound(), rhs.finite_bound())
                }
                _ => false,
            }
    }
}

impl<T: Element> Connects<&HalfInterval<T>> for FiniteInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &HalfInterval<T>) -> bool {
        if self.intersects(rhs) {
            return true;
        }

        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return true;
        };

        match rhs.side() {
            Side::Left => are_bounds_connected(lhs_max, rhs.finite_bound()),
            Side::Right => are_bounds_connected(rhs.finite_bound(), lhs_min),
        }
    }
}
commutative_predicate_impl!(Connects, connects, HalfInterval<T>, FiniteInterval<T>);

impl<T: Element> Connects<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.connects(rhs),
            Self::Half(lhs) => lhs.connects(rhs),
            Self::Unbounded => true,
        }
    }
}
commutative_predicate_impl!(Connects, connects, FiniteInterval<T>, EnumInterval<T>);

impl<T: Element> Connects<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.connects(rhs),
            Self::Half(lhs) => lhs.connects(rhs),
            Self::Unbounded => true,
        }
    }
}
commutative_predicate_impl!(Connects, connects, HalfInterval<T>, EnumInterval<T>);

impl<T: Element> Connects<&Self> for EnumInterval<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        match rhs {
            Self::Finite(rhs) => self.connects(rhs),
            Self::Half(rhs) => self.connects(rhs),
            Self::Unbounded => true,
        }
    }
}

// ===== MaybeDisjoint =====

macro_rules! maybe_disjoint_connects_self_impl {
    ($rhs:ty) => {
        impl<T: Element> Connects<$rhs> for MaybeDisjoint<T> {
            #[inline(always)]
            fn connects(&self, rhs: $rhs) -> bool {
                match self {
                    Self::Connected(iv) => iv.connects(rhs),
                    Self::Disjoint(a, b) => {
                        rhs.is_inhabited() && a.connects(rhs) && b.connects(rhs)
                    }
                }
            }
        }
    };
}

maybe_disjoint_connects_self_impl!(&FiniteInterval<T>);
maybe_disjoint_connects_self_impl!(&HalfInterval<T>);
maybe_disjoint_connects_self_impl!(&EnumInterval<T>);

macro_rules! connects_maybe_disjoint_impl {
    ($lhs:ty) => {
        impl<T: Element> Connects<&MaybeDisjoint<T>> for $lhs {
            #[inline(always)]
            fn connects(&self, rhs: &MaybeDisjoint<T>) -> bool {
                match rhs {
                    MaybeDisjoint::Connected(iv) => self.connects(iv),
                    MaybeDisjoint::Disjoint(c, d) => {
                        self.is_inhabited() && self.connects(c) && self.connects(d)
                    }
                }
            }
        }
    };
}

connects_maybe_disjoint_impl!(FiniteInterval<T>);
connects_maybe_disjoint_impl!(HalfInterval<T>);
connects_maybe_disjoint_impl!(EnumInterval<T>);

impl<T: Element> Connects<&Self> for MaybeDisjoint<T> {
    #[inline(always)]
    fn connects(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Connected(a), Self::Connected(b)) => a.connects(b),
            (Self::Connected(a), Self::Disjoint(c, d)) => {
                a.is_inhabited() && a.connects(c) && a.connects(d)
            }
            (Self::Disjoint(a, b), Self::Connected(c)) => {
                c.is_inhabited() && a.connects(c) && b.connects(c)
            }
            (Self::Disjoint(a, b), Self::Disjoint(c, d)) => {
                // Bipartite connectivity on {a,b} | {c,d}: each side
                // must have at least one piece that bridges every piece
                // on the opposite side.
                let lhs_bridges_rhs = rhs.connects(a) || rhs.connects(b);
                let rhs_bridges_lhs = self.connects(c) || self.connects(d);
                lhs_bridges_rhs && rhs_bridges_lhs
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;
    use crate::EnumInterval as EI;

    #[test]
    fn test_connects() {
        assert!(EI::closed(0, 10).connects(&EI::empty()));
        assert!(EI::empty().connects(&EI::closed(0, 10)));

        assert!(EI::closed(0, 10).connects(&EI::closed(10, 20)));
        assert!(EI::closed(0, 10).connects(&EI::closed(11, 20)));
        assert!(!EI::closed(0, 10).connects(&EI::closed(12, 20)));

        assert!(EI::closed(0.0, 10.0).connects(&EI::closed(10.0, 20.0)));
        assert!(EI::closed(0.0, 10.0).connects(&EI::open(10.0, 20.0)));
        assert!(EI::open(0.0, 10.0).connects(&EI::closed(10.0, 20.0)));
        assert!(!EI::open(0.0, 10.0).connects(&EI::open(10.0, 20.0)));

        assert!(EI::open(0.0, 10.0).connects(&EI::closed_unbound(10.0)));

        assert!(EI::unbounded().connects(&EI::closed(0, 10)));
    }

    #[test]
    fn test_connects_discrete_min_max() {
        let a = EI::closed(0, i32::MAX - 1);
        let b = EI::open_unbound(i32::MAX);
        assert!(!a.connects(&b));

        let a = EI::closed(0, i32::MAX);
        let b = EI::open_unbound(i32::MAX);
        assert!(a.connects(&b));

        let a = EI::closed(i32::MIN + 1, 0);
        let b = EI::unbound_open(i32::MIN);
        assert!(!a.connects(&b));

        let a = EI::closed(i32::MIN, 0);
        let b = EI::unbound_open(i32::MIN);
        assert!(a.connects(&b));
    }

    // ===== MaybeDisjoint =====

    fn md_pair(a: EI<i32>, b: EI<i32>) -> MaybeDisjoint<i32> {
        MaybeDisjoint::from_pair(a, b)
    }

    // ---- MD connects single piece ----

    #[test]
    fn md_disjoint_connects_single_that_bridges_gap() {
        // [0,5] ∪ [10,15] connects [3, 12] — bridges the gap (5, 10).
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(md.connects(&EI::closed(3, 12)));
    }

    #[test]
    fn md_disjoint_does_not_connect_single_that_only_touches_one_piece() {
        // [0,5] ∪ [10,15] does not connect [3, 7] — touches first
        // piece, leaves second unreached.
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(!md.connects(&EI::closed(3, 7)));
    }

    #[test]
    fn md_disjoint_does_not_connect_single_in_gap() {
        // [0,5] ∪ [10,15] does not connect [7, 8] — sits entirely in the gap.
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(!md.connects(&EI::closed(7, 8)));
    }

    #[test]
    fn md_disjoint_does_not_connect_empty() {
        // Topology: Disjoint ∪ ∅ = Disjoint, which is not connected.
        // The "empty trivially connects" convention applies only when
        // the surviving operand is itself a single connected piece.
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(!md.connects(&EI::empty()));
    }

    #[test]
    fn md_connected_delegates_to_inner() {
        let md = MaybeDisjoint::from_interval(EI::closed(0, 10));
        assert!(md.connects(&EI::closed(11, 20)));
        assert!(!md.connects(&EI::closed(15, 20)));
    }

    #[test]
    fn md_empty_connects_empty_only() {
        // Connected(empty).connects(empty_iv) delegates to the inner
        // single-piece "empty trivially connects" rule.
        let md = MaybeDisjoint::<i32>::empty();
        assert!(md.connects(&EI::empty()));
    }

    // ---- single connects MD (commutative) ----

    #[test]
    fn single_connects_md_when_it_bridges() {
        let bridge = EI::closed(3, 12);
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(bridge.connects(&md));
        assert_eq!(bridge.connects(&md), md.connects(&bridge));
    }

    #[test]
    fn empty_does_not_connect_md_disjoint() {
        let empty = EI::<i32>::empty();
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(!empty.connects(&md));
    }

    // ---- MD-MD: bipartite cases ----

    #[test]
    fn md_md_4_edge_connected() {
        // Both lhs pieces overlap with both rhs pieces (complete bipartite).
        let lhs = md_pair(EI::closed(0, 10), EI::closed(20, 30));
        let rhs = md_pair(EI::closed(-5, 25), EI::closed(8, 35));
        assert!(lhs.connects(&rhs));
    }

    #[test]
    fn md_md_3_edge_connected() {
        // The motivating case: 3 of 4 bipartite edges, still connected.
        // self = [0,5] ∪ [10,15], rhs = [3,12] ∪ [13,20].
        // a-c=T (overlap [3,5]), a-d=F (gap (5,13)), b-c=T (overlap [10,12]),
        // b-d=T (overlap [13,15]). Union merges to [0,20].
        let lhs = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        let rhs = md_pair(EI::closed(3, 12), EI::closed(13, 20));
        assert!(lhs.connects(&rhs));
    }

    #[test]
    fn md_md_2_edge_non_sharing_not_connected() {
        // a-c, b-d only → two components {a,c} and {b,d}.
        let lhs = md_pair(EI::closed(0, 5), EI::closed(100, 110));
        let rhs = md_pair(EI::closed(3, 8), EI::closed(105, 115));
        // a=[0,5] connects c=[3,8] ✓; a vs d=[105,115] no; b=[100,110] vs c=[3,8] no; b vs d ✓.
        assert!(!lhs.connects(&rhs));
    }

    #[test]
    fn md_md_2_edge_sharing_not_connected() {
        // a-c, a-d (both rhs pieces touch a, but b is isolated).
        let lhs = md_pair(EI::closed(0, 10), EI::closed(100, 110));
        let rhs = md_pair(EI::closed(3, 5), EI::closed(7, 9));
        // a=[0,10] connects both c and d (overlap). b=[100,110] connects neither.
        // Components: {a, c, d} and {b}.
        assert!(!lhs.connects(&rhs));
    }

    #[test]
    fn md_md_disjoint_vs_connected_in_lhs_gap() {
        // self = [0,5] ∪ [10,15], rhs = Connected([7, 8]).
        // rhs in the gap, no bridging.
        let lhs = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        let rhs = MaybeDisjoint::from_interval(EI::closed(7, 8));
        assert!(!lhs.connects(&rhs));
    }

    #[test]
    fn md_md_disjoint_vs_connected_bridging() {
        // rhs is the single bridge piece.
        let lhs = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        let rhs = MaybeDisjoint::from_interval(EI::closed(3, 12));
        assert!(lhs.connects(&rhs));
    }

    #[test]
    fn md_md_connected_connected_delegates() {
        let a = MaybeDisjoint::from_interval(EI::closed(0, 5));
        let b = MaybeDisjoint::from_interval(EI::closed(6, 10));
        assert!(a.connects(&b));

        let c = MaybeDisjoint::from_interval(EI::closed(8, 15));
        assert!(!a.connects(&c));
    }

    #[test]
    fn md_md_empty_with_disjoint() {
        // Connected(empty) ∪ Disjoint(c, d) = Disjoint(c, d) — not connected.
        let empty = MaybeDisjoint::<i32>::empty();
        let md = md_pair(EI::closed(0, 5), EI::closed(10, 15));
        assert!(!empty.connects(&md));
        assert!(!md.connects(&empty));
    }
}

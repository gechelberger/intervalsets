use core::cmp::Ordering;

use super::{EnumInterval, FiniteInterval, HalfInterval};
use crate::bound::ord::OrdBoundPair;
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::ops::{Connects, MergeConnected};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawMaybeDisjoint<T>"))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))
)]
pub enum MaybeDisjoint<T> {
    #[non_exhaustive]
    Connected(EnumInterval<T>),
    #[non_exhaustive]
    Disjoint(EnumInterval<T>, EnumInterval<T>),
}

/// Wire-format mirror of [`MaybeDisjoint`] used to drive validation
/// during `Deserialize`. Identical layout, no invariants — the
/// `TryFrom` proxy below enforces the `Disjoint`-pair invariants.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "MaybeDisjoint")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
enum RawMaybeDisjoint<T> {
    Connected(EnumInterval<T>),
    Disjoint(EnumInterval<T>, EnumInterval<T>),
}

#[cfg(feature = "serde")]
impl<T: Element> TryFrom<RawMaybeDisjoint<T>> for MaybeDisjoint<T> {
    type Error = crate::error::Error;

    fn try_from(raw: RawMaybeDisjoint<T>) -> Result<Self, Self::Error> {
        match raw {
            // Connected wraps a pre-validated EnumInterval; no further
            // checks needed.
            RawMaybeDisjoint::Connected(inner) => Ok(Self::Connected(inner)),
            // Disjoint requires the cross-piece invariants: both pieces
            // non-empty, sorted ascending, non-connecting. Reject
            // (strict) rather than normalize — a hand-crafted payload
            // that violates the shape isn't something the serializer
            // would emit.
            RawMaybeDisjoint::Disjoint(a, b) => {
                if Self::satisfies_invariants(&a, &b) {
                    Ok(Self::Disjoint(a, b))
                } else {
                    Err(crate::error::Error::InvalidBoundPair)
                }
            }
        }
    }
}

// PartialOrd/Ord: lexicographic on the bound sequence of pieces, in
// piece order. Each piece compares via `EnumInterval`'s `OrdBoundPair`-
// based `Ord`, so two pieces flatten to `(left, right)` lex. The empty
// set has bound pair `(LeftUnbounded, LeftUnbounded)` — the minimum
// `OrdBoundPair` — so `Connected(EnumInterval::empty())` correctly sits
// below every inhabited value without special-casing. `Connected` vs
// `Disjoint` at matching first piece tie-breaks to `Connected < Disjoint`
// (shorter sequence is lex-less).
//
// Generalizes `OrdBoundPair`'s order to multi-piece sets. Replaces the
// derived variant-tag ordering, which compared `Connected` < `Disjoint`
// by enum discriminant regardless of contents.
impl<T: Ord> Ord for MaybeDisjoint<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Connected(a), Self::Connected(b)) => a.cmp(b),
            (Self::Disjoint(a1, a2), Self::Disjoint(b1, b2)) => a1.cmp(b1).then_with(|| a2.cmp(b2)),
            (Self::Connected(a), Self::Disjoint(b1, _)) => a.cmp(b1).then(Ordering::Less),
            (Self::Disjoint(a1, _), Self::Connected(b)) => a1.cmp(b).then(Ordering::Greater),
        }
    }
}

impl<T: PartialOrd> PartialOrd for MaybeDisjoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let tiebreak = |ord: Option<Ordering>, fallback: Ordering| match ord? {
            Ordering::Equal => Some(fallback),
            o => Some(o),
        };
        match (self, other) {
            (Self::Connected(a), Self::Connected(b)) => a.partial_cmp(b),
            (Self::Disjoint(a1, a2), Self::Disjoint(b1, b2)) => match a1.partial_cmp(b1)? {
                Ordering::Equal => a2.partial_cmp(b2),
                o => Some(o),
            },
            (Self::Connected(a), Self::Disjoint(b1, _)) => {
                tiebreak(a.partial_cmp(b1), Ordering::Less)
            }
            (Self::Disjoint(a1, _), Self::Connected(b)) => {
                tiebreak(a1.partial_cmp(b), Ordering::Greater)
            }
        }
    }
}

/// Owning iterator over the pieces of a [`MaybeDisjoint`].
///
/// Created by [`MaybeDisjoint::into_iter`]. Yields at most two
/// non-empty [`EnumInterval`]s. After exhaustion, further `.next()`
/// calls return `None`.
pub struct MaybeDisjointIntoIter<T> {
    md: MaybeDisjoint<T>,
}

impl<T> Iterator for MaybeDisjointIntoIter<T> {
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.md.take_min();
        next.is_inhabited().then_some(next)
    }
}

impl<T> core::iter::FusedIterator for MaybeDisjointIntoIter<T> {}

impl<T> IntoIterator for MaybeDisjoint<T> {
    type Item = EnumInterval<T>;
    type IntoIter = MaybeDisjointIntoIter<T>;

    /// Yields the non-empty pieces of this `MaybeDisjoint` in order.
    /// Used as `IntoIterator` rather than `Iterator` directly so the
    /// iterator-trait methods (`cmp`, `partial_cmp`, `count`, `lt`,
    /// `gt`, etc.) don't shadow this type's `Ord`/`PartialOrd`/`Count`
    /// trait impls during method resolution.
    fn into_iter(self) -> Self::IntoIter {
        MaybeDisjointIntoIter { md: self }
    }
}

impl<T: Element> MaybeDisjoint<T> {
    /// Create a new MaybeDisjoint from two optional EnumIntervals.
    ///
    /// Invariants are applied.
    pub fn new(a: Option<EnumInterval<T>>, b: Option<EnumInterval<T>>) -> Self {
        match (a, b) {
            (None, None) => Self::empty(),
            (Some(interval), None) | (None, Some(interval)) => Self::from_interval(interval),
            (Some(a), Some(b)) => Self::from_pair(a, b),
        }
    }

    /// Create a new `MaybeDisjoint` from two EnumIntervals and repairs invariants.
    pub fn from_pair(a: EnumInterval<T>, b: EnumInterval<T>) -> Self {
        if a.connects(&b) {
            match a.merge_connected(b) {
                Some(merged) => Self::from_interval(merged),
                None => unreachable!("connects() implies merge_connected returns Some"),
            }
        } else {
            // the empty set connects trivially with all other sets,
            // so both a, and b must be inhabited, disjoint intervals.
            if a < b {
                MaybeDisjoint::Disjoint(a, b)
            } else {
                MaybeDisjoint::Disjoint(b, a)
            }
        }
    }

    pub(crate) fn new_disjoint_assume_valid(left: EnumInterval<T>, right: EnumInterval<T>) -> Self {
        debug_assert!(Self::satisfies_invariants(&left, &right));
        Self::Disjoint(left, right)
    }

    /// Returns `true` if `(left, right)` is a well-formed `Disjoint`
    /// pair: both non-empty, sorted, and non-connecting.
    pub(crate) fn satisfies_invariants(left: &EnumInterval<T>, right: &EnumInterval<T>) -> bool {
        !left.is_empty() && !right.is_empty() && left < right && !left.connects(right)
    }
}

impl<T> MaybeDisjoint<T> {
    pub fn empty() -> Self {
        Self::Connected(EnumInterval::empty())
    }

    pub fn from_interval(interval: EnumInterval<T>) -> Self {
        Self::Connected(interval)
    }

    /// take the first interval or empty; removes it from the set.
    pub fn take_min(&mut self) -> EnumInterval<T> {
        let mut inst = Self::Connected(EnumInterval::empty());
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Connected(interval) => interval,
            Self::Disjoint(lo, hi) => {
                *self = Self::Connected(hi);
                lo
            }
        }
    }

    /// take the greatest interval or empty; removes from the set.
    pub fn take_max(&mut self) -> EnumInterval<T> {
        let mut inst = Self::Connected(EnumInterval::empty());
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Connected(interval) => interval,
            Self::Disjoint(lo, hi) => {
                *self = Self::Connected(lo);
                hi
            }
        }
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; returns `None` if this is two disjoint intervals.
    pub fn into_interval(self) -> Option<EnumInterval<T>> {
        match self {
            Self::Connected(interval) => Some(interval),
            Self::Disjoint(_, _) => None,
        }
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; panics otherwise.
    ///
    /// # Panics
    ///
    /// Panics if this is two disjoint intervals. Use
    /// [`into_interval`](Self::into_interval) for a panic-free alternative.
    pub fn expect_interval(self) -> EnumInterval<T> {
        self.into_interval()
            .expect("expected a single connected interval")
    }
}

impl<T: Element> MaybeDisjoint<T> {
    /// Returns the convex hull as an [`EnumInterval`], consuming `self`.
    ///
    /// `Connected(iv)` returns `iv` directly. `Disjoint(a, b)` returns
    /// the interval spanning `a`'s left bound to `b`'s right bound —
    /// the gap between the two pieces is filled in.
    pub fn into_hull(self) -> EnumInterval<T> {
        match self {
            Self::Connected(interval) => interval,
            Self::Disjoint(a, b) => {
                let (a_left, _) = OrdBoundPair::from(a).into_raw();
                let (_, b_right) = OrdBoundPair::from(b).into_raw();
                // `Disjoint(a, b)` guarantees `a < b` and per-piece
                // invariants, so the resulting pair is well-formed and
                // `try_from` cannot fail.
                match EnumInterval::try_from(OrdBoundPair::new_assume_valid(a_left, b_right)) {
                    Ok(hull) => hull,
                    Err(_) => {
                        unreachable!("Disjoint invariants guarantee a valid convex hull")
                    }
                }
            }
        }
    }
}

impl<T: Element + Clone> MaybeDisjoint<T> {
    /// Returns the convex hull as an [`EnumInterval`], borrowing `self`.
    ///
    /// See [`into_hull`](Self::into_hull) for the consuming variant.
    pub fn hull(&self) -> EnumInterval<T> {
        match self {
            Self::Connected(interval) => interval.clone(),
            Self::Disjoint(a, b) => {
                let (a_left, _) = OrdBoundPair::from(a).into_raw();
                let (_, b_right) = OrdBoundPair::from(b).into_raw();
                let pair = OrdBoundPair::new_assume_valid(a_left.cloned(), b_right.cloned());
                match EnumInterval::try_from(pair) {
                    Ok(hull) => hull,
                    Err(_) => {
                        unreachable!("Disjoint invariants guarantee a valid convex hull")
                    }
                }
            }
        }
    }
}

impl<T> From<FiniteInterval<T>> for MaybeDisjoint<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<HalfInterval<T>> for MaybeDisjoint<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<EnumInterval<T>> for MaybeDisjoint<T> {
    fn from(interval: EnumInterval<T>) -> Self {
        Self::from_interval(interval)
    }
}

impl<T> Default for MaybeDisjoint<T> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    fn empty<T>() -> MaybeDisjoint<T> {
        MaybeDisjoint::empty()
    }

    fn connected<T: Element>(a: T, b: T) -> MaybeDisjoint<T> {
        MaybeDisjoint::from_interval(EnumInterval::closed(a, b))
    }

    fn two<T: Element>(a: T, b: T, c: T, d: T) -> MaybeDisjoint<T> {
        MaybeDisjoint::from_pair(EnumInterval::closed(a, b), EnumInterval::closed(c, d))
    }

    // ---- equality / reflexivity ----

    #[test]
    fn empty_equals_empty() {
        assert_eq!(empty::<i32>().cmp(&empty()), Ordering::Equal);
    }

    #[test]
    fn equal_disjoint_sets_compare_equal() {
        assert_eq!(
            two(0_i32, 5, 10, 20).cmp(&two(0, 5, 10, 20)),
            Ordering::Equal
        );
    }

    // ---- empty is the minimum ----

    #[test]
    fn empty_less_than_connected_nonempty() {
        assert!(empty::<i32>() < connected(0, 5));
    }

    #[test]
    fn empty_less_than_disjoint() {
        assert!(empty::<i32>() < two(0, 5, 10, 20));
    }

    // ---- prefix-shorter loses ----

    #[test]
    fn connected_less_than_disjoint_at_matching_first_piece() {
        // Same first piece [0, 5]; Disjoint extends. Lex: shorter < longer.
        let c = connected(0_i32, 5);
        let d = two(0, 5, 10, 20);
        assert!(c < d);
    }

    // ---- the counterexample motivating the rewrite ----

    #[test]
    fn disjoint_distinguishes_inner_bounds() {
        // Hull-only ordering would compare these equal (both have outer
        // hull [0, 20]). Lex-on-pieces distinguishes by the inner bounds.
        // Piece 0: [0,1] vs [0,5] — left ties, right 1 < 5 → first less.
        let small_inner_first = two(0_i32, 1, 10, 20);
        let bigger_first_piece = two(0_i32, 5, 15, 20);
        assert!(small_inner_first < bigger_first_piece);
    }

    // ---- intuitive left-to-right ordering ----

    #[test]
    fn disjoint_with_earlier_leftmost_less_than_connected_with_later_leftmost() {
        // Lex compares position 0 first; [0,1] < [100,200] regardless of
        // what comes after.
        assert!(two(0_i32, 1, 10, 20) < connected(100, 200));
    }

    // ---- contrast with the old derived ordering ----

    #[test]
    fn old_derived_order_would_have_been_wrong() {
        // Under the old `#[derive(Ord)]`, Connected < Disjoint by enum
        // tag, so this would have compared Less. Lex correctly says
        // Greater because Connected's piece [100,200] > Disjoint's
        // leftmost [0,1].
        assert_eq!(
            connected(100_i32, 200).cmp(&two(0, 1, 10, 20)),
            Ordering::Greater
        );
    }

    // ---- transitivity smoke check ----

    #[test]
    fn transitivity_across_variants() {
        let a = empty::<i32>();
        let b = connected(0, 5);
        let c = two(0, 5, 10, 20);
        let d = connected(100, 200);
        assert!(a < b);
        assert!(b < c);
        assert!(c < d);
        assert!(a < d);
    }

    // ---- PartialOrd agrees with Ord on Ord types ----

    #[test]
    fn partial_cmp_matches_cmp_for_ord_types() {
        // For T: Ord, partial_cmp(a, b) == Some(cmp(a, b)). Sanity check
        // that the hand-written PartialOrd impl doesn't diverge.
        let a = two(0_i32, 1, 10, 20);
        let b = two(0_i32, 5, 15, 20);
        assert_eq!(a.partial_cmp(&b), Some(a.cmp(&b)));
        assert_eq!(b.partial_cmp(&a), Some(b.cmp(&a)));
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
    }

    // ---- hull / into_hull ----

    #[test]
    fn into_hull_connected_returns_inner() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(3_i32, 10));
        assert_eq!(md.into_hull(), EnumInterval::closed(3, 10));
    }

    #[test]
    fn into_hull_empty_is_empty_interval() {
        let md = MaybeDisjoint::<i32>::empty();
        assert_eq!(md.into_hull(), EnumInterval::empty());
    }

    #[test]
    fn into_hull_disjoint_spans_outer_bounds() {
        // [0, 5] ∪ [10, 20] → [0, 20]
        let md = two(0_i32, 5, 10, 20);
        assert_eq!(md.into_hull(), EnumInterval::closed(0, 20));
    }

    #[test]
    fn into_hull_disjoint_preserves_outer_bound_kinds() {
        // (0, 5] ∪ [10, 20) → (0, 20)
        let md = MaybeDisjoint::from_pair(
            EnumInterval::open_closed(0_i32, 5),
            EnumInterval::closed_open(10, 20),
        );
        assert_eq!(md.into_hull(), EnumInterval::open(0, 20));
    }

    #[test]
    fn into_hull_disjoint_with_left_unbounded_first_piece() {
        // (<-, 5] ∪ [10, 20] → (<-, 20]
        let md = MaybeDisjoint::from_pair(
            EnumInterval::unbound_closed(5_i32),
            EnumInterval::closed(10, 20),
        );
        assert_eq!(md.into_hull(), EnumInterval::unbound_closed(20));
    }

    #[test]
    fn into_hull_disjoint_with_right_unbounded_second_piece() {
        // [0, 5] ∪ [10, ->) → [0, ->)
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i32, 5),
            EnumInterval::closed_unbound(10),
        );
        assert_eq!(md.into_hull(), EnumInterval::closed_unbound(0));
    }

    #[test]
    fn into_hull_disjoint_with_both_pieces_unbounded_is_unbounded() {
        // (<-, 0] ∪ [10, ->) → unbounded
        let md = MaybeDisjoint::from_pair(
            EnumInterval::unbound_closed(0_i32),
            EnumInterval::closed_unbound(10),
        );
        assert_eq!(md.into_hull(), EnumInterval::unbounded());
    }

    #[test]
    fn hull_by_ref_matches_into_hull() {
        let md = two(0_i32, 5, 10, 20);
        let by_ref = md.hull();
        let by_value = md.into_hull();
        assert_eq!(by_ref, by_value);
    }

    #[test]
    fn hull_does_not_consume() {
        let md = two(0_i32, 5, 10, 20);
        let _ = md.hull();
        // still usable
        assert_eq!(md.hull(), EnumInterval::closed(0, 20));
    }

    // ---- take_min / take_max ----

    #[test]
    fn take_min_drains_disjoint_then_yields_empty() {
        let mut md = two(0_i32, 5, 10, 20);
        assert_eq!(md.take_min(), EnumInterval::closed(0, 5));
        // The right piece is left behind as a Connected.
        assert_eq!(
            md,
            MaybeDisjoint::from_interval(EnumInterval::closed(10, 20))
        );
        assert_eq!(md.take_min(), EnumInterval::closed(10, 20));
        assert_eq!(md, MaybeDisjoint::empty());
        // Drained: further calls yield the empty interval.
        assert_eq!(md.take_min(), EnumInterval::empty());
        assert_eq!(md.take_min(), EnumInterval::empty());
    }

    #[test]
    fn take_max_drains_disjoint_then_yields_empty() {
        let mut md = two(0_i32, 5, 10, 20);
        assert_eq!(md.take_max(), EnumInterval::closed(10, 20));
        // The left piece is left behind as a Connected.
        assert_eq!(md, MaybeDisjoint::from_interval(EnumInterval::closed(0, 5)));
        assert_eq!(md.take_max(), EnumInterval::closed(0, 5));
        assert_eq!(md, MaybeDisjoint::empty());
        assert_eq!(md.take_max(), EnumInterval::empty());
        assert_eq!(md.take_max(), EnumInterval::empty());
    }

    #[test]
    fn take_min_on_connected_returns_inner_and_empties() {
        let mut md = MaybeDisjoint::from_interval(EnumInterval::closed(3_i32, 10));
        assert_eq!(md.take_min(), EnumInterval::closed(3, 10));
        assert_eq!(md, MaybeDisjoint::empty());
    }

    #[test]
    fn take_max_on_connected_returns_inner_and_empties() {
        let mut md = MaybeDisjoint::from_interval(EnumInterval::closed(3_i32, 10));
        assert_eq!(md.take_max(), EnumInterval::closed(3, 10));
        assert_eq!(md, MaybeDisjoint::empty());
    }

    #[test]
    fn take_on_already_empty_yields_empty() {
        let mut md = MaybeDisjoint::<i32>::empty();
        assert_eq!(md.take_min(), EnumInterval::empty());
        assert_eq!(md, MaybeDisjoint::empty());

        let mut md = MaybeDisjoint::<i32>::empty();
        assert_eq!(md.take_max(), EnumInterval::empty());
        assert_eq!(md, MaybeDisjoint::empty());
    }

    // ---- Default ----

    #[test]
    fn default_is_empty() {
        let md: MaybeDisjoint<i32> = Default::default();
        assert_eq!(md, MaybeDisjoint::empty());
    }
}

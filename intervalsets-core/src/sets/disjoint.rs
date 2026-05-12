use core::cmp::Ordering;

use super::{EnumInterval, FiniteInterval, HalfInterval};
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::ops::{Connects, MergeConnected};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaybeDisjoint<T> {
    #[non_exhaustive]
    Connected(EnumInterval<T>),
    #[non_exhaustive]
    Disjoint(EnumInterval<T>, EnumInterval<T>),
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

impl<T> Iterator for MaybeDisjoint<T> {
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // `Connected(EnumInterval::empty())` doubles as the drained
        // sentinel: it denotes ∅ as a set and yields `None` here, so the
        // iterator's exhausted state coincides with the canonical empty
        // value. `EnumInterval::empty()` is a tag-only variant — no `T`
        // is constructed — so the swap is just a discriminant write.
        let mut inst = Self::Connected(EnumInterval::empty());
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Connected(interval) => interval.is_inhabited().then_some(interval),
            Self::Disjoint(lhs, rhs) => {
                *self = Self::Connected(rhs);
                Some(lhs)
            }
        }
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

#[cfg(test)]
mod tests {
    // `Iterator for MaybeDisjoint<T>` shadows the trait methods (`cmp`,
    // `partial_cmp`, `lt`, `<`, ...) when called via method-call syntax,
    // because `Iterator` also defines `cmp`/`partial_cmp` and Rust's
    // method resolution picks the by-value `Iterator` version first.
    // These tests use fully-qualified trait syntax to invoke the
    // `Ord`/`PartialOrd` impls.

    use super::*;
    use crate::factory::FiniteFactory;

    fn cmp<T: Ord>(a: &MaybeDisjoint<T>, b: &MaybeDisjoint<T>) -> Ordering {
        Ord::cmp(a, b)
    }

    fn pcmp<T: PartialOrd>(a: &MaybeDisjoint<T>, b: &MaybeDisjoint<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(a, b)
    }

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
        assert_eq!(cmp(&empty::<i32>(), &empty()), Ordering::Equal);
    }

    #[test]
    fn equal_disjoint_sets_compare_equal() {
        assert_eq!(
            cmp(&two(0_i32, 5, 10, 20), &two(0, 5, 10, 20)),
            Ordering::Equal
        );
    }

    // ---- empty is the minimum ----

    #[test]
    fn empty_less_than_connected_nonempty() {
        assert_eq!(cmp(&empty::<i32>(), &connected(0, 5)), Ordering::Less);
    }

    #[test]
    fn empty_less_than_disjoint() {
        assert_eq!(cmp(&empty::<i32>(), &two(0, 5, 10, 20)), Ordering::Less);
    }

    // ---- prefix-shorter loses ----

    #[test]
    fn connected_less_than_disjoint_at_matching_first_piece() {
        // Same first piece [0, 5]; Disjoint extends. Lex: shorter < longer.
        let c = connected(0_i32, 5);
        let d = two(0, 5, 10, 20);
        assert_eq!(cmp(&c, &d), Ordering::Less);
    }

    // ---- the counterexample motivating the rewrite ----

    #[test]
    fn disjoint_distinguishes_inner_bounds() {
        // Hull-only ordering would compare these equal (both have outer
        // hull [0, 20]). Lex-on-pieces distinguishes by the inner bounds.
        // Piece 0: [0,1] vs [0,5] — left ties, right 1 < 5 → first less.
        let small_inner_first = two(0_i32, 1, 10, 20);
        let bigger_first_piece = two(0_i32, 5, 15, 20);
        assert_eq!(cmp(&small_inner_first, &bigger_first_piece), Ordering::Less);
    }

    // ---- intuitive left-to-right ordering ----

    #[test]
    fn disjoint_with_earlier_leftmost_less_than_connected_with_later_leftmost() {
        // Lex compares position 0 first; [0,1] < [100,200] regardless of
        // what comes after.
        assert_eq!(
            cmp(&two(0_i32, 1, 10, 20), &connected(100, 200)),
            Ordering::Less
        );
    }

    // ---- contrast with the old derived ordering ----

    #[test]
    fn old_derived_order_would_have_been_wrong() {
        // Under the old `#[derive(Ord)]`, Connected < Disjoint by enum
        // tag, so this would have compared Less. Lex correctly says
        // Greater because Connected's piece [100,200] > Disjoint's
        // leftmost [0,1].
        assert_eq!(
            cmp(&connected(100_i32, 200), &two(0, 1, 10, 20)),
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
        assert_eq!(cmp(&a, &b), Ordering::Less);
        assert_eq!(cmp(&b, &c), Ordering::Less);
        assert_eq!(cmp(&c, &d), Ordering::Less);
        assert_eq!(cmp(&a, &d), Ordering::Less);
    }

    // ---- PartialOrd agrees with Ord on Ord types ----

    #[test]
    fn partial_cmp_matches_cmp_for_ord_types() {
        // For T: Ord, partial_cmp(a, b) == Some(cmp(a, b)). Sanity check
        // that the hand-written PartialOrd impl doesn't diverge.
        let a = two(0_i32, 1, 10, 20);
        let b = two(0_i32, 5, 15, 20);
        assert_eq!(pcmp(&a, &b), Some(cmp(&a, &b)));
        assert_eq!(pcmp(&b, &a), Some(cmp(&b, &a)));
        assert_eq!(pcmp(&a, &a), Some(Ordering::Equal));
    }
}

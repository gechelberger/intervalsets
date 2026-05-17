use core::cmp::Ordering::Equal;

use crate::bound::ord::{FiniteOrdBound, OrdBound, OrdBoundPair};
use crate::bound::Side::{Left, Right};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Test if self is a superset of rhs.
///
/// ```text
/// Given: A = self, B = rhs:
/// Test:  ∀ x ∈ B -> x ∈ A
/// Alt:   A ⊇ B
/// ```
///
/// Raw elements are treated as singleton sets.
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
/// let x = FiniteInterval::open(0.0, 10.0);
/// assert_eq!(x.contains(&5.0), true);
/// assert_eq!(x.contains(&10.0), false);
/// assert_eq!(x.contains(&FiniteInterval::open(0.0, 10.0)), true);
/// assert_eq!(x.contains(&FiniteInterval::closed(0.0, 10.0)), false);
/// assert_eq!(x.contains(&FiniteInterval::empty()), true);
/// ```
pub trait Contains<T> {
    /// Test if rhs is a subset of self.
    fn contains(&self, rhs: T) -> bool;
}

impl<T: PartialOrd> Contains<&T> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return false;
        };

        lhs_min.try_contains(Left, rhs).unwrap_or(false)
            && lhs_max.try_contains(Right, rhs).unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<&T> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        self.finite_bound()
            .try_contains(self.side(), rhs)
            .unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<&T> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => rhs.partial_cmp(rhs) == Some(Equal),
        }
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return false;
        };

        let lhs_min = lhs_min.finite_ord(Left);
        let lhs_max = lhs_max.finite_ord(Right);
        lhs_min <= rhs && rhs <= lhs_max
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        let lhs = self.finite_ord_bound();
        match self.side() {
            Left => lhs <= rhs,
            Right => rhs <= lhs,
        }
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => rhs.0.partial_cmp(rhs.0) == Some(Equal),
        }
    }
}

impl<T: PartialOrd> Contains<&T> for OrdBoundPair<&T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let rhs = OrdBound::closed_assume_valid(rhs);
        let (lhs_min, lhs_max) = self.into_raw();
        lhs_min <= rhs && rhs <= lhs_max && lhs_max != OrdBound::LeftUnbounded
    }
}

impl<T: PartialOrd> Contains<&FiniteInterval<T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return false;
        };

        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return true;
        };

        lhs_min.finite_ord(Left) <= rhs_min.finite_ord(Left)
            && rhs_max.finite_ord(Right) <= lhs_max.finite_ord(Right)
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, _rhs: &HalfInterval<T>) -> bool {
        false
    }
}

impl<T: PartialOrd> Contains<&EnumInterval<T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<&FiniteInterval<T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return true;
        };

        let lhs = self.finite_ord_bound();
        match self.side() {
            Left => lhs <= rhs_min.finite_ord(Left), // rhs <= rhs_max transitive
            Right => rhs_max.finite_ord(Right) <= lhs, // rhs_min <= lhs transitive
        }
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        self.side() == rhs.side() && self.contains(rhs.finite_ord_bound())
    }
}

impl<T: PartialOrd> Contains<&EnumInterval<T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true, // ok; EnumInterval invariants ensure comparable.
        }
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true, // ok; HalfInterval invariants ensure comparable.
        }
    }
}

impl<T: PartialOrd> Contains<&EnumInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true, // ok; EnumInterval invariants ensure comparable.
        }
    }
}

// ===== MaybeDisjoint =====
//
// `Contains` is a predicate; cardinality doesn't constrain it. Each
// piece is an `EnumInterval`, so containment delegates piece-by-piece.
//
// - `self: MaybeDisjoint contains rhs` — rhs must lie wholly within
//   some single piece (gaps are not part of the set). Connected
//   intervals can't straddle a gap, so checking "any piece contains
//   rhs" is sufficient.
// - `rhs: &MaybeDisjoint` — every piece of rhs must be contained in
//   self. Single-piece types can still contain a MaybeDisjoint
//   provided every piece of the MaybeDisjoint fits inside them.

macro_rules! maybe_disjoint_is_container_impl {
    ($rhs:ty) => {
        impl<T: PartialOrd> Contains<$rhs> for MaybeDisjoint<T> {
            #[inline(always)]
            fn contains(&self, rhs: $rhs) -> bool {
                match self {
                    Self::Connected(iv) => iv.contains(rhs),
                    Self::Disjoint(a, b) => a.contains(rhs) || b.contains(rhs),
                }
            }
        }
    };
}

maybe_disjoint_is_container_impl!(&T);
maybe_disjoint_is_container_impl!(FiniteOrdBound<&T>);
maybe_disjoint_is_container_impl!(&FiniteInterval<T>);
maybe_disjoint_is_container_impl!(&HalfInterval<T>);
maybe_disjoint_is_container_impl!(&EnumInterval<T>);

macro_rules! contains_maybe_disjoint_impl {
    ($lhs:ty) => {
        impl<T: PartialOrd> Contains<&MaybeDisjoint<T>> for $lhs {
            #[inline(always)]
            fn contains(&self, rhs: &MaybeDisjoint<T>) -> bool {
                match rhs {
                    MaybeDisjoint::Connected(iv) => self.contains(iv),
                    MaybeDisjoint::Disjoint(a, b) => self.contains(a) && self.contains(b),
                }
            }
        }
    };
}

contains_maybe_disjoint_impl!(FiniteInterval<T>);
contains_maybe_disjoint_impl!(HalfInterval<T>);
contains_maybe_disjoint_impl!(EnumInterval<T>);
contains_maybe_disjoint_impl!(MaybeDisjoint<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::*;

    #[test]
    fn test_open_contains_self() {
        let f = FiniteInterval::open(0.0, 10.0);
        assert!(f.contains(&f));

        let h = EnumInterval::unbound_open(10.0);
        assert!(h.contains(&h));
        assert!(h.contains(&f));

        let h = EnumInterval::open_unbound(0.0);
        assert!(h.contains(&h));
        assert!(h.contains(&f));
    }

    #[test]
    fn test_contains_nan() {
        let closed_ord_nan = crate::bound::ord::FiniteOrdBound::closed_assume_valid(&f64::NAN);

        let f = FiniteInterval::open(0.0, 10.0);
        assert!(!f.contains(&f64::NAN));
        assert!(!f.contains(closed_ord_nan));

        let h = EnumInterval::unbound_open(0.0);
        assert!(!h.contains(&f64::NAN));
        assert!(!h.contains(closed_ord_nan));

        let h = EnumInterval::open_unbound(0.0);
        assert!(!h.contains(&f64::NAN));
        assert!(!h.contains(closed_ord_nan));

        let h = EnumInterval::unbounded();
        assert!(!h.contains(&f64::NAN));
        assert!(!h.contains(closed_ord_nan));
    }

    // ===== MaybeDisjoint =====

    fn md_pair(a: EnumInterval<i32>, b: EnumInterval<i32>) -> MaybeDisjoint<i32> {
        MaybeDisjoint::from_pair(a, b)
    }

    #[test]
    fn md_contains_element_in_first_piece() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(md.contains(&3));
    }

    #[test]
    fn md_contains_element_in_second_piece() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(md.contains(&12));
    }

    #[test]
    fn md_does_not_contain_element_in_gap() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(!md.contains(&7));
    }

    #[test]
    fn md_does_not_contain_element_outside() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(!md.contains(&-1));
        assert!(!md.contains(&20));
    }

    #[test]
    fn md_empty_contains_nothing_inhabited() {
        // Matches the existing convention for empty single-piece sets:
        // an empty self contains no inhabited value or interval.
        let empty = MaybeDisjoint::<i32>::empty();
        assert!(!empty.contains(&5));
        assert!(!empty.contains(&EnumInterval::closed(0, 1)));
    }

    #[test]
    fn md_contains_finite_interval_fitting_in_piece() {
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        assert!(md.contains(&FiniteInterval::closed(2, 8)));
        assert!(md.contains(&FiniteInterval::closed(22, 28)));
    }

    #[test]
    fn md_does_not_contain_interval_spanning_gap() {
        // [3, 25] crosses the gap (10, 20); MaybeDisjoint doesn't
        // contain the gap, so this is false even though [3, 25]'s
        // endpoints lie in different pieces.
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        assert!(!md.contains(&FiniteInterval::closed(3, 25)));
    }

    #[test]
    fn md_contains_md_when_every_piece_fits() {
        let outer = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let inner = md_pair(EnumInterval::closed(2, 8), EnumInterval::closed(22, 28));
        assert!(outer.contains(&inner));
    }

    #[test]
    fn md_does_not_contain_md_when_a_piece_escapes() {
        let outer = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        // [22, 35] extends past outer's second piece — overall not contained.
        let inner = md_pair(EnumInterval::closed(2, 8), EnumInterval::closed(22, 35));
        assert!(!outer.contains(&inner));
    }

    #[test]
    fn single_piece_contains_md_when_both_pieces_fit() {
        // A connected interval can contain a MaybeDisjoint because
        // gaps in rhs don't matter — only that every rhs element is
        // in self.
        let outer = FiniteInterval::closed(0, 100);
        let md = md_pair(EnumInterval::closed(10, 20), EnumInterval::closed(50, 60));
        assert!(outer.contains(&md));
    }

    #[test]
    fn single_piece_does_not_contain_md_when_a_piece_escapes() {
        let outer = FiniteInterval::closed(0, 30);
        let md = md_pair(EnumInterval::closed(10, 20), EnumInterval::closed(50, 60));
        assert!(!outer.contains(&md));
    }

    #[test]
    fn enum_interval_unbounded_contains_any_md() {
        let outer = EnumInterval::<i32>::unbounded();
        let md = md_pair(EnumInterval::closed(10, 20), EnumInterval::closed(50, 60));
        assert!(outer.contains(&md));
    }
}

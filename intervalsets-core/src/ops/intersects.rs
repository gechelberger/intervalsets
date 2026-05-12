use super::util::commutative_predicate_impl;
use super::Contains;
//use crate::bound::ord::{OrdBound, OrdBounded};
use crate::bound::Side::{Left, Right};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Test if the intersection of two sets would be non-empty.
///
/// ```text
/// ∃x | x ∈ A ∧ x ∈ B
/// ```
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
/// let x = EnumInterval::closed(10, 20);
/// let y = EnumInterval::closed_unbound(15);
/// assert_eq!(x.intersects(&y), true);
///
/// let y = EnumInterval::empty();
/// assert_eq!(x.intersects(&y), false);
/// ```
pub trait Intersects<T> {
    /// Tests if two sets share any elements.
    fn intersects(&self, rhs: T) -> bool;

    /// Tests if two sets share no elements.
    fn is_disjoint_from(&self, rhs: T) -> bool {
        !self.intersects(rhs)
    }
}

impl<T: PartialOrd> Intersects<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return false;
        };

        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return false;
        };

        lhs_min.finite_ord(Left) <= rhs_max.finite_ord(Right)
            && rhs_min.finite_ord(Left) <= lhs_max.finite_ord(Right)

        /*
        // this is definitely correct but slightly more expensive
        let (lhs_min, lhs_max) = self.ord_bound_pair().into_raw();
        let (rhs_min, rhs_max) = rhs.ord_bound_pair().into_raw();
        lhs_max != OrdBound::LeftUnbounded
            && rhs_max != OrdBound::LeftUnbounded
            && lhs_min <= rhs_max
            && rhs_min <= lhs_max
        */
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for HalfInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        let Some((left, right)) = rhs.view_raw() else {
            return false;
        };

        self.contains(left.finite_ord(Left)) || self.contains(right.finite_ord(Right))
    }
}

impl<T: PartialOrd> Intersects<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        self.contains(rhs.finite_ord_bound()) || rhs.contains(self.finite_ord_bound())
    }
}

impl<T: PartialOrd> Intersects<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.intersects(rhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => *rhs != FiniteInterval::empty(),
        }
    }
}

impl<T: PartialOrd> Intersects<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Intersects<&Self> for EnumInterval<T> {
    #[inline(always)]
    fn intersects(&self, rhs: &Self) -> bool {
        match self {
            Self::Finite(lhs) => rhs.intersects(lhs),
            Self::Half(lhs) => rhs.intersects(lhs),
            Self::Unbounded => *rhs != EnumInterval::empty(),
        }
    }
}

commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, HalfInterval<T>);
commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, EnumInterval<T>);
commutative_predicate_impl!(Intersects, intersects, HalfInterval<T>, EnumInterval<T>);

// ===== MaybeDisjoint =====

macro_rules! maybe_disjoint_intersects_self_impl {
    ($rhs:ty) => {
        impl<T: crate::numeric::Element> Intersects<$rhs> for MaybeDisjoint<T> {
            #[inline(always)]
            fn intersects(&self, rhs: $rhs) -> bool {
                match self {
                    Self::Connected(iv) => iv.intersects(rhs),
                    Self::Disjoint(a, b) => a.intersects(rhs) || b.intersects(rhs),
                }
            }
        }
    };
}

maybe_disjoint_intersects_self_impl!(&FiniteInterval<T>);
maybe_disjoint_intersects_self_impl!(&HalfInterval<T>);
maybe_disjoint_intersects_self_impl!(&EnumInterval<T>);

macro_rules! intersects_maybe_disjoint_impl {
    ($lhs:ty) => {
        impl<T: crate::numeric::Element> Intersects<&MaybeDisjoint<T>> for $lhs {
            #[inline(always)]
            fn intersects(&self, rhs: &MaybeDisjoint<T>) -> bool {
                match rhs {
                    MaybeDisjoint::Connected(iv) => self.intersects(iv),
                    MaybeDisjoint::Disjoint(a, b) => self.intersects(a) || self.intersects(b),
                }
            }
        }
    };
}

intersects_maybe_disjoint_impl!(FiniteInterval<T>);
intersects_maybe_disjoint_impl!(HalfInterval<T>);
intersects_maybe_disjoint_impl!(EnumInterval<T>);
intersects_maybe_disjoint_impl!(MaybeDisjoint<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_open_intersects_self() {
        let x = FiniteInterval::open(0.0, 10.0);
        assert!(x.intersects(&x));

        let y = HalfInterval::unbound_open(0.0);
        assert!(y.intersects(&y));

        assert!(x.is_disjoint_from(&y));
    }

    #[test]
    fn test_open_disjoint() {
        let a = FiniteInterval::open(0.0, 10.0);
        let b = FiniteInterval::open(10.0, 20.0);

        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));

        let hb = HalfInterval::open_unbound(10.0);
        assert!(!a.intersects(&hb));
        assert!(!hb.intersects(&a));

        let ha = HalfInterval::unbound_open(0.0);
        assert!(!a.intersects(&ha));
        assert!(!ha.intersects(&a));
    }

    // ===== MaybeDisjoint =====

    fn md_pair(a: EnumInterval<i32>, b: EnumInterval<i32>) -> MaybeDisjoint<i32> {
        MaybeDisjoint::from_pair(a, b)
    }

    #[test]
    fn md_intersects_single_via_first_piece() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(md.intersects(&EnumInterval::closed(3, 7)));
    }

    #[test]
    fn md_intersects_single_via_second_piece() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(md.intersects(&EnumInterval::closed(12, 20)));
    }

    #[test]
    fn md_does_not_intersect_single_in_gap() {
        // [0,5] ∪ [10,15] vs [7, 8] — fits entirely in the gap.
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(!md.intersects(&EnumInterval::closed(7, 8)));
    }

    #[test]
    fn md_does_not_intersect_single_outside() {
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert!(!md.intersects(&EnumInterval::closed(20, 30)));
    }

    #[test]
    fn md_empty_intersects_nothing() {
        let empty = MaybeDisjoint::<i32>::empty();
        assert!(!empty.intersects(&EnumInterval::closed(0, 10)));
    }

    #[test]
    fn md_intersects_md_when_any_piece_pair_overlaps() {
        // [0,5] ∪ [20,25] vs [4,6] ∪ [30,35] — first piece overlaps at 4-5.
        let a = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(20, 25));
        let b = md_pair(EnumInterval::closed(4, 6), EnumInterval::closed(30, 35));
        assert!(a.intersects(&b));
    }

    #[test]
    fn md_does_not_intersect_md_when_all_piece_pairs_disjoint() {
        // Pieces interleave but never overlap.
        let a = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(20, 25));
        let b = md_pair(EnumInterval::closed(10, 15), EnumInterval::closed(30, 35));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn md_intersects_md_cross_piece_overlap() {
        // self's second piece overlaps with rhs's first piece.
        let a = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(20, 30));
        let b = md_pair(EnumInterval::closed(40, 50), EnumInterval::closed(25, 35));
        assert!(a.intersects(&b));
    }

    #[test]
    fn commutative_single_intersects_md() {
        let single = EnumInterval::closed(3, 7);
        let md = md_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert_eq!(single.intersects(&md), md.intersects(&single));
        assert!(single.intersects(&md));
    }

    #[test]
    fn is_disjoint_from_negation() {
        let md = md_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 15));
        let in_gap = EnumInterval::closed(7, 8);
        let overlap = EnumInterval::closed(3, 7);
        assert!(md.is_disjoint_from(&in_gap));
        assert!(!md.is_disjoint_from(&overlap));
    }
}

use super::util::commutative_predicate_impl;
use super::Contains;
//use crate::bound::ord::{OrdBound, OrdBounded};
use crate::bound::Side::{Left, Right};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if the intersection of two sets would be non-empty.
///
/// ```text
/// ∃x | x ∈ A ∧ x ∈ B
/// ```
///
/// # Contract
///
/// Intersects should be usable with strict api calls, therefore it should
/// not panic. Since it is only testing between instantiated sets, comparability
/// is already addressed by set invariants and should not be a problem.
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
        let Self::Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let Self::Bounded(rhs_min, rhs_max) = rhs else {
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
        let FiniteInterval::Bounded(left, right) = rhs else {
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
            Self::Unbounded => *rhs != FiniteInterval::Empty,
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
            Self::Unbounded => *rhs != FiniteInterval::Empty.into(),
        }
    }
}

commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, HalfInterval<T>);
commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, EnumInterval<T>);
commutative_predicate_impl!(Intersects, intersects, HalfInterval<T>, EnumInterval<T>);

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

        assert_eq!(a.intersects(&b), false);
        assert_eq!(b.intersects(&a), false);

        let hb = HalfInterval::open_unbound(10.0);
        assert_eq!(a.intersects(&hb), false);
        assert_eq!(hb.intersects(&a), false);

        let ha = HalfInterval::unbound_open(0.0);
        assert_eq!(a.intersects(&ha), false);
        assert_eq!(ha.intersects(&a), false);
    }
}

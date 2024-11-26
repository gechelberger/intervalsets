use core::cmp::Ordering::Equal;

use crate::bound::ord::{FiniteOrdBound, OrdBound, OrdBoundPair};
use crate::bound::Side::{Left, Right};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if self is a superset of rhs.
///
/// ```text
/// Given: A = self, B = rhs:
/// Test:  ∀ x ∈ B -> x ∈ A
/// Alt:   A ⊇ B
/// ```
///
/// Individual elements are treated as if they were a singleton set.
///
/// # Contract
///
/// Contains should be useable in strict api calls. Therefore, it should not
/// panic and it should always return false for incomparable arguments.
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
    /// Test if rhs is fully contained.
    fn contains(&self, rhs: T) -> bool;
}

impl<T: PartialOrd> Contains<&T> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return false;
        };

        lhs_min.strict_contains(Left, rhs).unwrap_or(false)
            && lhs_max.strict_contains(Right, rhs).unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<&T> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        self.finite_bound()
            .strict_contains(self.side(), rhs)
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
        let rhs = OrdBound::closed(rhs);
        let (lhs_min, lhs_max) = self.into_raw();
        lhs_min <= rhs && rhs <= lhs_max && lhs_max != OrdBound::LeftUnbounded
    }
}

impl<T: PartialOrd> Contains<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
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

impl<T: PartialOrd> Contains<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
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
            Self::Unbounded => true, // ok; set type invariants ensure comparable.
        }
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true, // ok; set type invariants ensure comparable.
        }
    }
}

impl<T: PartialOrd> Contains<&Self> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true, // ok; set type invariants ensure comparable.
        }
    }
}

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
        let closed_ord_nan = crate::bound::ord::FiniteOrdBound::closed(&f64::NAN);

        let f = FiniteInterval::open(0.0, 10.0);
        assert_eq!(f.contains(&f64::NAN), false);
        assert_eq!(f.contains(closed_ord_nan), false);

        let h = EnumInterval::unbound_open(0.0);
        assert_eq!(h.contains(&f64::NAN), false);
        assert_eq!(h.contains(closed_ord_nan), false);

        let h = EnumInterval::open_unbound(0.0);
        assert_eq!(h.contains(&f64::NAN), false);
        assert_eq!(h.contains(closed_ord_nan), false);

        let h = EnumInterval::unbounded();
        assert_eq!(h.contains(&f64::NAN), false);
        assert_eq!(h.contains(closed_ord_nan), false);
    }
}

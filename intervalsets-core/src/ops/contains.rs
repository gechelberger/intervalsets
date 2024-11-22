use FiniteInterval::Bounded;

use crate::bound::ord::{FiniteOrdBound, OrdBound, OrdBoundPair};
use crate::bound::Side;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test if this `Set` fully contains `T`.
///
/// ```text
/// Given A = self, B = rhs:
/// ∀ x ∈ B -> x ∈ A
/// ```
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
/// ```
pub trait Contains<T> {
    /// Test if rhs is fully contained.
    fn contains(&self, rhs: T) -> bool;
}

impl<T: PartialOrd> Contains<&T> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        lhs_min.strict_contains(Side::Left, rhs).unwrap_or(false)
            && lhs_max.strict_contains(Side::Right, rhs).unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<&T> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        self.bound.strict_contains(self.side, rhs).unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<&T> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let lhs_min = lhs_min.finite_ord(Side::Left);
        let lhs_max = lhs_max.finite_ord(Side::Right);
        lhs_min <= rhs && rhs <= lhs_max
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        let lhs = self.bound.finite_ord(self.side);
        match self.side {
            Side::Left => lhs <= rhs,
            Side::Right => rhs <= lhs,
        }
    }
}

impl<T: PartialOrd> Contains<FiniteOrdBound<&T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: FiniteOrdBound<&T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

/*
impl<T: PartialOrd> Contains<OrdBound<&T>> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: OrdBound<&T>) -> bool {
        let (lhs_min, lhs_max) = self.ord_bound_pair().into_raw();
        lhs_min <= rhs && rhs <= lhs_max && lhs_max != OrdBound::LeftUnbounded // lhs empty
    }
}

impl<T: PartialOrd> Contains<OrdBound<&T>> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: OrdBound<&T>) -> bool {
        let lhs = self.bound.ord(self.side);
        match self.side {
            Side::Left => lhs <= rhs,
            Side::Right => rhs <= lhs,
        }
    }
}

impl<T: PartialOrd> Contains<OrdBound<&T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: OrdBound<&T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}*/

impl<T: PartialOrd> Contains<&T> for OrdBoundPair<&T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let rhs = OrdBound::closed(rhs);
        let (lhs_min, lhs_max) = self.into_raw();
        lhs_min <= rhs && rhs <= lhs_max && lhs_max != OrdBound::LeftUnbounded
    }
}

/*
#[inline(always)]
fn ord_bound_pair_contains<T: PartialOrd>(lhs: OrdBoundPair<T>, rhs: OrdBoundPair<T>) -> bool {
    let (lhs_min, lhs_max) = lhs.into_raw();
    let (rhs_min, rhs_max) = rhs.into_raw();

    lhs_min <= rhs_min
        && rhs_max <= lhs_max
        && lhs_max != OrdBound::LeftUnbounded
        && rhs_max != OrdBound::LeftUnbounded
}*/

impl<T: PartialOrd> Contains<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        // SAFETY: lhs_min <= lhs_max && rhs_min <= rhs_max so all are comparable.
        unsafe {
            lhs_min.contains_bound_unchecked(Side::Left, rhs_min)
                && lhs_max.contains_bound_unchecked(Side::Right, rhs_max)
        }

        //lhs_min.finite_ord(Side::Left) <= rhs_min.finite_ord(Side::Left)
        //    && rhs_max.finite_ord(Side::Right) <= lhs_max.finite_ord(Side::Right)
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
        let Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        self.contains(rhs_min.finite_ord(Side::Left))
            && self.contains(rhs_max.finite_ord(Side::Right))
    }
}

impl<T: PartialOrd> Contains<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(rhs.bound.finite_ord(rhs.side))

        // SAFETY: invariants already satisfied
        // unsafe {
        //     self.side == rhs.side
        //         && self.bound.contains_bound_unchecked(self.side, &rhs.bound)
        // }
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
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<&Self> for EnumInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => match rhs {
                Self::Finite(rhs) => self.contains(rhs),
                Self::Half(rhs) => self.contains(rhs),
                Self::Unbounded => true,
            },
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
}

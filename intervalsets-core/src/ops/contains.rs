use FiniteInterval::Bounded;

use crate::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
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

        lhs_min.contains(Side::Left, rhs) && lhs_max.contains(Side::Right, rhs)
    }
}

impl<T: PartialOrd> Contains<&T> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        self.bound.contains(self.side, rhs)
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
}

impl<T: PartialOrd> Contains<&T> for OrdBoundPair<&T> {
    #[inline(always)]
    fn contains(&self, rhs: &T) -> bool {
        let rhs = OrdBound::closed(rhs);
        let (lhs_min, lhs_max) = self.into_raw();
        lhs_min <= rhs && rhs <= lhs_max && lhs_max != OrdBound::LeftUnbounded
    }
}

#[inline(always)]
fn ord_bound_pair_contains<T: PartialOrd>(lhs: OrdBoundPair<T>, rhs: OrdBoundPair<T>) -> bool {
    let (lhs_min, lhs_max) = lhs.into_raw();
    let (rhs_min, rhs_max) = rhs.into_raw();

    lhs_min <= rhs_min
        && rhs_max <= lhs_max
        && lhs_max != OrdBound::LeftUnbounded
        && rhs_max != OrdBound::LeftUnbounded
}

impl<T: PartialOrd> Contains<&Self> for FiniteInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        ord_bound_pair_contains(self.ord_bound_pair(), rhs.ord_bound_pair())
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
        ord_bound_pair_contains(self.ord_bound_pair(), rhs.ord_bound_pair())
    }
}

impl<T: PartialOrd> Contains<&Self> for HalfInterval<T> {
    #[inline(always)]
    fn contains(&self, rhs: &Self) -> bool {
        // left to possibly short circuit before the more expensive checks
        self.side == rhs.side
            && ord_bound_pair_contains(self.ord_bound_pair(), rhs.ord_bound_pair())
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

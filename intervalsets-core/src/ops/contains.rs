use FiniteInterval::Bounded;

use super::Contains;
use crate::bound::Side;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: PartialOrd> Contains<&T> for FiniteInterval<T> {
    #[inline]
    fn contains(&self, rhs: &T) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        lhs_min.contains(Side::Left, rhs) && lhs_max.contains(Side::Right, rhs)
    }
}

impl<T: PartialOrd> Contains<&Self> for FiniteInterval<T> {
    #[inline]
    fn contains(&self, rhs: &Self) -> bool {
        let Bounded(lhs_min, lhs_max) = self else {
            return false;
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        lhs_min.contains(Side::Left, rhs_min.value())
            && lhs_max.contains(Side::Right, rhs_max.value())
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for FiniteInterval<T> {
    #[inline]
    fn contains(&self, _rhs: &HalfInterval<T>) -> bool {
        false
    }
}

impl<T: PartialOrd> Contains<&EnumInterval<T>> for FiniteInterval<T> {
    #[inline]
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<&T> for HalfInterval<T> {
    #[inline]
    fn contains(&self, rhs: &T) -> bool {
        self.bound.contains(self.side, rhs)
    }
}

impl<T: PartialOrd> Contains<&FiniteInterval<T>> for HalfInterval<T> {
    #[inline]
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        let Bounded(rhs_min, rhs_max) = rhs else {
            return false;
        };

        self.contains(rhs_min.value()) && self.contains(rhs_max.value())
    }
}

impl<T: PartialOrd> Contains<&Self> for HalfInterval<T> {
    #[inline]
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(rhs.bound.value())
    }
}

impl<T: PartialOrd> Contains<&EnumInterval<T>> for HalfInterval<T> {
    #[inline]
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<&T> for EnumInterval<T> {
    #[inline]
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<&FiniteInterval<T>> for EnumInterval<T> {
    #[inline]
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: PartialOrd> Contains<&HalfInterval<T>> for EnumInterval<T> {
    #[inline]
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<&Self> for EnumInterval<T> {
    #[inline]
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

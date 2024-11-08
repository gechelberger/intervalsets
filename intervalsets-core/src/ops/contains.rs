use super::Contains;
use crate::bound::Side;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};

impl<T: PartialOrd> Contains<T> for FiniteInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.ref_map(|left, right| {
            left.contains(Side::Left, rhs) && right.contains(Side::Right, rhs)
        })
        .unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<Self> for FiniteInterval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.ref_map(|left_out, right_out| {
            rhs.ref_map(|left_in, right_in| {
                left_out.contains(Side::Left, left_in.value())
                    && right_out.contains(Side::Right, right_in.value())
            })
            .unwrap_or(false)
        })
        .unwrap_or(false)

        /*
        I'm curious to bench mark the two of these and see if there is any difference

        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => match rhs {
                Self::Empty => false,
                Self::NonZero(a, b) => {
                    left.contains(Side::Left, &a.value)
                        && right.contains(Side::Right, &b.value)
                }
            },
        }*/
    }
}

impl<T: PartialOrd> Contains<HalfInterval<T>> for FiniteInterval<T> {
    fn contains(&self, _rhs: &HalfInterval<T>) -> bool {
        false
    }
}

impl<T: PartialOrd> Contains<EnumInterval<T>> for FiniteInterval<T> {
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<T> for HalfInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.bound.contains(self.side, rhs)
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.ref_map(|left, right| self.contains(left.value()) && self.contains(right.value()))
            .unwrap_or(false)
    }
}

impl<T: PartialOrd> Contains<Self> for HalfInterval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(rhs.bound.value())
    }
}

impl<T: PartialOrd> Contains<EnumInterval<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        match rhs {
            EnumInterval::Finite(rhs) => self.contains(rhs),
            EnumInterval::Half(rhs) => self.contains(rhs),
            EnumInterval::Unbounded => false,
        }
    }
}

impl<T: PartialOrd> Contains<T> for EnumInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for EnumInterval<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => *rhs != FiniteInterval::Empty,
        }
    }
}

impl<T: PartialOrd> Contains<HalfInterval<T>> for EnumInterval<T> {
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: PartialOrd> Contains<Self> for EnumInterval<T> {
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

impl<T: PartialOrd> Contains<T> for StackSet<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for StackSet<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: PartialOrd> Contains<HalfInterval<T>> for StackSet<T> {
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: PartialOrd> Contains<EnumInterval<T>> for StackSet<T> {
    fn contains(&self, rhs: &EnumInterval<T>) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: PartialOrd> Contains<StackSet<T>> for StackSet<T> {
    fn contains(&self, rhs: &StackSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

impl<T: PartialOrd> Contains<StackSet<T>> for FiniteInterval<T> {
    fn contains(&self, rhs: &StackSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

impl<T: PartialOrd> Contains<StackSet<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &StackSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

impl<T: PartialOrd> Contains<StackSet<T>> for EnumInterval<T> {
    fn contains(&self, rhs: &StackSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

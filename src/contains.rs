use crate::ival::Side;
use crate::{half::HalfInterval, FiniteInterval};
use crate::infinite::{Interval, IntervalSet};


/// A trait to determine whether one item fully contains another.
/// Contains is not associative.
pub trait Contains<Rhs> {
    
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T: PartialOrd> Contains<T> for FiniteInterval<T> {

    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => {
                left.contains(Side::Left, rhs) && right.contains(Side::Right, rhs)
            }
        }
    }
}

impl<T: PartialOrd> Contains<Self> for FiniteInterval<T> {

    /// Check if this interval fully contains the other
    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => {
                match rhs {
                    Self::Empty => false,
                    Self::NonZero(a, b) => {
                        left.contains(Side::Left, &a.value) &&
                        right.contains(Side::Right, &b.value)
                    }
                }
            }
        }
    }
}

impl<T> Contains<HalfInterval<T>> for FiniteInterval<T> {

    /// A FiniteInterval can never contain a HalfInterval
    fn contains(&self, _: &HalfInterval<T>) -> bool {
        false
    }
}

impl<T: PartialOrd> Contains<Interval<T>> for FiniteInterval<T> {

    fn contains(&self, rhs: &Interval<T>) -> bool {
        match rhs {
            Interval::Infinite => false,
            Interval::Half(interval) => self.contains(interval),
            Interval::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: PartialOrd> Contains<T> for HalfInterval<T> {

    fn contains(&self, rhs: &T) -> bool {
        self.ival.contains(self.side, rhs)
    }
}

impl<T: PartialOrd> Contains<Self> for HalfInterval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side 
            && self.ival.contains(self.side, &rhs.ival.value)
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match rhs {
            FiniteInterval::Empty => false,
            FiniteInterval::NonZero(left, right) => {
                self.ival.contains(self.side, &left.value) 
                    && self.ival.contains(self.side, &right.value)
            }
        }
    }
}

impl<T: PartialOrd> Contains<Interval<T>> for HalfInterval<T> {

    fn contains(&self, rhs: &Interval<T>) -> bool {
        match rhs {
            Interval::Infinite => false,
            Interval::Half(interval) => self.contains(interval),
            Interval::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: PartialOrd> Contains<T> for Interval<T> {

    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for Interval<T> {

    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Infinite => *rhs != FiniteInterval::Empty,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<HalfInterval<T>> for Interval<T> {

    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<Self> for Interval<T> {

    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Infinite => match rhs {
                Self::Infinite => false, // I think?
                Self::Half(interval) => self.contains(interval),
                Self::Finite(interval) => self.contains(interval), 
            },
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs)
        }
    }
}

////////////////////////////////////////


impl<T: PartialOrd> Contains<T> for IntervalSet<T> {

    fn contains(&self, rhs: &T) -> bool {
        self.intervals.iter()
            .any(|subset| subset.contains(rhs))
    }
}

// todo: other interval set conains
use crate::numeric::Domain;
use crate::ops::Contains;

use super::*;

impl<T: Domain> Contains<T> for Finite<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.map_or(false, |left, right| {
            left.contains(Side::Left, rhs) && right.contains(Side::Right, rhs)
        })
    }
}

impl<T: Domain> Contains<Self> for Finite<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.map_or(false, |left_out, right_out| {
            rhs.map_or(false, |left_in, right_in| {
                left_out.contains(Side::Left, left_in.value())
                    && right_out.contains(Side::Right, right_in.value())
            })
        })

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

impl<T: Domain> Contains<HalfBounded<T>> for Finite<T> {
    fn contains(&self, rhs: &HalfBounded<T>) -> bool {
        false
    }
}

impl<T: Domain> Contains<BoundCase<T>> for Finite<T> {
    fn contains(&self, rhs: &BoundCase<T>) -> bool {
        match rhs {
            BoundCase::Finite(rhs) => self.contains(rhs),
            BoundCase::Half(rhs) => self.contains(rhs),
            BoundCase::Unbounded => false,
        }
    }
}

impl<T: Domain> Contains<T> for HalfBounded<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.bound.contains(self.side, rhs)
    }
}

impl<T: Domain> Contains<Finite<T>> for HalfBounded<T> {
    fn contains(&self, rhs: &Finite<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(left.value()) && self.contains(right.value())
        })
    }
}

impl<T: Domain> Contains<Self> for HalfBounded<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(rhs.bound.value())
    }
}

impl<T: Domain> Contains<BoundCase<T>> for HalfBounded<T> {
    fn contains(&self, rhs: &BoundCase<T>) -> bool {
        match rhs {
            BoundCase::Finite(rhs) => self.contains(rhs),
            BoundCase::Half(rhs) => self.contains(rhs),
            BoundCase::Unbounded => false,
        }
    }
}

impl<T: Domain> Contains<T> for BoundCase<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: Domain> Contains<Finite<T>> for BoundCase<T> {
    fn contains(&self, rhs: &Finite<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => *rhs != Finite::Empty,
        }
    }
}

impl<T: Domain> Contains<HalfBounded<T>> for BoundCase<T> {
    fn contains(&self, rhs: &HalfBounded<T>) -> bool {
        match self {
            Self::Finite(lhs) => lhs.contains(rhs),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Unbounded => true,
        }
    }
}

impl<T: Domain> Contains<Self> for BoundCase<T> {
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

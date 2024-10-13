use crate::contains::Contains;
use crate::intersects::Intersects;
use crate::ival::{IVal, Side};
use crate::{FiniteInterval, HalfInterval, Interval};

/// Union for two intervals that are contiguous.
pub trait Merged<Rhs = Self> {
    type Output;

    fn merged(&self, rhs: &Rhs) -> Option<Self::Output>;
}

impl<T: Copy + PartialOrd> Merged<Self> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) {
            // TODO Adjacency

            return None;
        }

        self.map(|a_left, a_right| {
            rhs.map_bounds(|b_left, b_right| {
                FiniteInterval::NonZero(
                    IVal::min_left(a_left, b_left),
                    IVal::max_right(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Copy + PartialOrd> Merged<Self> for HalfInterval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(&rhs.ival.value) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else {
            // unfortunately we have to check from both sides to catch the
            // case where left and right values are the same but open & closed
            if self.contains(&rhs.ival.value) && rhs.contains(&self.ival.value) {
                Some(Interval::Infinite)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Copy + PartialOrd> Merged<FiniteInterval<T>> for HalfInterval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match rhs {
            FiniteInterval::Empty => Some(self.clone().into()),
            FiniteInterval::NonZero(left, right) => {
                let n_seen = [left, right]
                    .into_iter()
                    .filter(|ival| self.contains(&ival.value))
                    .count();

                match n_seen {
                    2 => Some(self.clone().into()),
                    1 => match self.side {
                        Side::Left => Some(HalfInterval::new(self.side, *left).into()),
                        Side::Right => Some(HalfInterval::new(self.side, *right).into()),
                    },
                    _ => None, // disjoint
                }
            }
        }
    }
}

impl<T: Copy + PartialOrd> Merged<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

//////////////////

impl<T: Copy + PartialOrd> Merged<FiniteInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Finite(lhs) => lhs.merged(rhs).map(|itv| itv.into()),
        }
    }
}

impl<T: Copy + PartialOrd> Merged<HalfInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Finite(lhs) => rhs.merged(lhs),
        }
    }
}

impl<T: Copy + PartialOrd> Merged<Self> for Interval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Infinite => Some(Self::Infinite),
            Self::Half(lhs) => rhs.merged(lhs),
            Self::Finite(lhs) => rhs.merged(lhs),
        }
    }
}

impl<T: Copy + PartialOrd> Merged<Interval<T>> for FiniteInterval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &Interval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Copy + PartialOrd> Merged<Interval<T>> for HalfInterval<T> {
    type Output = Interval<T>;

    fn merged(&self, rhs: &Interval<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_contiguous() {
        assert_eq!(
            Interval::open(0, 100).merged(&Interval::open(50, 150)),
            Some(Interval::open(0, 150))
        );

        assert_eq!(
            Interval::open(0, 100).merged(&Interval::open(100, 200)),
            None,
        );

        assert_eq!(
            Interval::closed(0, 100).merged(&Interval::closed(100, 200)),
            Some(Interval::closed(0, 200))
        );
    }
}

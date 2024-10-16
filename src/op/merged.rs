use crate::empty::MaybeEmpty;
use crate::ival::{IVal, Side};
use crate::numeric::Domain;
use crate::pred::contains::Contains;
use crate::pred::intersects::Intersects;
use crate::{EBounds, FiniteInterval, HalfBounded, Interval};

/// Union for two intervals that are contiguous.
pub trait Merged<Rhs = Self> {
    type Output;

    fn merged(&self, rhs: &Rhs) -> Option<Self::Output>;
}

impl<T: Domain> Merged<Self> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        if self.is_disjoint_from(rhs) {
            // TODO Adjacency?
            // For T in Real, (0, 1) U [1, 2] => (0, 2]
            if self.is_empty() {
                return Some(rhs.clone());
            } else if rhs.is_empty() {
                return Some(self.clone());
            } else {
                return None;
            }
        }

        self.map(|a_left, a_right| {
            rhs.map_bounds(|b_left, b_right| {
                FiniteInterval::FullyBounded(
                    IVal::min_left(a_left, b_left),
                    IVal::max_right(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Domain> Merged<Self> for HalfBounded<T> {
    type Output = EBounds<T>;

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
                Some(EBounds::Unbounded)
            } else {
                None // disjoint
            }
        }
    }
}

impl<T: Domain> Merged<FiniteInterval<T>> for HalfBounded<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match rhs {
            FiniteInterval::Empty => Some(self.clone().into()),
            FiniteInterval::FullyBounded(left, right) => {
                let n_seen = [left, right]
                    .into_iter()
                    .filter(|ival| self.contains(&ival.value))
                    .count();

                match n_seen {
                    2 => Some(self.clone().into()),
                    1 => match self.side {
                        Side::Left => Some(HalfBounded::new(self.side, left.clone()).into()),
                        Side::Right => Some(HalfBounded::new(self.side, right.clone()).into()),
                    },
                    _ => None, // disjoint
                }
            }
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for FiniteInterval<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &HalfBounded<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged<FiniteInterval<T>> for EBounds<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Self::Unbounded => Some(Self::Unbounded),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Finite(lhs) => lhs.merged(rhs).map(|itv| itv.into()),
        }
    }
}

impl<T: Domain> Merged<HalfBounded<T>> for EBounds<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &HalfBounded<T>) -> Option<Self::Output> {
        match self {
            Self::Unbounded => Some(Self::Unbounded),
            Self::Half(lhs) => lhs.merged(rhs),
            Self::Finite(lhs) => rhs.merged(lhs),
        }
    }
}

impl<T: Domain> Merged<Self> for EBounds<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        match self {
            Self::Unbounded => Some(Self::Unbounded),
            Self::Half(lhs) => rhs.merged(lhs),
            Self::Finite(lhs) => rhs.merged(lhs),
        }
    }
}

impl<T: Domain> Merged<EBounds<T>> for FiniteInterval<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &EBounds<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged<EBounds<T>> for HalfBounded<T> {
    type Output = EBounds<T>;

    fn merged(&self, rhs: &EBounds<T>) -> Option<Self::Output> {
        rhs.merged(self)
    }
}

impl<T: Domain> Merged for Interval<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        self.0.merged(&rhs.0).map(|v| v.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merged_empty() {
        assert_eq!(
            FiniteInterval::<i8>::Empty.merged(&FiniteInterval::Empty),
            Some(FiniteInterval::Empty)
        );

        assert_eq!(
            FiniteInterval::<i8>::Empty.merged(&FiniteInterval::closed(0, 10)),
            Some(FiniteInterval::closed(0, 10))
        );

        assert_eq!(
            FiniteInterval::closed(0, 10).merged(&FiniteInterval::Empty),
            Some(FiniteInterval::closed(0, 10))
        );
    }

    #[test]
    fn test_finite_merged() {
        assert_eq!(
            EBounds::open(0, 100).merged(&EBounds::open(50, 150)),
            Some(EBounds::open(0, 150))
        );

        assert_eq!(
            EBounds::open(0, 100).merged(&EBounds::open(100, 200)),
            None,
        );

        assert_eq!(
            EBounds::closed(0, 100).merged(&EBounds::closed(100, 200)),
            Some(EBounds::closed(0, 200))
        );
    }
}

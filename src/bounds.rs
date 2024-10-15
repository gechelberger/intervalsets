use crate::ival::{Bound, IVal, Side};
use crate::numeric::Domain;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

/// The `Bounds` trait provides safe accessors for the
/// boundary conditions of intervals/sets.
///
/// Both Empty and Infinite bounds are None.
/// In order to distinguish between them, use
/// the MaybeEmpty trait to check for emptiness.
pub trait Bounds<T> {
    /// Get the left or right bound if it exists.
    ///
    /// Both Empty and Infinite bounds are None.
    /// In order to distinguish between them, use
    /// the MaybeEmpty trait to check for emptiness.
    fn bound(&self, side: Side) -> Option<IVal<T>>;

    fn left(&self) -> Option<IVal<T>> {
        self.bound(Side::Left)
    }

    fn right(&self) -> Option<IVal<T>> {
        self.bound(Side::Right)
    }

    fn lval(&self) -> Option<T> {
        self.left().map(|v| v.value)
    }

    fn rval(&self) -> Option<T> {
        self.right().map(|v| v.value)
    }

    fn lbound(&self) -> Option<Bound> {
        self.left().map(|v| v.bound)
    }

    fn rbound(&self) -> Option<Bound> {
        self.right().map(|v| v.bound)
    }
}

impl<T: Clone> Bounds<T> for FiniteInterval<T> {
    fn bound(&self, side: Side) -> Option<IVal<T>> {
        match self {
            Self::Empty => None,
            Self::NonZero(left, right) => match side {
                Side::Left => Some(left.clone()),
                Side::Right => Some(right.clone()),
            },
        }
    }
}

impl<T: Clone> Bounds<T> for HalfInterval<T> {
    fn bound(&self, side: Side) -> Option<IVal<T>> {
        if self.side == side {
            Some(self.ival.clone())
        } else {
            None
        }
    }
}

impl<T: Clone> Bounds<T> for Interval<T> {
    fn bound(&self, side: Side) -> Option<IVal<T>> {
        match self {
            Self::Infinite => None,
            Self::Half(interval) => interval.bound(side),
            Self::Finite(interval) => interval.bound(side),
        }
    }
}

impl<T: Domain> Bounds<T> for IntervalSet<T> {
    fn bound(&self, side: Side) -> Option<IVal<T>> {
        let mut result = None;

        for itv in self.intervals.iter() {
            let candidate = itv.bound(side);

            // any None implies an infinite bound
            candidate.as_ref()?;
            //if candidate.is_none() {
            //    return None;
            //}

            result = match result {
                None => candidate,
                Some(result) => Some(match side {
                    Side::Left => IVal::min_left(&result, &candidate.unwrap()),
                    Side::Right => IVal::max_right(&result, &candidate.unwrap()),
                }),
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_interval_bounds() {
        let itv = FiniteInterval::open_closed(0, 5);
        assert_eq!(itv.lval(), Some(1));
        assert_eq!(itv.rval(), Some(5));
        assert_eq!(itv.lbound(), Some(Bound::Closed));
        assert_eq!(itv.rbound(), Some(Bound::Closed));
    }

    #[test]
    fn test_half_interval_bounds() {
        let itv = HalfInterval::open_unbound(5);
        assert_eq!(itv.lval(), Some(6));
        assert_eq!(itv.rval(), None);
        assert_eq!(itv.lbound(), Some(Bound::Closed));
        assert_eq!(itv.rbound(), None);
    }

    #[test]
    #[ignore]
    fn test_interval_bounds() {}

    #[test]
    #[ignore]
    fn test_interval_set_bounds() {}
}

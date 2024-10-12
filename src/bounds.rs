use crate::{half::HalfInterval, infinite::IntervalSet, ival::{Bound, IVal, Side}, FiniteInterval, Interval};

/// The `Bounds` trait provides safe accessors for the boundary conditions
/// of any interval that implements it.
trait Bounds<T> {

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
                Side::Right => Some(right.clone())
            }
        }
    }

    fn left(&self) -> Option<IVal<T>> {
        match self {
            Self::Empty => None,
            Self::NonZero(left, _) => Some(left.clone())
        }
    }

    fn right(&self) -> Option<IVal<T>> {
        match self {
            Self::Empty => None,
            Self::NonZero(_, right) => Some(right.clone())
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

impl<T: Clone + Eq + Ord> Bounds<T> for IntervalSet<T> {

    fn bound(&self, side: Side) -> Option<IVal<T>> {
        let mut result = None;

        for itv in self.intervals.iter() {
            let candidate = itv.bound(side);
            if candidate == None {
                // any None implies an infinite bound
                return None;
            }
            
            result = match result {
                None => candidate,
                Some(result) => Some(match side {
                    Side::Left => IVal::min(&result, &candidate.unwrap()),
                    Side::Right => IVal::max(&result, &candidate.unwrap())
                }
                )
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
        let itv = FiniteInterval::openclosed(0, 5);
        assert_eq!(itv.lval(), Some(0));
        assert_eq!(itv.rval(), Some(5));
        assert_eq!(itv.lbound(), Some(Bound::Open));
        assert_eq!(itv.rbound(), Some(Bound::Closed));
    }

    #[test]
    fn test_half_interval_bounds() {
        let itv = HalfInterval::open_unbound(5);
        assert_eq!(itv.lval(), Some(5));
        assert_eq!(itv.rval(), None);
        assert_eq!(itv.lbound(), Some(Bound::Open));
        assert_eq!(itv.rbound(), None);
    }

}
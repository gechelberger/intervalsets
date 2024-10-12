use crate::{half::HalfInterval, infinite::IntervalSet, ival::{Bound, IVal, Side}, FiniteInterval, Interval};

/// The `Bounds` trait provides safe accessors for the boundary conditions
/// of any interval that implements it.
trait Bounds<T> {
    fn left(&self) -> Option<IVal<T>>;

    fn right(&self) -> Option<IVal<T>>;

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

    fn left(&self) -> Option<IVal<T>> {
        match self.side {
            Side::Left => Some(self.ival.clone()),
            Side::Right => None,
        }
    }

    fn right(&self) -> Option<IVal<T>> {
        match self.side {
            Side::Left => None,
            Side::Right => Some(self.ival.clone()),
        }
    }
}

impl<T: Clone> Bounds<T> for Interval<T> {

    fn left(&self) -> Option<IVal<T>> {
        match self {
            Self::Infinite => None,
            Self::Half(interval) => interval.left(),
            Self::Finite(interval) => interval.left(),
        }
    }

    fn right(&self) -> Option<IVal<T>> {
        match self {
            Self::Infinite => None,
            Self::Half(interval) => interval.right(),
            Self::Finite(interval) => interval.right(),
        }
    }
}

impl<T: Clone + Eq + Ord> Bounds<T> for IntervalSet<T> {

    fn left(&self) -> Option<IVal<T>> {
        let mut result = None;

        for itv in self.intervals.iter() {
            let left_candidate = itv.left();
            if left_candidate == None {
                // any left of None implies an infinite left bound
                return None;
            }
            
            result = match result {
                None => left_candidate,
                Some(result) => Some(
                    IVal::min(&result, &left_candidate.unwrap())
                )
            }
        }
        result
    }

    fn right(&self) -> Option<IVal<T>> {
        let mut result = None;

        for itv in self.intervals.iter() {
            let right_candidate = itv.right();
            if right_candidate == None {
                // any None implies an infinite bound
                return None;
            }
            
            result = match result {
                None => right_candidate,
                Some(result) => Some(
                    IVal::min(&result, &right_candidate.unwrap())
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

}
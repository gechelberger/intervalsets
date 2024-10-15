use crate::ival::{Bound, IVal, Side};
use crate::numeric::Domain;

/// A fully bounded interval in N, Z, or R.
///
/// (a, a) = (a, a] = [a, a) = Empty { x not in T }
/// [a, a] = NonZero { x in T |    x = a    }
/// (a, b) = NonZero { x in T | a <  x <  b }
/// (a, b] = NonZero { x in T | a <  x <= b }
/// [a, b) = NonZero { x in T | a <= x <  b }
/// [a, b] = NonZero { x in T | a <= x <= b }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FiniteInterval<T> {
    Empty,
    NonZero(IVal<T>, IVal<T>),
}

impl<T: Domain> FiniteInterval<T> {
    pub fn new(left: IVal<T>, right: IVal<T>) -> Self {
        if left.value > right.value {
            Self::Empty
        } else if left.value == right.value {
            if left.bound == Bound::Open || right.bound == Bound::Open {
                Self::Empty
            } else {
                // singleton set
                Self::new_unchecked(left, right)
            }
        } else {
            Self::new_unchecked(left.normalized(Side::Left), right.normalized(Side::Right))
        }
    }

    pub fn new_unchecked(left: IVal<T>, right: IVal<T>) -> Self {
        Self::NonZero(left, right)
    }

    pub fn open(left: T, right: T) -> Self {
        Self::new(IVal::new(Bound::Open, left), IVal::new(Bound::Open, right))
    }

    pub fn closed(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        )
    }

    pub fn openclosed(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        )
    }

    pub fn closedopen(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        )
    }
}

impl<T> FiniteInterval<T> {
    pub fn lval_unchecked(&self) -> &T {
        match self {
            Self::Empty => panic!("Empty interval has no left bound"),
            Self::NonZero(left, _) => &left.value,
        }
    }

    pub fn rval_unchecked(&self) -> &T {
        match self {
            Self::Empty => panic!("Empty interval has no right bound"),
            Self::NonZero(_, right) => &right.value,
        }
    }

    pub fn map_bounds(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> Self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonZero(left, right) => func(left, right),
        }
    }

    #[allow(dead_code)]
    pub fn map<U>(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> U) -> Option<U> {
        match self {
            Self::Empty => None,
            Self::NonZero(left, right) => Some(func(left, right)),
        }
    }

    pub fn map_or<U>(&self, default: U, func: impl Fn(&IVal<T>, &IVal<T>) -> U) -> U {
        match self {
            Self::Empty => default,
            Self::NonZero(left, right) => func(left, right),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_interval_new() {
        let interval: FiniteInterval<usize> = FiniteInterval::open(0, 20);

        let interval = FiniteInterval::open(0, 0);
        assert_eq!(interval, FiniteInterval::Empty);
    }
}

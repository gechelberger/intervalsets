use crate::{half::HalfInterval, ival::{Bound, IVal, Side}, FiniteInterval, Interval};

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

impl<T> Bounds<T> for Interval<T> {

    fn left(&self) -> Option<IVal<T>> {
        todo!()
    }

    fn right(&self) -> Option<IVal<T>> {
        todo!()
    }
}
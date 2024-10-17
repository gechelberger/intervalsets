use crate::{Bound, Domain, Interval, IntervalSet, Side};

pub trait Bounding<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>>;

    fn left(&self) -> Option<&Bound<T>> {
        self.bound(Side::Left)
    }

    fn right(&self) -> Option<&Bound<T>> {
        self.bound(Side::Right)
    }

    fn lval(&self) -> Option<&T> {
        self.left().map(|b| b.value())
    }

    fn rval(&self) -> Option<&T> {
        self.right().map(|b| b.value())
    }
}

impl<T: Domain> Bounding<T> for Interval<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>> {
        self.0.bound(side)
    }
}

impl<T: Domain> Bounding<T> for IntervalSet<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>> {
        match side {
            Side::Left => self.intervals().first().and_then(|s| s.left()),
            Side::Right => self.intervals().last().and_then(|s| s.right()),
        }
    }
}

use crate::{Bound, Domain, Interval, Side};

pub trait Bounding<T: Domain> {
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

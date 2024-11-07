use crate::numeric::Domain;
use crate::{Bound, Interval, IntervalSet, Side};

/// The boundaries of a Set on the number line.
///
/// The left and right boundary represented here
/// are are extrema of a Set, and would properly
/// define the convex hull of a Set.
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

impl<'a, T: Domain> Bounding<T> for &'a Interval<T> {
    fn bound(&self, side: Side) -> Option<&'a Bound<T>> {
        (*self).bound(side)
    }
}

impl<T: Domain> Bounding<T> for IntervalSet<T> {
    fn bound(&self, side: Side) -> Option<&Bound<T>> {
        let maybe_interval = match side {
            Side::Left => self.slice().first(),
            Side::Right => self.slice().last(),
        };

        maybe_interval.and_then(|s| s.bound(side))
    }
}

impl<'a, T: Domain> Bounding<T> for &'a IntervalSet<T> {
    fn bound(&self, side: Side) -> Option<&'a Bound<T>> {
        (*self).bound(side)
    }
}
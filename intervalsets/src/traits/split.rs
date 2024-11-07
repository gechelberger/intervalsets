use crate::bound::Side;
use crate::numeric::Domain;
use crate::ops::Contains;
use crate::{Bounding, Interval, IntervalSet, MaybeEmpty};

/// Split a Set into two disjoint subsets, fully covering the original.
///
/// `at` provides the new bound where the set should be split.
///
/// # Example
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x = Interval::closed(0, 10);
/// let (left, right) = x.split(5, Side::Left);
/// assert_eq!(left, Interval::closed(0, 5));
/// assert_eq!(right, Interval::closed(6, 10));
/// ```
pub trait Split<T> {
    type Output: Sized;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output);
}

pub trait RefSplit<T>: Split<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output);
}

impl<T: Domain> Split<T> for Interval<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        let (left, right) = self.0.split(at, closed);
        (left.into(), right.into())
    }
}

impl<T: Domain> RefSplit<T> for Interval<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.clone().split(at, closed)
    }
}

impl<T: Domain> Split<T> for IntervalSet<T> {
    type Output = Self;

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        if self.is_empty() {
            return (Self::empty(), Self::empty());
        }

        let mut left = Vec::<Interval<T>>::new();
        let mut right = Vec::<Interval<T>>::new();

        // faster than a binary search for small (typical) N.
        for subset in self.into_iter() {
            if subset.contains(&at) {
                let (ileft, iright) = subset.split(at.clone(), closed);
                left.push(ileft);
                right.push(iright);
            } else if let Some(rbound) = subset.right() {
                if !rbound.contains(Side::Right, &at) {
                    left.push(subset);
                } else {
                    right.push(subset);
                }
            } else {
                right.push(subset);
            }
        }

        (Self::new_unchecked(left), Self::new_unchecked(right))
    }
}

impl<T: Domain> RefSplit<T> for IntervalSet<T> {
    fn ref_split(&self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.clone().split(at, closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::Union;
    use crate::Factory;

    #[test]
    fn test_split_interval_empty() {
        let interval = Interval::<i32>::empty();
        let (left, right) = Interval::<i32>::empty().split(0, Side::Left);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, Interval::empty());
    }

    #[test]
    fn test_split_interval() {
        let interval = Interval::closed(0, 100);
        let (left, right) = interval.split(50, Side::Left);
        assert_eq!(left, Interval::closed(0, 50));
        assert_eq!(right, Interval::closed(51, 100));

        let interval = Interval::<i32>::unbounded();
        let (left, right) = interval.split(50, Side::Left);
        assert_eq!(left, Interval::unbound_closed(50));
        assert_eq!(right, Interval::closed_unbound(51));

        let (left, right) = Interval::<f32>::unbounded().split(0.0, Side::Right);
        assert_eq!(left, Interval::unbound_open(0.0));
        assert_eq!(right, Interval::closed_unbound(0.0));
    }

    #[test]
    fn test_split_interval_on_bound() {
        let x = Interval::closed(0, 10);
        let (left, right) = x.clone().split(0, Side::Left);
        assert_eq!(left, (0, 0).into());
        assert_eq!(right, (1, 10).into());

        let (left, right) = x.clone().split(0, Side::Right);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, x);

        let x = Interval::closed(0.0, 10.0);
        let (left, right) = x.clone().split(0.0, Side::Left);
        assert_eq!(left, (0.0, 0.0).into());
        assert_eq!(right, Interval::open_closed(0.0, 10.0));

        let (left, right) = x.clone().split(0.0, Side::Right);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, x.clone());

        let (left, right) = x.clone().split(10.0, Side::Left);
        assert_eq!(left, x.clone());
        assert_eq!(right, Interval::empty());

        let (left, right) = x.clone().split(10.0, Side::Right);
        assert_eq!(left, Interval::closed_open(0.0, 10.0));
        assert_eq!(right, (10.0, 10.0).into());
    }

    #[test]
    fn test_split_interval_not_contained() {
        let interval = Interval::closed(50, 100);
        let (left, right) = interval.ref_split(0, Side::Left);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, interval);

        let (left, right) = interval.ref_split(200, Side::Left);
        assert_eq!(left, interval);
        assert_eq!(right, Interval::empty());
    }

    #[test]
    fn test_split_half() {
        let interval = Interval::unbound_closed(100);
        let (left, right) = interval.split(0, Side::Left);
        assert_eq!(left, Interval::unbound_closed(0));
        assert_eq!(right, Interval::open_closed(0, 100));
    }

    #[test]
    fn test_split_set_empty() {
        let set = IntervalSet::<i32>::empty();
        let (left, right) = set.split(0, Side::Left);
        assert_eq!(left, IntervalSet::empty());
        assert_eq!(right, IntervalSet::empty());
    }

    #[test]
    fn test_split_set_not_contained() {
        let set = Interval::closed(-100, -50).union(Interval::closed(50, 100));
        let (left, right) = set.split(0, Side::Left);
        assert_eq!(left.expect_interval(), Interval::closed(-100, -50));
        assert_eq!(right.expect_interval(), Interval::closed(50, 100));

        let neg = Interval::closed(-200, -150).union(Interval::closed(-100, -50));
        let pos = Interval::closed(50, 100).union(Interval::closed(150, 200));
        let set = neg.clone().union(pos.clone());
        let (left, right) = set.ref_split(0, Side::Left);
        assert_eq!(left, neg);
        assert_eq!(right, pos);

        let (left, right) = set.ref_split(-125, Side::Left);
        assert_eq!(left.expect_interval(), Interval::closed(-200, -150));
        assert_eq!(right, pos.union(Interval::closed(-100, -50)));
    }
}

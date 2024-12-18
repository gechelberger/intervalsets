pub use intervalsets_core::ops::Split;

use crate::bound::Side;
use crate::numeric::{Element, Zero};
use crate::ops::Contains;
use crate::{Interval, IntervalSet, MaybeEmpty, SetBounds};

impl<T: Element + Clone + Zero> Split<T> for Interval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn strict_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        self.0
            .strict_split(at, closed)
            .map(|(l, r)| (l.into(), r.into()))
    }
}

impl<T: Element + Clone + Zero> Split<T> for IntervalSet<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn strict_split(
        self,
        at: T,
        closed: Side,
    ) -> Result<(Self::Output, Self::Output), Self::Error> {
        if self.is_empty() {
            return Ok((Self::Output::empty(), Self::Output::empty()));
        }

        let mut left = Vec::<Interval<T>>::new();
        let mut right = Vec::<Interval<T>>::new();

        // iter is faster than a binary search for small (typical) N.
        for subset in self.into_iter() {
            if subset.contains(&at) {
                let split = subset.strict_split(at.clone(), closed)?;
                if !split.0.is_empty() {
                    left.push(split.0);
                }
                if !split.1.is_empty() {
                    right.push(split.1);
                }
            } else if let Some(rbound) = subset.right() {
                if !rbound.strict_contains(Side::Right, &at)? {
                    left.push(subset);
                } else {
                    right.push(subset);
                }
            } else {
                right.push(subset);
            }
        }

        // SAFETY:
        // 1. no input subsets may be empty. split subsets are checked for empty.
        // 2. original subset order is maintained
        // 3. if intervals were unconnected in original set then that is preserved
        //    in split child sets.
        unsafe {
            let left_set = Self::Output::new_unchecked(left);
            let right_set = Self::Output::new_unchecked(right);
            Ok((left_set, right_set))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;
    use crate::ops::Union;

    #[test]
    fn test_split_interval_empty() {
        let interval = Interval::<i32>::empty();
        let (left, right) = interval.split(0, Side::Left);
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
        assert_eq!(left, [0, 0].into());
        assert_eq!(right, [1, 10].into());

        let (left, right) = x.clone().split(0, Side::Right);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, x);

        let x = Interval::closed(0.0, 10.0);
        let (left, right) = x.clone().split(0.0, Side::Left);
        assert_eq!(left, [0.0, 0.0].into());
        assert_eq!(right, Interval::open_closed(0.0, 10.0));

        let (left, right) = x.clone().split(0.0, Side::Right);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, x.clone());

        let (left, right) = x.clone().split(10.0, Side::Left);
        assert_eq!(left, x.clone());
        assert_eq!(right, Interval::empty());

        let (left, right) = x.clone().split(10.0, Side::Right);
        assert_eq!(left, Interval::closed_open(0.0, 10.0));
        assert_eq!(right, [10.0, 10.0].into());
    }

    #[test]
    fn test_split_interval_not_contained() {
        let interval = Interval::closed(50, 100);
        let (left, right) = interval.split(0, Side::Left);
        assert_eq!(left, Interval::empty());
        assert_eq!(right, interval);

        let (left, right) = interval.split(200, Side::Left);
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
        let (left, right) = set.clone().split(0, Side::Left);
        assert_eq!(left, neg);
        assert_eq!(right, pos);

        let (left, right) = set.split(-125, Side::Left);
        assert_eq!(left.expect_interval(), Interval::closed(-200, -150));
        assert_eq!(right, pos.union(Interval::closed(-100, -50)));
    }

    #[test]
    fn test_split_set_on_interval_bound() {
        let iset = IntervalSet::new(vec![
            Interval::closed(10, 20),
            Interval::closed(30, 40),
            Interval::closed(50, 60),
        ]);

        let (left, right) = iset.clone().strict_split(30, Side::Right).unwrap();
        assert_eq!(left, IntervalSet::from_iter([[10, 20]]));
        assert_eq!(right, IntervalSet::from_iter([[30, 40], [50, 60]]));

        let (left, right) = iset.strict_split(40, Side::Left).unwrap();
        assert_eq!(left, IntervalSet::from_iter([[10, 20], [30, 40]]));
        assert_eq!(right, IntervalSet::from_iter([[50, 60]]));
    }
}

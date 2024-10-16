use crate::bounds::Bounds;
use crate::FiniteInterval;
use crate::empty::MaybeEmpty;
use crate::ival::{IVal, Side};
use crate::numeric::Domain;
use crate::{HalfInterval, Interval, IntervalSet};

/// Defines the creation of the minimal contiguous Interval/Set
/// which covers all of the provided items.
///
/// # Example
/// ```
/// use intervalsets::Interval;
/// use intervalsets::op::hull::ConvexHull;
///
/// let interval = Interval::convex_hull([5, 3, -120, 44, 100, -100]);
/// assert_eq!(interval, Interval::closed(-120, 100));
/// ```
pub trait ConvexHull<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self;
}

impl<T: Domain> ConvexHull<T> for FiniteInterval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut iter = iter.into_iter();

        // check our empty case first
        let mut bounds = match iter.next() {
            None => return FiniteInterval::Empty,
            Some(item) => (IVal::closed(item.clone()), IVal::closed(item)),
        };

        for item in iter {
            let item_ival = IVal::closed(item);
            let left = IVal::min_left(&bounds.0, &item_ival);
            let right = IVal::max_right(&bounds.1, &item_ival);
            bounds = (left, right);
        }

        FiniteInterval::new_unchecked(bounds.0, bounds.1)
    }
}

impl<T: Domain> ConvexHull<T> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        FiniteInterval::convex_hull(iter).into()
    }
}

// private impl based on Bounds + MaybeEmpty
fn convex_hull_bounds_impl<T, B, U>(iter: U) -> Interval<T>
where
    T: Domain,
    B: Bounds<T> + MaybeEmpty,
    U: IntoIterator<Item = B>,
{
    let mut iter = iter.into_iter();

    // this is kind of wonky syntax:
    // take from iterator until (skipping over empty intervals):
    // 1) it is exhausted -> return Empty
    // 2) we find a non-empty interval and extract it's left and right bounds (or None for +/- inf)
    let mut bounds = loop {
        match iter.next() {
            None => return FiniteInterval::Empty.into(),
            Some(set) => {
                if set.is_empty() {
                    continue;
                } else {
                    break (set.left(), set.right());
                }
            }
        }
    };

    for item in iter {
        if item.is_empty() {
            continue;
        }

        // None should take the greatest precedence since it represents infinity.
        let left = bounds
            .0
            .and_then(|prev| item.left().map(|value| IVal::min_left(&prev, &value)));

        let right = bounds
            .1
            .and_then(|prev| item.right().map(|value| IVal::max_right(&prev, &value)));

        bounds = (left, right);
    }

    match bounds {
        (None, None) => Interval::Infinite,
        (Some(ival), None) => Interval::Half(HalfInterval::new(Side::Left, ival)),
        (None, Some(ival)) => Interval::Half(HalfInterval::new(Side::Right, ival)),
        (Some(left), Some(right)) => Interval::Finite(FiniteInterval::new_unchecked(left, right)),
    }
}

impl<T: Domain> ConvexHull<Interval<T>> for Interval<T> {
    /// Create a new interval that covers a set of intervals
    ///
    /// # Example
    /// ```
    /// use intervalsets::Interval;
    /// use intervalsets::op::hull::ConvexHull;
    ///
    /// let iv = Interval::convex_hull(vec![
    ///     Interval::closed(100.0, 200.0),
    ///     Interval::open(0.0, 10.0),
    ///     Interval::closed_unbound(500.0),
    /// ]);
    /// assert_eq!(iv, Interval::open_unbound(0.0));
    /// ```
    fn convex_hull<U: IntoIterator<Item = Interval<T>>>(iter: U) -> Self {
        convex_hull_bounds_impl(iter)
    }
}

impl<T: Domain> ConvexHull<IntervalSet<T>> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = IntervalSet<T>>>(iter: U) -> Self {
        convex_hull_bounds_impl(iter)
    }
}

#[cfg(test)]
mod tests {
    use crate::op::union::Union;

    use super::*;

    #[test]
    fn test_finite_hull_of_points() {
        let iv = FiniteInterval::convex_hull(vec![5, 3, -1, 30, 2, -22, 100, -100]);
        assert_eq!(iv, FiniteInterval::closed(-100, 100))
    }

    #[test]
    fn test_hull_of_intervals_empty() {
        let items: Vec<u32> = vec![];
        assert_eq!(Interval::convex_hull(items), Interval::empty())
    }

    #[test]
    fn test_hull_of_intervals() {
        let iv = Interval::convex_hull(vec![
            Interval::empty(),
            Interval::empty(),
            Interval::closed(0, 10),
            Interval::empty(),
            Interval::empty(),
        ]);
        assert_eq!(iv, Interval::closed(0, 10));
    }

    #[test]
    fn test_hull_of_intervals_unbound() {
        let iv = Interval::convex_hull(vec![
            Interval::empty(),
            Interval::closed(100.0, 200.0),
            Interval::empty(),
            Interval::open(0.0, 10.0),
            Interval::empty(),
            Interval::closed_unbound(500.0),
            Interval::empty(),
        ]);
        assert_eq!(iv, Interval::open_unbound(0.0));
    }

    #[test]
    fn test_hull_of_sets() {
        let sets: Vec<IntervalSet<f64>> = vec![
            IntervalSet::empty(),
            Interval::closed(0.0, 10.0)
                .union(&Interval::open(100.0, 110.0))
                .union(&Interval::open(200.0, 210.0)),
            IntervalSet::empty(),
            Interval::closed(-110.0, -100.0).union(&Interval::closed(-1000.0, -900.0)),
        ];
        assert_eq!(
            Interval::convex_hull(sets),
            Interval::closed_open(-1000.0, 210.0)
        );
    }
}

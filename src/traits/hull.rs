use crate::numeric::Domain;
use crate::{Bound, Bounding, Interval, IntervalSet, MaybeEmpty, Side};

/// Defines the minimal contiguous Interval
/// which fully contains every provided item.
///
/// # Example
/// ```
/// use intervalsets::{ConvexHull, Interval, IntervalSet};
/// use intervalsets::ops::Union;
///
/// // from points on the number line
/// let hull = Interval::convex_hull([5, 3, -120, 44, 100, -100]);
/// assert_eq!(hull, Interval::closed(-120, 100));
///
///
/// // from intervals
/// let intervals = vec![
///     Interval::open(30.0, 50.0),
///     Interval::closed(20.0, 40.0),
///     Interval::closed(1000.0, 2000.0),
///     Interval::unbound_open(0.0),
/// ];
/// let hull = Interval::convex_hull(intervals);
/// assert_eq!(hull, Interval::unbound_closed(2000.0));
///
///
/// // from sets
/// let sets: Vec<IntervalSet<i32>> = vec![
///     Interval::closed(0, 10).union(&Interval::closed(1000, 1010)),
///     Interval::closed(-1000, 10).into(),
///     Interval::closed(-500, 500).union(&Interval::closed_unbound(800))
/// ];
/// let hull: Interval<i32> = Interval::convex_hull(sets);
/// assert_eq!(hull, Interval::closed_unbound(-1000))
/// ```
pub trait ConvexHull<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self;
}

impl<T: Domain> ConvexHull<T> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut iter = iter.into_iter();

        // check our empty case first
        let mut bounds = match iter.next() {
            None => return Interval::empty(),
            Some(item) => (Bound::closed(item.clone()), Bound::closed(item)),
        };

        for item in iter {
            let item_ival = Bound::closed(item);
            let left = Bound::min_left(&bounds.0, &item_ival);
            let right = Bound::max_right(&bounds.1, &item_ival);
            bounds = (left, right);
        }

        Interval::new_finite(bounds.0, bounds.1)
    }
}

// private impl based on Bounds + MaybeEmpty
fn convex_hull_bounds_impl<T, B, U>(iter: U) -> Interval<T>
where
    T: Domain,
    B: Bounding<T> + MaybeEmpty,
    U: IntoIterator<Item = B>,
{
    let mut iter = iter.into_iter();

    // this is kind of wonky syntax:
    // take from iterator until (skipping over empty intervals):
    // 1) it is exhausted -> return Empty
    // 2) we find a non-empty interval and extract it's left and right bounds (or None for +/- inf)
    let mut bounds = loop {
        match iter.next() {
            None => return Interval::empty(),
            Some(set) => {
                if set.is_empty() {
                    continue;
                } else {
                    break (set.left().cloned(), set.right().cloned());
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
            .and_then(|prev| item.left().map(|value| Bound::min_left(&prev, value)));

        let right = bounds
            .1
            .and_then(|prev| item.right().map(|value| Bound::max_right(&prev, value)));

        bounds = (left, right);
    }

    match bounds {
        (Some(left), Some(right)) => Interval::new_finite(left, right),
        (Some(bound), None) => Interval::new_half_bounded(Side::Left, bound),
        (None, Some(bound)) => Interval::new_half_bounded(Side::Right, bound),
        (None, None) => Interval::unbounded(),
    }
}

impl<T: Domain> ConvexHull<Interval<T>> for Interval<T> {
    /// Create a new interval that covers a set of intervals
    ///
    /// # Example
    /// ```
    /// use intervalsets::Interval;
    /// use intervalsets::ConvexHull;
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
    use crate::ops::Union;

    use super::*;

    #[test]
    fn test_finite_hull_of_points() {
        let iv = Interval::convex_hull(vec![5, 3, -1, 30, 2, -22, 100, -100]);
        assert_eq!(iv, Interval::closed(-100, 100))
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

        let sets: Vec<IntervalSet<i32>> = vec![
            Interval::closed(0, 10).union(&Interval::closed(1000, 1010)),
            Interval::closed(-1000, 10).into(),
            Interval::closed(-500, 500).union(&Interval::closed_unbound(800)),
        ];
        let hull: Interval<i32> = Interval::<i32>::convex_hull(sets);
        assert_eq!(hull, Interval::closed_unbound(-1000))
    }
}

pub use intervalsets_core::ops::ConvexHull;
use intervalsets_core::ops::{convex_hull_into_ord_bound_impl, convex_hull_ord_bounded_impl};
use intervalsets_core::EnumInterval;

use crate::numeric::Domain;
use crate::{Interval, IntervalSet};

impl<T: Domain + Clone> ConvexHull<T> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        EnumInterval::convex_hull(iter).into()
    }
}

impl<'a, T: Domain + Clone> ConvexHull<&'a T> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = &'a T>>(iter: U) -> Self {
        EnumInterval::convex_hull(iter).into()
    }
}

impl<T: Domain + Clone> ConvexHull<Interval<T>> for Interval<T> {
    /// Create a new interval that covers a set of intervals
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::ConvexHull;
    ///
    /// let iv = Interval::convex_hull(vec![
    ///     Interval::closed(100.0, 200.0),
    ///     Interval::open(0.0, 10.0),
    ///     Interval::closed_unbound(500.0),
    /// ]);
    /// assert_eq!(iv, Interval::open_unbound(0.0));
    /// ```
    fn convex_hull<U: IntoIterator<Item = Interval<T>>>(iter: U) -> Self {
        convex_hull_into_ord_bound_impl(iter).unwrap().into()
    }
}

impl<'a, T: Domain + Clone> ConvexHull<&'a Interval<T>> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = &'a Interval<T>>>(iter: U) -> Self {
        convex_hull_ord_bounded_impl(iter).unwrap().into()
    }
}

impl<T: Domain + Clone> ConvexHull<IntervalSet<T>> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = IntervalSet<T>>>(iter: U) -> Self {
        convex_hull_into_ord_bound_impl(iter).unwrap().into()
    }
}

impl<'a, T: Domain + Clone> ConvexHull<&'a IntervalSet<T>> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = &'a IntervalSet<T>>>(iter: U) -> Self {
        convex_hull_ord_bounded_impl(iter).unwrap().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::Union;
    use crate::Factory;

    #[test]
    fn test_hull_of_points_empty() {
        let points: Vec<i32> = vec![];

        let hull = Interval::convex_hull(points);
        assert_eq!(hull, Interval::empty());
    }

    #[test]
    fn test_hull_of_points_by_value() {
        let points = vec![5, 3, -1, 30, 2, -22, 100, -100];

        let hull = Interval::convex_hull(points);
        assert_eq!(hull, Interval::closed(-100, 100));
    }

    #[test]
    fn test_hull_of_points_by_reference() {
        let points = vec![5, 3, -1, 30, 2, -22, 100, -100];

        let hull = Interval::convex_hull(points.iter());
        assert_eq!(hull, Interval::closed(-100, 100));
    }

    #[test]
    fn test_hull_of_intervals_empty() {
        let items: Vec<u32> = vec![];
        assert_eq!(Interval::convex_hull(items), Interval::empty())
    }

    #[test]
    fn test_hull_of_intervals_by_value() {
        let items = vec![
            Interval::empty(),
            Interval::empty(),
            Interval::closed(0, 10),
            Interval::empty(),
            Interval::empty(),
        ];
        let hull = Interval::convex_hull(items);
        assert_eq!(hull, Interval::closed(0, 10));
    }

    #[test]
    fn test_hull_of_intervals_by_reference() {
        let items = vec![
            Interval::empty(),
            Interval::empty(),
            Interval::closed(0, 10),
            Interval::empty(),
            Interval::empty(),
        ];
        let hull = Interval::convex_hull(items.iter());
        assert_eq!(hull, Interval::closed(0, 10));
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
    fn test_hull_of_sets_empty() {
        let sets: Vec<IntervalSet<f32>> = vec![];
        let hull = Interval::convex_hull(sets);
        assert_eq!(hull, Interval::empty())
    }

    #[test]
    fn test_hull_of_sets_by_value() {
        let sets: Vec<IntervalSet<f64>> = vec![
            IntervalSet::empty(),
            Interval::closed(0.0, 10.0)
                .union(Interval::open(100.0, 110.0))
                .union(Interval::open(200.0, 210.0)),
            IntervalSet::empty(),
            Interval::closed(-110.0, -100.0).union(Interval::closed(-1000.0, -900.0)),
        ];
        assert_eq!(
            Interval::convex_hull(sets),
            Interval::closed_open(-1000.0, 210.0)
        );

        let sets: Vec<IntervalSet<i32>> = vec![
            Interval::closed(0, 10).union(Interval::closed(1000, 1010)),
            Interval::closed(-1000, 10).into(),
            Interval::closed(-500, 500).union(Interval::closed_unbound(800)),
        ];
        let hull: Interval<i32> = Interval::<i32>::convex_hull(sets);
        assert_eq!(hull, Interval::closed_unbound(-1000))
    }

    #[test]
    fn test_hull_of_sets_by_reference() {
        let sets: Vec<IntervalSet<f64>> = vec![
            IntervalSet::empty(),
            Interval::closed(0.0, 10.0)
                .union(Interval::open(100.0, 110.0))
                .union(Interval::open(200.0, 210.0)),
            IntervalSet::empty(),
            Interval::closed(-110.0, -100.0).union(Interval::closed(-1000.0, -900.0)),
        ];
        assert_eq!(
            Interval::convex_hull(sets.iter()),
            Interval::closed_open(-1000.0, 210.0)
        );

        let sets: Vec<IntervalSet<i32>> = vec![
            Interval::closed(0, 10).union(Interval::closed(1000, 1010)),
            Interval::closed(-1000, 10).into(),
            Interval::closed(-500, 500).union(Interval::closed_unbound(800)),
        ];
        let hull: Interval<i32> = Interval::convex_hull(sets.iter());
        assert_eq!(hull, Interval::closed_unbound(-1000))
    }
}

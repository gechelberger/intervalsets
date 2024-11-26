pub use intervalsets_core::ops::Intersection;
use intervalsets_core::ops::SetSetIntersection;

use crate::numeric::Element;
use crate::{Interval, IntervalSet, MaybeEmpty};

/// The intersection of two sets.
///
/// ```text
/// {x | x ∈ A ∧ x ∈ B }
/// ```
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x = Interval::closed(0, 10);
/// let y = Interval::closed(5, 15);
/// assert_eq!(x.intersection(y), Interval::closed(5, 10));
///
/// let y = Interval::closed(20, 30);
/// assert!(x.intersection(y).is_empty());
/// ```
impl<T: Element> Intersection<Self> for Interval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0.strict_intersection(rhs.0).map(Interval::from)
    }
}

impl<T: Element + Clone> Intersection<Interval<T>> for IntervalSet<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn strict_intersection(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        // invariants: intervals remain sorted; remain disjoint; filter out empty results;
        let intervals = self
            .into_iter()
            .map(|iv| iv.strict_intersection(rhs.clone()))
            .collect::<Result<Vec<_>, Self::Error>>()?;

        let intervals = intervals.into_iter().filter(|iv| !iv.is_empty());

        Ok(Self::new_unchecked(intervals))
    }
}

impl<T: Element + Clone> Intersection<IntervalSet<T>> for Interval<T> {
    type Output = IntervalSet<T>;
    type Error = crate::error::Error;

    fn strict_intersection(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        rhs.strict_intersection(self)
    }
}

impl<T: Element + Clone> Intersection<Self> for IntervalSet<T> {
    type Output = IntervalSet<T>;
    type Error = crate::error::Error;

    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let intersection =
            SetSetIntersection::new(self.into_iter().map(|x| x.0), rhs.into_iter().map(|x| x.0))
                .collect::<Result<Vec<_>, crate::error::Error>>()?;

        Ok(Self::new_unchecked(
            intersection.into_iter().map(Interval::from),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_finite_intersection_empty() {
        // (---A---) (---B---)
        assert_eq!(
            Interval::open(0, 10).intersection(Interval::open(20, 30)),
            Interval::empty()
        );

        // (---B---) (---A---)
        assert_eq!(
            Interval::open(20, 30).intersection(Interval::open(0, 10)),
            Interval::empty()
        );

        // (---A---)
        //         [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(Interval::closed(10, 20)),
            Interval::empty()
        )
    }

    #[test]
    fn test_finite_intersection_fully() {
        // (---A---)
        //   (-B-)
        assert_eq!(
            Interval::open(0, 30).intersection(Interval::open(10, 20)),
            Interval::open(10, 20)
        );

        //   (-A-)
        // (---B---)
        assert_eq!(
            Interval::open(10, 20).intersection(Interval::open(0, 30)),
            Interval::open(10, 20)
        );

        //   [-A-]
        // (---B---)
        assert_eq!(
            Interval::closed(10, 20).intersection(Interval::open(0, 30)),
            Interval::closed(10, 20)
        );

        // (---A---)
        // [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(Interval::closed(0, 10)),
            Interval::open(0, 10)
        )
    }

    #[test]
    fn test_finite_intersection_partial() {
        // |---A---|
        //     |---B---|
        assert_eq!(
            Interval::open(0, 100).intersection(Interval::open(50, 150)),
            Interval::open(50, 100)
        );

        //     |---A---|
        // |---B---|
        assert_eq!(
            Interval::open(50, 150).intersection(Interval::open(0, 100)),
            Interval::open(50, 100)
        );

        // [---A---]
        //     (---B---)
        assert_eq!(
            Interval::closed(0, 10).intersection(Interval::open(5, 15)),
            Interval::open_closed(5, 10)
        );

        // (---A---)
        //     [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(Interval::closed(5, 15)),
            Interval::closed_open(5, 10)
        );
    }

    #[test]
    fn test_half_intersection_same_side() {
        // [--->
        //    [--->
        assert_eq!(
            Interval::closed_unbound(0).intersection(Interval::closed_unbound(50)),
            Interval::closed_unbound(50)
        );

        //    [--->
        // [--->
        assert_eq!(
            Interval::closed_unbound(50).intersection(Interval::closed_unbound(0)),
            Interval::closed_unbound(50)
        );

        // <----]
        // <-------]
        assert_eq!(
            Interval::unbound_closed(0).intersection(Interval::unbound_closed(50)),
            Interval::unbound_closed(0)
        );

        // <-------]
        // <----]
        assert_eq!(
            Interval::unbound_closed(50).intersection(Interval::unbound_closed(0)),
            Interval::unbound_closed(0)
        );

        // [----->
        // (----->
        assert_eq!(
            Interval::closed_unbound(0).intersection(Interval::open_unbound(0)),
            Interval::open_unbound(0)
        )
    }

    #[test]
    fn test_set_set_intersection() {
        let a: IntervalSet<i32> = IntervalSet::new_unchecked(vec![
            Interval::closed(0, 100),
            Interval::closed(500, 600),
            Interval::closed(1000, 1100),
            Interval::closed(10000, 11000),
        ]);

        let b: IntervalSet<i32> = IntervalSet::new_unchecked(vec![
            Interval::closed(-500, -400),  // dropped
            Interval::closed(-300, -200),  // dropped
            Interval::closed(-50, 10),     // [0, 10]
            Interval::closed(20, 30),      // [20, 30]
            Interval::closed(50, 60),      // [50, 60],
            Interval::closed(90, 150),     // [90, 100],
            Interval::closed(200, 300),    // dropped
            Interval::closed(350, 450),    // dropped
            Interval::closed(490, 510),    // [500, 510]
            Interval::closed(550, 560),    // [550, 560]
            Interval::closed(590, 610),    // [590, 600]
            Interval::closed_unbound(800), // [1000, 1100] + [10000, 11000]
        ]);

        assert_eq!(
            a.intersection(b),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::closed(0, 10),
                Interval::closed(20, 30),
                Interval::closed(50, 60),
                Interval::closed(90, 100),
                Interval::closed(500, 510),
                Interval::closed(550, 560),
                Interval::closed(590, 600),
                Interval::closed(1000, 1100),
                Interval::closed(10000, 11000)
            ])
        );
    }

    #[test]
    fn test_set_set_intersection2() {
        let a: IntervalSet<i32> = IntervalSet::new_unchecked(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 100),
            Interval::closed(1000, 1100),
            Interval::closed_unbound(10000),
        ]);

        let b: IntervalSet<i32> = IntervalSet::new_unchecked(vec![
            Interval::unbound_closed(-1000), // full
            Interval::closed(-500, -400),    // full
            Interval::closed(-50, -40),      // drop
            Interval::closed(-10, 10),       // [0, 10]
            Interval::closed_unbound(12000), // full
        ]);

        assert_eq!(
            a.intersection(b),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::unbound_closed(-1000),
                Interval::closed(-500, -400),
                Interval::closed(0, 10),
                Interval::closed_unbound(12000)
            ])
        )
    }
}

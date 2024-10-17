use crate::sets::IntervalSet;
use crate::util::commutative_op_impl;
use crate::{Bound, Domain, Interval};

use super::bounding::Bounding;
use super::empty::MaybeEmpty;

/// Defines the intersection of two sets.
pub trait Intersection<Rhs = Self> {
    type Output;

    fn intersection(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Domain> Intersection<Self> for Interval<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        self.0.intersection(&rhs.0).into()
    }
}

impl<T: Domain> Intersection<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Interval<T>) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::new_unchecked(vec![]);
        }

        // invariants:
        // intervals remain sorted; remain disjoint; filter out empty results;
        let intervals = self
            .intervals()
            .iter()
            .map(|iv| iv.intersection(rhs))
            .filter(|iv| !iv.is_empty())
            .collect();

        Self::new_unchecked(intervals)
    }
}
commutative_op_impl!(
    Intersection,
    intersection,
    Interval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);

impl<T: Domain> Intersection<Self> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::new_unchecked(vec![]);
        }

        let mut it_a = itertools::put_back(self.intervals().iter());
        let mut it_b = itertools::put_back(rhs.intervals().iter());

        let mut intervals: Vec<Interval<T>> = Vec::new();

        loop {
            match (it_a.next(), it_b.next()) {
                (_, None) => break,
                (None, _) => break,
                (Some(a), Some(b)) => {
                    let result = a.intersection(b);
                    if !result.is_empty() {
                        intervals.push(result);

                        // keep the one with the right most right endpoint
                        let ra = a.right();
                        let rb = b.right();
                        match (ra, rb) {
                            (None, None) => continue, // both +inf
                            (Some(_), None) => {
                                it_b.put_back(b);
                            } // b = (_, ->)
                            (None, Some(_)) => {
                                it_a.put_back(a);
                            } // a = (_, ->)
                            (Some(ra), Some(rb)) => {
                                if ra == rb {
                                    continue;
                                }

                                let r_max = Bound::max_right(ra, rb);
                                if *ra == r_max {
                                    it_a.put_back(a);
                                } else {
                                    it_b.put_back(b);
                                }
                            }
                        }
                    } else {
                        // no intersection
                        // keep the one with the right most left endpoint
                        match (a.left(), b.left()) {
                            (None, None) => continue, // both -inf
                            (Some(_), None) => {
                                it_a.put_back(a);
                            }
                            (None, Some(_)) => {
                                it_b.put_back(b);
                            }
                            (Some(la), Some(lb)) => {
                                let l_max = Bound::max_left(la, lb);
                                if *la == l_max {
                                    it_a.put_back(a);
                                } else {
                                    it_b.put_back(b);
                                }
                            }
                        }
                    }
                }
            }
        }

        Self::new_unchecked(intervals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_intersection_empty() {
        // (---A---) (---B---)
        assert_eq!(
            Interval::open(0, 10).intersection(&Interval::open(20, 30)),
            Interval::empty()
        );

        // (---B---) (---A---)
        assert_eq!(
            Interval::open(20, 30).intersection(&Interval::open(0, 10)),
            Interval::empty()
        );

        // (---A---)
        //         [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(&Interval::closed(10, 20)),
            Interval::empty()
        )
    }

    #[test]
    fn test_finite_intersection_fully() {
        // (---A---)
        //   (-B-)
        assert_eq!(
            Interval::open(0, 30).intersection(&Interval::open(10, 20)),
            Interval::open(10, 20)
        );

        //   (-A-)
        // (---B---)
        assert_eq!(
            Interval::open(10, 20).intersection(&Interval::open(0, 30)),
            Interval::open(10, 20)
        );

        //   [-A-]
        // (---B---)
        assert_eq!(
            Interval::closed(10, 20).intersection(&Interval::open(0, 30)),
            Interval::closed(10, 20)
        );

        // (---A---)
        // [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(&Interval::closed(0, 10)),
            Interval::open(0, 10)
        )
    }

    #[test]
    fn test_finite_intersection_partial() {
        // |---A---|
        //     |---B---|
        assert_eq!(
            Interval::open(0, 100).intersection(&Interval::open(50, 150)),
            Interval::open(50, 100)
        );

        //     |---A---|
        // |---B---|
        assert_eq!(
            Interval::open(50, 150).intersection(&Interval::open(0, 100)),
            Interval::open(50, 100)
        );

        // [---A---]
        //     (---B---)
        assert_eq!(
            Interval::closed(0, 10).intersection(&Interval::open(5, 15)),
            Interval::open_closed(5, 10)
        );

        // (---A---)
        //     [---B---]
        assert_eq!(
            Interval::open(0, 10).intersection(&Interval::closed(5, 15)),
            Interval::closed_open(5, 10)
        );
    }

    #[test]
    fn test_half_intersection_same_side() {
        // [--->
        //    [--->
        assert_eq!(
            Interval::closed_unbound(0).intersection(&Interval::closed_unbound(50)),
            Interval::closed_unbound(50)
        );

        //    [--->
        // [--->
        assert_eq!(
            Interval::closed_unbound(50).intersection(&Interval::closed_unbound(0)),
            Interval::closed_unbound(50)
        );

        // <----]
        // <-------]
        assert_eq!(
            Interval::unbound_closed(0).intersection(&Interval::unbound_closed(50)),
            Interval::unbound_closed(0)
        );

        // <-------]
        // <----]
        assert_eq!(
            Interval::unbound_closed(50).intersection(&Interval::unbound_closed(0)),
            Interval::unbound_closed(0)
        );

        // [----->
        // (----->
        assert_eq!(
            Interval::closed_unbound(0).intersection(&Interval::open_unbound(0)),
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
            a.intersection(&b),
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
            a.intersection(&b),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::unbound_closed(-1000),
                Interval::closed(-500, -400),
                Interval::closed(0, 10),
                Interval::closed_unbound(12000)
            ])
        )
    }
}

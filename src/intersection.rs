use crate::bounds::Bounds;
use crate::empty::MaybeEmpty;
use crate::ival::{IVal, Side};
use crate::util::commutative_op_impl;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

use crate::pred::contains::Contains;

pub trait Intersection<Rhs = Self> {
    type Output;

    fn intersection(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Copy + PartialOrd> Intersection<Self> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        self.map_or(FiniteInterval::Empty, |a_left, a_right| {
            rhs.map_bounds(|b_left, b_right| {
                // new() will clean up empty sets where left & right have violated bounds
                FiniteInterval::new(
                    IVal::max_left(a_left, b_left),
                    IVal::min_right(a_right, b_right),
                )
            })
        })
    }
}

impl<T: Copy + PartialOrd> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn intersection(&self, rhs: &HalfInterval<T>) -> Self::Output {
        self.map_bounds(|left, right| {
            let n_seen = [left, right]
                .into_iter()
                .filter(|end| rhs.contains(&end.value))
                .count();

            if n_seen == 2 {
                Self::new(*left, *right)
            } else if n_seen == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.ival, *right),
                    Side::Right => Self::new(*left, rhs.ival),
                }
            } else {
                Self::Empty
            }
        })
    }
}

impl<T: Copy + PartialOrd> Intersection<Self> for HalfInterval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(&rhs.ival.value) {
                rhs.clone().into()
            } else {
                self.clone().into()
            }
        } else {
            // new() handles degenerate cases => Empty
            match self.side {
                Side::Left => FiniteInterval::new(self.ival, rhs.ival),
                Side::Right => FiniteInterval::new(rhs.ival, self.ival),
            }
            .into()
        }
    }
}

impl<T: Copy + PartialOrd> Intersection<FiniteInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Infinite => rhs.clone().into(),
            Self::Half(lhs) => lhs.intersection(rhs).into(),
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
        }
    }
}

impl<T: Copy + PartialOrd> Intersection<HalfInterval<T>> for Interval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &HalfInterval<T>) -> Self::Output {
        match self {
            Self::Infinite => rhs.clone().into(),
            Self::Half(lhs) => lhs.intersection(rhs),
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
        }
    }
}

impl<T: Copy + PartialOrd> Intersection<Self> for Interval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        match self {
            Self::Infinite => rhs.clone(),
            Self::Half(lhs) => rhs.intersection(lhs),
            Self::Finite(lhs) => rhs.intersection(lhs),
        }
    }
}

commutative_op_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    FiniteInterval<T>,
    FiniteInterval<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    FiniteInterval<T>,
    Interval<T>,
    Interval<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    Interval<T>,
    Interval<T>
);

impl<T: Copy + PartialOrd> Intersection<FiniteInterval<T>> for IntervalSet<T> {
    type Output = Self;

    fn intersection(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::new_unchecked(vec![]);
        }

        // invariants:
        // intervals remain sorted; remain disjoint; filter out empty results;
        let intervals = self
            .intervals
            .iter()
            .map(|iv| iv.intersection(rhs))
            .filter(|iv| !iv.is_empty())
            .collect();

        Self::new_unchecked(intervals)
    }
}

impl<T: Copy + PartialOrd> Intersection<HalfInterval<T>> for IntervalSet<T> {
    type Output = Self;

    fn intersection(&self, rhs: &HalfInterval<T>) -> Self::Output {
        if self.is_empty() {
            // half intervals can not be empty
            return Self::new_unchecked(vec![]);
        }

        // invariants:
        // intervals remain sorted; remain disjoint; filter out empty results;
        let intervals = self
            .intervals
            .iter()
            .map(|iv| iv.intersection(rhs))
            .filter(|iv| !iv.is_empty())
            .collect();

        Self::new_unchecked(intervals)
    }
}

impl<T: Copy + PartialOrd> Intersection<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn intersection(&self, rhs: &Interval<T>) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::new_unchecked(vec![]);
        }

        // invariants:
        // intervals remain sorted; remain disjoint; filter out empty results;
        let intervals = self
            .intervals
            .iter()
            .map(|iv| iv.intersection(rhs))
            .filter(|iv| !iv.is_empty())
            .collect();

        Self::new_unchecked(intervals)
    }
}

impl<T: Copy + PartialOrd> Intersection<Self> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::new_unchecked(vec![]);
        }

        let mut it_a = itertools::put_back(self.intervals.iter());
        let mut it_b = itertools::put_back(rhs.intervals.iter());

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

                                let r_max = IVal::max_right(&ra, &rb);
                                if ra == r_max {
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
                                let l_max = IVal::max_left(&la, &lb);
                                if la == l_max {
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

commutative_op_impl!(
    Intersection,
    intersection,
    FiniteInterval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);
commutative_op_impl!(
    Intersection,
    intersection,
    Interval<T>,
    IntervalSet<T>,
    IntervalSet<T>
);

#[cfg(test)]
mod tests {
    use crate::union::Union;

    use super::*;

    #[test]
    fn test_finite_intersection_empty() {
        // (---A---) (---B---)
        assert_eq!(
            FiniteInterval::open(0, 10).intersection(&FiniteInterval::open(20, 30)),
            FiniteInterval::Empty
        );

        // (---B---) (---A---)
        assert_eq!(
            FiniteInterval::open(20, 30).intersection(&FiniteInterval::open(0, 10)),
            FiniteInterval::Empty
        );

        // (---A---)
        //         [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).intersection(&FiniteInterval::closed(10, 20)),
            FiniteInterval::Empty
        )
    }

    #[test]
    fn test_finite_intersection_fully() {
        // (---A---)
        //   (-B-)
        assert_eq!(
            FiniteInterval::open(0, 30).intersection(&FiniteInterval::open(10, 20)),
            FiniteInterval::open(10, 20)
        );

        //   (-A-)
        // (---B---)
        assert_eq!(
            FiniteInterval::open(10, 20).intersection(&FiniteInterval::open(0, 30)),
            FiniteInterval::open(10, 20)
        );

        //   [-A-]
        // (---B---)
        assert_eq!(
            FiniteInterval::closed(10, 20).intersection(&FiniteInterval::open(0, 30)),
            FiniteInterval::closed(10, 20)
        );

        // (---A---)
        // [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).intersection(&FiniteInterval::closed(0, 10)),
            FiniteInterval::open(0, 10)
        )
    }

    #[test]
    fn test_finite_intersection_partial() {
        // |---A---|
        //     |---B---|
        assert_eq!(
            FiniteInterval::open(0, 100).intersection(&FiniteInterval::open(50, 150)),
            FiniteInterval::open(50, 100)
        );

        //     |---A---|
        // |---B---|
        assert_eq!(
            FiniteInterval::open(50, 150).intersection(&FiniteInterval::open(0, 100)),
            FiniteInterval::open(50, 100)
        );

        // [---A---]
        //     (---B---)
        assert_eq!(
            FiniteInterval::closed(0, 10).intersection(&FiniteInterval::open(5, 15)),
            FiniteInterval::openclosed(5, 10)
        );

        // (---A---)
        //     [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).intersection(&FiniteInterval::closed(5, 15)),
            FiniteInterval::closedopen(5, 10)
        );
    }

    #[test]
    fn test_half_intersection_same_side() {
        // [--->
        //    [--->
        assert_eq!(
            HalfInterval::closed_unbound(0).intersection(&HalfInterval::closed_unbound(50)),
            HalfInterval::closed_unbound(50).into()
        );

        //    [--->
        // [--->
        assert_eq!(
            HalfInterval::closed_unbound(50).intersection(&HalfInterval::closed_unbound(0)),
            HalfInterval::closed_unbound(50).into()
        );

        // <----]
        // <-------]
        assert_eq!(
            HalfInterval::unbound_closed(0).intersection(&HalfInterval::unbound_closed(50)),
            HalfInterval::unbound_closed(0).into()
        );

        // <-------]
        // <----]
        assert_eq!(
            HalfInterval::unbound_closed(50).intersection(&HalfInterval::unbound_closed(0)),
            HalfInterval::unbound_closed(0).into()
        );

        // [----->
        // (----->
        assert_eq!(
            HalfInterval::closed_unbound(0).intersection(&HalfInterval::open_unbound(0)),
            HalfInterval::open_unbound(0).into()
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

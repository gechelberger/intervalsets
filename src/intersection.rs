use crate::finite::FiniteInterval;
use crate::infinite::{Interval, IntervalSet};
use crate::ival::{IVal, Side};
use crate::util::commutative_impl;
use crate::HalfInterval;

use crate::contains::Contains;

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
                Self::new(left.clone(), right.clone())
            } else if n_seen == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.ival, right.clone()),
                    Side::Right => Self::new(left.clone(), rhs.ival),
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
                return rhs.clone().into();
            } else {
                return self.clone().into();
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

commutative_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    FiniteInterval<T>,
    FiniteInterval<T>
);
commutative_impl!(
    Intersection,
    intersection,
    FiniteInterval<T>,
    Interval<T>,
    Interval<T>
);
commutative_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    Interval<T>,
    Interval<T>
);

////////////////

impl<T: Copy + PartialOrd> Intersection<Self> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        todo!()
    }
}

impl<T: Copy + PartialOrd> Intersection<Interval<T>> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &Interval<T>) -> Self::Output {
        todo!()
    }
}

impl<T: Copy + PartialOrd> Intersection<HalfInterval<T>> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &HalfInterval<T>) -> Self::Output {
        todo!()
    }
}

impl<T: Copy + PartialOrd> Intersection<FiniteInterval<T>> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn intersection(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_interval_overlapped_empty() {
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
    fn test_finite_interval_overlapped_fully() {
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
    fn test_finite_interval_overlapped() {
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
}

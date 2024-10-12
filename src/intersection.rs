use crate::infinite::{Interval, IntervalSet};
use crate::finite::FiniteInterval;
use crate::ival::Side;
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
                let new_left = if a_left.contains(Side::Left, &b_left.value) {
                    *b_left
                } else {
                    *a_left
                };
                let new_right = if a_right.contains(Side::Right, &b_right.value) {
                    *b_right
                } else {
                    *a_right
                };
                // new() will clean up empty sets where left & right have violated bounds
                FiniteInterval::new(new_left, new_right)
            })
        })
    }
}

impl<T: Copy + PartialOrd> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn intersection(&self, rhs: &HalfInterval<T>) -> Self::Output {
        self.map_bounds(|left, right| {
            let n_seen = [left, right].into_iter()
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

impl<T: Copy + PartialOrd> Intersection<FiniteInterval<T>> for HalfInterval<T> {
    type Output = FiniteInterval<T>;

    fn intersection(&self, rhs: &FiniteInterval<T>) -> Self::Output {
        rhs.intersection(self)
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
            }.into()
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

impl<T: Copy + PartialOrd> Intersection<Interval<T>> for FiniteInterval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &Interval<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

impl<T: Copy + PartialOrd> Intersection<Interval<T>> for HalfInterval<T> {
    type Output = Interval<T>;

    fn intersection(&self, rhs: &Interval<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

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
use std::ops::Sub;

use num::Zero;

use crate::{
    half::HalfInterval,
    ival::{Bound, IVal, Side},
    FiniteInterval,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<T> {
    /// (a, a) = (a, a] = [a, a) = Empty { x not in T }
    /// [a, a] = NonZero { x in T |    x = a    }
    /// (a, b) = NonZero { x in T | a <  x <  b }
    /// (a, b] = NonZero { x in T | a <  x <= b }
    /// [a, b) = NonZero { x in T | a <= x <  b }
    /// [a, b] = NonZero { x in T | a <= x <= b }
    Finite(FiniteInterval<T>),

    /// (a, ->) = Left  { x in T | a <  x      }
    /// [a, ->) = Left  { x in T | a <= x      }
    /// (<-, b) = Right { x in T |      x < b  }
    /// (<-, b] = Right { x in T |      x <= b }
    Half(HalfInterval<T>),

    /// {<-, ->) = { x in T }
    Infinite,
}

impl<T> Interval<T>
where
    T: Copy + PartialOrd,
{
    pub fn empty() -> Self {
        FiniteInterval::Empty.into()
    }

    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        FiniteInterval::new(IVal::new(Bound::Open, left), IVal::new(Bound::Open, right)).into()
    }

    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        )
        .into()
    }

    /// (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        )
        .into()
    }

    /// [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        )
        .into()
    }

    // (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        HalfInterval::new(Side::Right, IVal::new(Bound::Open, right)).into()
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        HalfInterval::new(Side::Right, IVal::new(Bound::Closed, right)).into()
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        HalfInterval::new(Side::Left, IVal::new(Bound::Open, left)).into()
    }

    /// [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        HalfInterval::new(Side::Left, IVal::new(Bound::Closed, left)).into()
    }

    pub fn unbound() -> Self {
        Self::Infinite
    }

    pub fn lval_unchecked(&self) -> T {
        match self {
            Self::Finite(interval) => interval.lval_unchecked(),
            Self::Half(interval) => interval.lval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }

    pub fn rval_unchecked(&self) -> T {
        match self {
            Self::Finite(interval) => interval.rval_unchecked(),
            Self::Half(interval) => interval.rval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }

    /*
    pub fn complement(&self) -> Vec<Self> {
        match self {
            Self::Empty => vec![Self::Infinite],
            Self::Infinite => vec![Self::Empty],
            Self::Half(data) => {
                let (side, ival) = data;
                vec![Self::Half((side.flip(), ival.flip()))]
            }
            Self::Finite(data) => {
                let (left, right) = data;
                vec![
                    Self::Half((Side::Right, left.flip())),
                    Self::Half((Side::Left, right.flip())),
                ]
            }
        }
    }

    pub fn union(&self, other: &Self) -> Vec<Self> {
        match (self, other) {
            (Self::Empty, _) => vec![other.clone()],
            (_, Self::Empty) => vec![self.clone()],
            (Self::Infinite, _) => vec![Self::Infinite],
            (_, Self::Infinite) => vec![Self::Infinite],
            (Self::Half(lhs), Self::Half(rhs)) => Self::union_half_half(lhs, rhs),
            (Self::Finite(lhs), Self::Finite(rhs)) => Self::union_finite_finite(lhs, rhs),
            (Self::Half(lhs), Self::Finite(rhs)) => Self::union_finite_half(rhs, lhs),
            (Self::Finite(lhs), Self::Half(rhs)) => Self::union_finite_half(lhs, rhs),
        }
    }

    fn union_half_half(a: &(Side, IVal<T>), b: &(Side, IVal<T>)) -> Vec<Self> {
        let (a_side, a_ival) = a;
        let (b_side, b_ival) = b;

        if a_side == b_side {
            if a_ival.contains(*a_side, &b_ival.value) {
                return vec![Self::Half(*a)];
            } else {
                return vec![Self::Half(*b)];
            }
        } else {
            if a_ival.contains(*a_side, &b_ival.value) {
                return vec![Self::Infinite];
            } else {
                return vec![Self::Half(*a), Self::Half(*b)];
            }
        }
    }



    fn union_finite_half(finite: &(IVal<T>, IVal<T>), half: &(Side, IVal<T>)) -> Vec<Self> {
        let (a_left, a_right) = finite;
        let (h_side, h_ival) = half;

        if a_left.contains(Side::Left, &h_ival.value)
            && a_right.contains(Side::Right, &h_ival.value)
        {
            // half interval starts in the finite interval
            // keep the `side`` of the half interval but using the bound from the finite one
            let new_bound = match h_side {
                Side::Left => a_left.clone(),
                Side::Right => a_right.clone(),
            };

            vec![Interval::Half((*h_side, new_bound))]
        } else {
            let half = Interval::Half(half.clone());

            if half.contains(&a_left.value) {
                // implies contains a_right too
                // half interval fully contains finite interval
                vec![half]
            } else {
                // disjoint intervals
                vec![half, Interval::Finite(*finite)]
            }
        }
    }

    */
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntervalSet<T> {
    pub intervals: Vec<Interval<T>>,
}

#[allow(dead_code)]
impl<T: Copy + PartialOrd + Zero + Sub<Output = T>> IntervalSet<T> {
    fn new() -> Self {
        Self { intervals: vec![] }
    }

    fn complement(&self) -> Self {
        // complement of all sub intervals
        // then folded intersection of those?
        let mut cloned = self.clone();
        cloned.complement_mut();
        cloned
    }

    fn complement_mut(&mut self) -> &mut Self {
        self
    }

    fn union(&mut self, other: &Self) -> &Self {
        todo!()
    }

    fn union_interval(&mut self, other: &Interval<T>) -> &Self {
        todo!()
    }

    fn difference(&self, rhs: &Self) -> Self {
        let mut cloned = self.clone();
        cloned.difference_mut(rhs);
        cloned
    }

    fn difference_mut(&mut self, rhs: &Self) -> &Self {
        //self.intersection_mut(rhs.complement());
        //self
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::contains::Contains;
    use crate::normalize::Normalize;

    use super::*;

    #[quickcheck]
    fn test_half_interval_contains_f64(x: f64) {
        let interval: Interval<f64> = Interval::unbound_open(0.0);
        assert_eq!(interval.contains(&x), x < 0.0);

        let interval: Interval<f64> = Interval::unbound_closed(1.0);
        assert_eq!(interval.contains(&x), x <= 1.0);

        let interval: Interval<f64> = Interval::open_unbound(0.0);
        assert_eq!(interval.contains(&x), x > 0.0);

        let interval: Interval<f64> = Interval::closed_unbound(1.0);
        assert_eq!(interval.contains(&x), x >= 1.0);
    }

    #[quickcheck]
    fn test_half_interval_contains_u64(x: u64) {
        let interval: Interval<u64> = Interval::unbound_open(100);
        assert_eq!(interval.contains(&x), x < 100);

        let interval: Interval<u64> = Interval::unbound_closed(100);
        assert_eq!(interval.contains(&x), x <= 100);

        let interval: Interval<u64> = Interval::open_unbound(100);
        assert_eq!(interval.contains(&x), x > 100);
    }

    /*

    #[quickcheck]
    fn test_half_interval_complement_i64(x: i64) {
        let interval: Interval<i64> = Interval::closed_unbound(0);
        let complement = &interval.complement()[0];

        assert_eq!(interval.contains(&x), !complement.contains(&x));
    }

    */
}

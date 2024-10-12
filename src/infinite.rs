use std::ops::Sub;

use num::{Zero};

use crate::{half::HalfInterval, ival::{Bound, IVal, Side}, FiniteInterval};


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
    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Open, left), 
            IVal::new(Bound::Open, right)
        ).into()
    }

    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        ).into()
    }

    /// (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        ).into()
    }

    /// [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        ).into()
    }

    // (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        HalfInterval::new(
            Side::Right, 
            IVal::new(Bound::Open, right)
        ).into()
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

    pub fn lval_unchecked(&self) -> T {
        match self {
            Self::Finite(interval) => {
                interval.lval_unchecked()
            },
            Self::Half(interval) => {
                interval.lval_unchecked()
            },
            _ => panic!("left bound of interval is not in T")
        }
    }

    pub fn rval_unchecked(&self) -> T {
        match self {
            Self::Finite(interval) => {
                interval.rval_unchecked()
            },
            Self::Half(interval) => {
                interval.rval_unchecked()
            },
            _ => panic!("left bound of interval is not in T")
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

    fn union_finite_finite(a: &(IVal<T>, IVal<T>), b: &(IVal<T>, IVal<T>)) -> Vec<Self> {
        let (a_left, a_right) = a;
        let (b_left, b_right) = b;

        // must check from both left and right to ensure open/closed bounds are properly handled
        let overlapping = a_left.contains(Side::Left, &b_right.value)
            && b_left.contains(Side::Left, &a_right.value)
            && a_right.contains(Side::Right, &b_left.value)
            && b_right.contains(Side::Right, &a_left.value);

        if !overlapping {
            vec![Interval::Finite(a.clone()), Interval::Finite(b.clone())]
        } else {
            let left = if a_left.contains(Side::Left, &b_left.value) {
                a_left
            } else {
                b_left
            };

            let right = if a_right.contains(Side::Right, &b_right.value) {
                a_right
            } else {
                b_right
            };

            vec![Interval::new_finite(left.clone(), right.clone())]
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

    pub fn intersection(&self, other: &Interval<T>) -> Self {
        match (self, other) {
            (Self::Empty, _) => Self::Empty,
            (_, Self::Empty) => Self::Empty,
            (Self::Infinite, _) => other.clone(),
            (_, Self::Infinite) => self.clone(),
            (Self::Half(lhs), Self::Half(rhs)) => Self::intersection_half_half(lhs, rhs),
            (Self::Finite(lhs), Self::Finite(rhs)) => Self::intersection_finite_finite(lhs, rhs),
            (Self::Half(lhs), Self::Finite(rhs)) => Self::intersection_finite_half(rhs, lhs),
            (Self::Finite(lhs), Self::Half(rhs)) => Self::intersection_finite_half(lhs, rhs),
        }
    }

    fn intersection_half_half(a: &(Side, IVal<T>), b: &(Side, IVal<T>)) -> Self {
        let (a_side, a_ival) = a;
        let (b_side, b_ival) = b;

        if a_side == b_side {
            if a_ival.contains(*a_side, &b_ival.value) {
                return Interval::Half(*b);
            } else {
                return Interval::Half(*a);
            }
        } else {
            match a_side {
                Side::Left => Self::new_finite(*a_ival, *b_ival),
                Side::Right => Self::new_finite(*b_ival, *a_ival),
            }
        }
    }

    fn intersection_finite_finite(a: &(IVal<T>, IVal<T>), b: &(IVal<T>, IVal<T>)) -> Self {
        let (a_left, a_right) = a;
        let (b_left, b_right) = b;

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
        Self::new_finite(new_left, new_right)
    }

    /// Three cases:
    /// 1) half interval sees left & right => full finite result
    /// 2) half interval sees 1 => (half -> right) or (left, half)
    /// 3) half interval sees 0 => Empty
    fn intersection_finite_half(finite: &(IVal<T>, IVal<T>), half: &(Side, IVal<T>)) -> Self {
        let finite = [finite.0, finite.1];
        let (h_side, h_ival) = half;

        let n_seen = finite.iter()
            .filter(|ival| h_ival.contains(*h_side, &ival.value))
            .count();

        if n_seen == 2 {
            Self::new_finite(finite[0].clone(), finite[1].clone())
        } else if n_seen == 1 {
            match h_side {
                Side::Left => Self::new_finite(*h_ival, finite[1].clone()),
                Side::Right => Self::new_finite(finite[0].clone(), *h_ival),
            }
        } else {
            Self::Empty
        }
        
    }
    */

}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntervalSet<T> {
    pub intervals: Vec<Interval<T>>,
}

impl<T: Copy + PartialOrd + Zero + Sub<Output = T>> IntervalSet<T> {
    fn new(intervals: Vec<Interval<T>>) -> Self {
        Self { intervals: vec![] }
    }

    fn complement(&self) -> Self {
        // complement of all sub intervals
        // then folded intersection of those?
        let mut cloned = self.clone();
        cloned.complement_mut();
        cloned
    }

    fn complement_mut<'a>(&'a mut self) -> &'a mut Self {

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
    use crate::normalize::Normalize;
    use crate::contains::Contains;

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

    #[quickcheck]
    fn test_half_interval_intersection_i8(x: i8) {
        let interval: Interval<i8> =
            Interval::open_unbound(10).intersection(&Interval::closed_unbound(20));
        assert_eq!(interval.contains(&x), x >= 20);

        let interval: Interval<i8> =
            Interval::unbound_closed(10).intersection(&Interval::unbound_open(0));
        assert_eq!(interval.contains(&x), x < 0);

        let interval: Interval<i8> =
            Interval::unbound_closed(100).intersection(&Interval::closed_unbound(0));
        assert_eq!(interval.contains(&x), 0 <= x && x <= 100);

        let interval: Interval<i8> =
            Interval::unbound_closed(0).intersection(&Interval::open_unbound(0));
        assert_eq!(interval.contains(&x), false);
    }

    #[quickcheck]
    fn test_finite_intersection_i8(x: i8) {
        let interval: Interval<i8> = Interval::open(0, 10).intersection(&Interval::open(5, 15));
        assert_eq!(interval.contains(&x), 5 < x && x < 10);
    }

    #[quickcheck]
    fn test_finite_half_intersection_i8(x: i8) {
        let interval: Interval<i8> = Interval::open(0, 10).intersection(&Interval::closed_unbound(5));
        assert_eq!(interval.contains(&x), 5 <= x && x < 10);
        
        let interval: Interval<i8> = Interval::closed(0, 10).intersection(&Interval::open_unbound(0));
        assert_eq!(interval.contains(&x), 0 < x && x <= 10);
    }

    */

    #[test]
    fn test_normalized_integers() {
        //let interval = Interval::open(50.0, 60.0);
        //let foo = interval.normalized();

        assert_eq!(Interval::open(0, 10).normalized(), Interval::closed(1, 9));
        assert_eq!(Interval::open_closed(0, 10).normalized(), Interval::closed(1, 10));
        assert_eq!(Interval::unbound_open(5 as i8).normalized(), Interval::unbound_closed(4 as i8));
        assert_eq!(Interval::unbound_closed(5 as i8).normalized(), Interval::unbound_closed(5 as i8));
        assert_eq!(Interval::open_unbound(5 as i8).normalized(), Interval::closed_unbound(6 as i8));
    }
}

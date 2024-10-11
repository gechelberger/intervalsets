use std::ops::Sub;

use num::Zero;

use crate::ival::{Bound, IVal, Side};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum ISize<T> {
    Finite(T),
    Infinite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<T> {
    /// (a, a) = (a, a] = [a, a) = Empty { x not in T }
    Empty,

    /// [a, a] = NonZero { x in T |    x = a    }
    /// (a, b) = NonZero { x in T | a <  x <  b }
    /// (a, b] = NonZero { x in T | a <  x <= b }
    /// [a, b) = NonZero { x in T | a <= x <  b }
    /// [a, b] = NonZero { x in T | a <= x <= b }
    Finite((IVal<T>, IVal<T>)),

    /// (a, ->) = Left  { x in T | a <  x      }
    /// [a, ->) = Left  { x in T | a <= x      }
    /// (<-, b) = Right { x in T |      x < b  }
    /// (<-, b] = Right { x in T |      x <= b }
    Half((Side, IVal<T>)),

    /// {<-, ->) = { x in T }
    Infinite,
}

impl<T> Interval<T>
where
    T: Copy + PartialOrd + Sub<Output = T> + Zero,
{
    pub fn new_finite(left: IVal<T>, right: IVal<T>) -> Self {
        if left.value > right.value {
            Self::Empty
        } else if left.value == right.value {
            if left.bound == Bound::Open || right.bound == Bound::Open {
                Self::Empty
            } else {
                Self::new_finite_unchecked(left, right)
            }
        } else {
            Self::new_finite_unchecked(left, right)
        }
    }

    pub fn new_finite_unchecked(left: IVal<T>, right: IVal<T>) -> Self {
        Self::Finite((left, right))
    }

    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        Self::new_finite(IVal::new(Bound::Open, left), IVal::new(Bound::Open, right))
    }

    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        Self::new_finite(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        )
    }

    /// (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        Self::new_finite(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        )
    }

    /// [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        Self::new_finite(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        )
    }

    // (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        Interval::Half((Side::Right, IVal::new(Bound::Open, right)))
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        Interval::Half((Side::Right, IVal::new(Bound::Closed, right)))
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        Interval::Half((Side::Left, IVal::new(Bound::Open, left)))
    }

    /// [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        Interval::Half((Side::Left, IVal::new(Bound::Closed, left)))
    }

    pub fn size(&self) -> ISize<T> {
        match self {
            Self::Empty => ISize::Finite(T::zero()),
            Self::Infinite => ISize::Infinite,
            Self::Half(_) => ISize::Infinite,
            Self::Finite((a, b)) => ISize::Finite(b.value - a.value),
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match self {
            Self::Empty => false,
            Self::Infinite => true,
            Self::Half(data) => {
                let (side, ival) = data;
                ival.contains(*side, value)
            }
            Self::Finite(data) => {
                let (left, right) = data;
                left.contains(Side::Left, value) && right.contains(Side::Right, value)
            }
        }
    }

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
                return vec!{Self::Infinite};
            } else {
                return vec![Self::Half(*a), Self::Half(*b)]
            }
        }
    }

    fn union_finite_finite(a: &(IVal<T>, IVal<T>), b: &(IVal<T>, IVal<T>)) -> Vec<Self> {
        let (a_left, a_right) = a;
        let (b_left, b_right) = b;

        todo!()
    }

    fn union_finite_half(finite: &(IVal<T>, IVal<T>), half: &(Side, IVal<T>)) -> Vec<Self> {
        let (a_left, a_right) = finite;
        let (h_side, h_ival) = half;

        if a_left.contains(Side::Left, &h_ival.value) && a_right.contains(Side::Right, &h_ival.value) {
            // half interval starts in the finite interval
            // keep the `side`` of the half interval but using the bound from the finite one
            let new_bound = match h_side {
                Side::Left => a_left.clone(),
                Side::Right => a_right.clone()
            };

            vec![Interval::Half((*h_side, new_bound))]
        } else {
            let half = Interval::Half(half.clone());

            if half.contains(&a_left.value) {
                // implies contains a_right too
                // half interval fully contains finite interval
                vec![ half ]
            } else {
                // disjoint intervals
                vec![ half, Interval::Finite(*finite) ]
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

    fn intersection_finite_half(finite: &(IVal<T>, IVal<T>), half: &(Side, IVal<T>)) -> Self {
        let (f_left, f_right) = finite;
        let (h_side, h_ival) = half;

        match h_side {
            Side::Left => Self::new_finite(*h_ival, *f_right),
            Side::Right => Self::new_finite(*f_left, *h_ival),
        }
    }
}

struct IntervalSet<T> {
    intervals: Vec<Interval<T>>,
}

impl<T: Copy + PartialOrd + Zero + Sub<Output = T>> IntervalSet<T> {
    fn new() -> Self {
        Self { intervals: vec![] }
    }

    fn contains(&self, x: &T) -> bool {
        self.intervals.iter().any(|iv| iv.contains(x))
    }

    fn complement(&mut self) -> &Self {
        // complement of all sub intervals
        // then folded intersection of those?

        todo!()
    }

    fn set_union(&mut self, other: &Self) -> &Self {
        todo!()
    }

    fn interval_union(&mut self, other: &Interval<T>) -> &Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
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

    #[quickcheck]
    fn test_half_interval_complement_i64(x: i64) {
        let interval: Interval<i64> = Interval::closed_unbound(0);
        let complement = &interval.complement()[0];

        assert_eq!(interval.contains(&x), !complement.contains(&x));
    }

    #[quickcheck]
    fn test_half_interval_intersection(x: i8) {

        let interval: Interval<i8> = Interval::open_unbound(10).intersection(&Interval::closed_unbound(20));
        assert_eq!(interval.contains(&x), x >= 20);

        let interval: Interval<i8> = Interval::unbound_closed(10).intersection(&Interval::unbound_open(0));
        assert_eq!(interval.contains(&x), x < 0);

        let interval: Interval<i8> = Interval::unbound_closed(100).intersection(&Interval::closed_unbound(0));
        assert_eq!(interval.contains(&x), 0 <= x && x <= 100);

        let interval: Interval<i8> = Interval::unbound_closed(0).intersection(&Interval::open_unbound(0));
        assert_eq!(interval.contains(&x), false);
    }
}

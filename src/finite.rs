use std::ops::{Add, Div, Mul, Sub};

use num::{FromPrimitive, Zero};

use crate::ival::*;

/// (a, a) = (a, a] = [a, a) = Empty { x not in T }
/// [a, a] = NonZero { x in T |    x = a    }
/// (a, b) = NonZero { x in T | a <  x <  b }
/// (a, b] = NonZero { x in T | a <  x <= b }
/// [a, b) = NonZero { x in T | a <= x <  b }
/// [a, b] = NonZero { x in T | a <= x <= b }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<T> {
    Empty,
    NonZero(IVal<T>, IVal<T>),
}

impl<T> Interval<T>
where
    T: Copy + Ord + Add<Output = T> + Sub<Output = T> + Div<Output = T> + FromPrimitive,
{
    pub fn new(left: IVal<T>, right: IVal<T>) -> Self {
        if left.value > right.value {
            Self::Empty
        } else if left.value == right.value {
            if left.bound == Bound::Open || right.bound == Bound::Open {
                Self::Empty
            } else {
                // singleton set
                Self::new_unchecked(left, right)
            }
        } else {
            Self::new_unchecked(left, right)
        }
    }

    pub fn new_unchecked(left: IVal<T>, right: IVal<T>) -> Self {
        Self::NonZero(left, right)
    }

    pub fn open(left: T, right: T) -> Self {
        Self::new(IVal::new(Bound::Open, left), IVal::new(Bound::Open, right))
    }

    pub fn closed(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        )
    }

    pub fn openclosed(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        )
    }

    pub fn closedopen(left: T, right: T) -> Self {
        Self::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        )
    }

    pub fn left(&self) -> Option<IVal<T>> {
        match self {
            Self::Empty => None,
            Self::NonZero(left, _) => Some(*left),
        }
    }

    pub fn right(&self) -> Option<IVal<T>> {
        match self {
            Self::Empty => None,
            Self::NonZero(_, right) => Some(*right),
        }
    }

    pub fn lbound(&self) -> Option<Bound> {
        self.left().map(|ival| ival.bound)
    }

    pub fn lval(&self) -> Option<T> {
        self.left().map(|ival| ival.value)
    }

    pub fn rbound(&self) -> Option<Bound> {
        self.right().map(|ival| ival.bound)
    }

    pub fn rval(&self) -> Option<T> {
        self.right().map(|ival| ival.value)
    }

    pub fn size(&self) -> T {
        match self {
            Self::Empty => T::from_u64(0).expect("T from 0 as u64 failed"),
            Self::NonZero(left, right) => right.value - left.value,
        }
    }

    pub fn center(&self) -> Option<T> {
        match self {
            Self::Empty => None,
            Self::NonZero(left, right) => Some(
                left.value
                    + self.size() / T::from_u64(2).expect("T from 2 as u64 failed in center()"),
            ),
        }
    }

    pub fn contains(&self, value: T) -> bool {
        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => {
                left.contains(Side::Left, value) && right.contains(Side::Right, value)
            }
        }
    }

    pub fn overlaps(&self, other: Interval<T>) -> bool {
        // probably cheaper ways to do it...
        self.overlapped(other) != Interval::Empty
    }

    pub fn overlapped(&self, other: Interval<T>) -> Interval<T> {
        match (self, other) {
            (Interval::Empty, _) => Interval::Empty,
            (_, Interval::Empty) => Interval::Empty,
            (Interval::NonZero(a_left, a_right), Interval::NonZero(b_left, b_right)) => {
                let new_left = if a_left.contains(Side::Left, b_left.value) {
                    b_left
                } else {
                    *a_left
                };
                let new_right = if a_right.contains(Side::Right, b_right.value) {
                    b_right
                } else {
                    *a_right
                };

                // new() will clean up empty sets where left & right have violated bounds
                Interval::new(new_left, new_right)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_interval_new() {
        let interval: Interval<usize> = Interval::open(0, 20);
    }

    #[test]
    fn test_finite_interval_contains() {
        let iv = Interval::open(-100, 100);
        assert!(iv.contains(0));
        assert!(iv.contains(50));
        assert!(!iv.contains(100));
        assert!(!iv.contains(1000));

        assert!(iv.contains(-50));
        assert!(!iv.contains(-100));
        assert!(!iv.contains(-1000));
    }

    #[test]
    fn test_finite_interval_overlapped_empty() {
        // (---A---) (---B---)
        assert_eq!(
            Interval::open(0, 10).overlapped(Interval::open(20, 30)),
            Interval::Empty
        );

        // (---B---) (---A---)
        assert_eq!(
            Interval::open(20, 30).overlapped(Interval::open(0, 10)),
            Interval::Empty
        );

        // (---A---)
        //         [---B---]
        assert_eq!(
            Interval::open(0, 10).overlapped(Interval::closed(10, 20)),
            Interval::Empty
        )
    }

    #[test]
    fn test_finite_interval_overlapped_fully() {
        // (---A---)
        //   (-B-)
        assert_eq!(
            Interval::open(0, 30).overlapped(Interval::open(10, 20)),
            Interval::open(10, 20)
        );

        //   (-A-)
        // (---B---)
        assert_eq!(
            Interval::open(10, 20).overlapped(Interval::open(0, 30)),
            Interval::open(10, 20)
        );

        //   [-A-]
        // (---B---)
        assert_eq!(
            Interval::closed(10, 20).overlapped(Interval::open(0, 30)),
            Interval::closed(10, 20)
        );

        // (---A---)
        // [---B---]
        assert_eq!(
            Interval::open(0, 10).overlapped(Interval::closed(0, 10)),
            Interval::open(0, 10)
        )
    }

    #[test]
    fn test_finite_interval_overlapped() {
        // |---A---|
        //     |---B---|
        assert_eq!(
            Interval::open(0, 100).overlapped(Interval::open(50, 150)),
            Interval::open(50, 100)
        );

        //     |---A---|
        // |---B---|
        assert_eq!(
            Interval::open(50, 150).overlapped(Interval::open(0, 100)),
            Interval::open(50, 100)
        );

        // [---A---]
        //     (---B---)
        assert_eq!(
            Interval::closed(0, 10).overlapped(Interval::open(5, 15)),
            Interval::openclosed(5, 10)
        );

        // (---A---)
        //     [---B---]
        assert_eq!(
            Interval::open(0, 10).overlapped(Interval::closed(5, 15)),
            Interval::closedopen(5, 10)
        );
    }
}

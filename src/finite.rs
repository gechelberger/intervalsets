//mod iset;

use std::ops::{Add, Div, Mul, Sub};

use num::{PrimInt, Zero};

use crate::{ival::*, normalize::Normalize};

/// (a, a) = (a, a] = [a, a) = Empty { x not in T }
/// [a, a] = NonZero { x in T |    x = a    }
/// (a, b) = NonZero { x in T | a <  x <  b }
/// (a, b] = NonZero { x in T | a <  x <= b }
/// [a, b) = NonZero { x in T | a <= x <  b }
/// [a, b] = NonZero { x in T | a <= x <= b }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FiniteInterval<T> {
    Empty,
    NonZero(IVal<T>, IVal<T>),
}

impl<T> FiniteInterval<T>
where
    T: Copy + Ord + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Zero,
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
            Self::Empty => T::zero(),
            Self::NonZero(left, right) => right.value - left.value,
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => {
                left.contains(Side::Left, value) && right.contains(Side::Right, value)
            }
        }
    }

    pub fn overlaps(&self, other: &FiniteInterval<T>) -> bool {
        // probably cheaper ways to do it:
        // AL sees BR.V && BL sees AR.V
        self.overlapped(other) != FiniteInterval::Empty
    }

    pub fn overlapped(&self, other: &FiniteInterval<T>) -> FiniteInterval<T> {
        match (self, other) {
            (FiniteInterval::Empty, _) => FiniteInterval::Empty,
            (_, FiniteInterval::Empty) => FiniteInterval::Empty,
            (
                FiniteInterval::NonZero(a_left, a_right),
                FiniteInterval::NonZero(b_left, b_right),
            ) => {
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
            }
        }
    }

    fn map_bounds(&self, func: impl Fn(&IVal<T>, &IVal<T>) -> Self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonZero(left, right) => func(left, right),
        }
    }

    pub fn shifted(&self, offset: T) -> Self {
        self.map_bounds(|left, right| Self::new_unchecked(*left + offset, *right + offset))
    }

    pub fn padded(&self, amount: T) -> Self {
        self.padded_lr(amount, amount)
    }

    pub fn padded_lr(&self, left: T, right: T) -> Self {
        self.map_bounds(|iv_left, iv_right| Self::new_unchecked(*iv_left - left, *iv_right + right))
    }

    pub fn partition(&self, other: &FiniteInterval<T>) -> Vec<FiniteInterval<T>> {
        if !self.overlaps(other) {
            return vec![self.clone(), other.clone()];
        }

        todo!()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_interval_new() {
        let interval: FiniteInterval<usize> = FiniteInterval::open(0, 20);
    }

    #[test]
    fn test_finite_interval_contains() {
        let iv = FiniteInterval::open(-100, 100);
        assert!(iv.contains(&0));
        assert!(iv.contains(&50));
        assert!(!iv.contains(&100));
        assert!(!iv.contains(&1000));

        assert!(iv.contains(&-50));
        assert!(!iv.contains(&-100));
        assert!(!iv.contains(&-1000));
    }

    #[test]
    fn test_finite_interval_overlapped_empty() {
        // (---A---) (---B---)
        assert_eq!(
            FiniteInterval::open(0, 10).overlapped(&FiniteInterval::open(20, 30)),
            FiniteInterval::Empty
        );

        // (---B---) (---A---)
        assert_eq!(
            FiniteInterval::open(20, 30).overlapped(&FiniteInterval::open(0, 10)),
            FiniteInterval::Empty
        );

        // (---A---)
        //         [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).overlapped(&FiniteInterval::closed(10, 20)),
            FiniteInterval::Empty
        )
    }

    #[test]
    fn test_finite_interval_overlapped_fully() {
        // (---A---)
        //   (-B-)
        assert_eq!(
            FiniteInterval::open(0, 30).overlapped(&FiniteInterval::open(10, 20)),
            FiniteInterval::open(10, 20)
        );

        //   (-A-)
        // (---B---)
        assert_eq!(
            FiniteInterval::open(10, 20).overlapped(&FiniteInterval::open(0, 30)),
            FiniteInterval::open(10, 20)
        );

        //   [-A-]
        // (---B---)
        assert_eq!(
            FiniteInterval::closed(10, 20).overlapped(&FiniteInterval::open(0, 30)),
            FiniteInterval::closed(10, 20)
        );

        // (---A---)
        // [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).overlapped(&FiniteInterval::closed(0, 10)),
            FiniteInterval::open(0, 10)
        )
    }

    #[test]
    fn test_finite_interval_overlapped() {
        // |---A---|
        //     |---B---|
        assert_eq!(
            FiniteInterval::open(0, 100).overlapped(&FiniteInterval::open(50, 150)),
            FiniteInterval::open(50, 100)
        );

        //     |---A---|
        // |---B---|
        assert_eq!(
            FiniteInterval::open(50, 150).overlapped(&FiniteInterval::open(0, 100)),
            FiniteInterval::open(50, 100)
        );

        // [---A---]
        //     (---B---)
        assert_eq!(
            FiniteInterval::closed(0, 10).overlapped(&FiniteInterval::open(5, 15)),
            FiniteInterval::openclosed(5, 10)
        );

        // (---A---)
        //     [---B---]
        assert_eq!(
            FiniteInterval::open(0, 10).overlapped(&FiniteInterval::closed(5, 15)),
            FiniteInterval::closedopen(5, 10)
        );
    }

    #[test]
    fn test_shifted() {
        assert_eq!(
            FiniteInterval::open(0, 10).shifted(10),
            FiniteInterval::open(10, 20)
        );
    }

    #[test]
    fn test_padded() {
        assert_eq!(
            FiniteInterval::open(10, 20).padded(10),
            FiniteInterval::open(0, 30)
        );
    }
}

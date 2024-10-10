

use std::ops::{Add, Div, Mul, Sub};
use std::marker::ConstParamTy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound { Open, Closed }

#[derive(Debug, Clone, Copy, PartialEq, Eq, ConstParamTy)]
pub enum Side { Left, Right }

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IVal<T>{
    bound: Bound,
    value: T,
}

impl<T> IVal<T> {
    fn new(bound: Bound, value: T) -> Self {
        IVal { bound, value }
    }
}

impl<T: Ord> IVal<T> {

    fn contains(&self, side: Side, value: T) -> bool {
        match side {
            Side::Left => match self.bound {
                Bound::Open   => self.value < value,
                Bound::Closed => self.value <= value,
            },
            Side::Right => match self.bound {
                Bound::Open => value < self.value,
                Bound::Closed => value <= self.value,
            }
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HalfInterval<T> {
    side: Side,
    ival: IVal<T>,
}

/// (a, ->) = Left  { x in T | a <  x      }
/// [a, ->) = Left  { x in T | a <= x      }
/// (<-, b) = Right { x in T |      x < b  }
/// (<-, b] = Right { x in T |      x <= b }
impl<T: Ord> HalfInterval<T> {
    fn new(side: Side, ival: IVal<T>) -> Self {
        Self { side, ival }
    }

    fn contains(&self, value: T) -> bool {
        self.ival.contains(self.side, value)
    }

}


/// (a, a) = (a, a] = [a, a) = Empty { x not in T }
/// [a, a] = NonZero { x in T |    x = a    }
/// (a, b) = NonZero { x in T | a <  x <  b }
/// (a, b] = NonZero { x in T | a <  x <= b }
/// [a, b) = NonZero { x in T | a <= x <  b }
/// [a, b] = NonZero { x in T | a <= x <= b }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<T> {
    Empty,
    NonZero(IVal<T>, IVal<T>)
}

impl<T: Copy + Ord> Interval<T> {

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
        Self::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Open, right),
        )
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
                let new_left = if a_left.contains(Side::Left, b_left.value) { b_left } else {*a_left};
                let new_right = if a_right.contains(Side::Right, b_right.value) { b_right } else {*a_right};

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
        
    }

    #[test]
    fn test_finite_interval_contains() {
        let iv = Interval::open(-100,  100);
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


/*
impl<T: Sub<T, Output=T>> Interval<T> {

    pub fn size(&self) -> T {
        match self {
            Self::Empty => 0,
            Self::NonZero(left, right) => right.value - left.value
        }
    }

}

/// Finite: (See Interval)
/// (<-, ->) = Infinite     { x in T }
pub enum IntervalExt<T> {
    Finite(Interval<T>),
    LeftInfinite(IVal<T>),
    RightInfinite(IVal<T>), 
    Infinite
}

impl<T> IVal<T> {

    fn map(self, func: impl Fn(T) -> T) -> Self {
        match self {
            Self::Unbounded => Self::Unbounded,
            Self::Finite(bound, value) => Self::Finite(bound, func(value)),
        }
    }

    fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        match self {
            Self::Unbounded => Self::Unbounded,
            Self::Finite(bound, lhs) => Self::Finite(bound, func(lhs, rhs))
        }
    }
}

impl<T> Add<T> for IVal<T> {
    type Output = IVal<T>;

    fn add(self, rhs: T) -> Self::Output {
        self.binary_map(T::add, rhs)
    }
}

impl<T> Sub<T> for IVal<T> {
    type Output = IVal<T>;

    fn sub(self, rhs: T) -> Self::Output {
        self.binary_map(T::sub, rhs)
    }
}

impl<T> Mul<T> for IVal<T> {
    type Output = IVal<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.binary_map(T::mul, rhs)
    }
}

impl<T> Div<T> for IVal<T> {
    type Output = IVal<T>;

    fn div(self, rhs: T) -> Self::Output {
        self.binary_map(T::div, rhs)
    }
}

impl<T> IVal<T> {
    fn flip(self) -> Self {
        match self {
            Self::Finite(bound, value) => Self::Finite(bound.flip(), value),
            Self::Unbounded => Self::Unbounded
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval<T>(IVal<T>, IVal<T>);

#[derive(Debug, thiserror::Error)]
pub enum IError {

    #[error("")]
    BoundsError
}

type IResult<T> = Result<T, IError>;





impl<T: Copy + Ord + Add<Output = T> + Sub<Output = T> + Div<Output = T> + From<usize>>
    Interval<T>
{

    pub fn shifted(&self, offset: T) -> Self {
        Self {
            range: (self.lower() + offset, self.upper() + offset),
            bounds: self.bounds,
        }
    }

    pub fn padded(&self, amount: T) -> Self {
        self.padded2(amount, amount)
    }

    pub fn padded2(&self, pad_lower: T, pad_upper: T) -> Self {
        let (lower, upper) = self.range;
        Self {
            range: (lower - pad_lower, upper + pad_upper),
            ..*self
        }
    }

    pub fn center(&self) -> T {
        self.lower() + self.size() / T::from(2)
    }


    fn overlaps(&self, other: Self) -> bool {
        let (a_lower, a_upper) = self.range;
        let (b_lower, b_upper) = other.range;
        if b_upper < a_lower || b_lower > a_upper {
            return false;
        }

        todo!()
    }

    fn overlap(&self, other: Self) -> Option<Self> {
        todo!()
    }
}


*/
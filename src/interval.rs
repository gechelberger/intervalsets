use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound { Open, Closed }

impl Bound {
    pub fn flip(self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IVal<T> {
    Finite(Bound, T),
    Unbounded
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


impl<T: Copy + Ord> Interval<T> {

    pub fn new(left: IVal<T>, right: IVal<T>) -> IResult<Self> {
        match (left, right) {
            (IVal::Unbounded, _) => Ok(Self(left, right)),
            (_, IVal::Unbounded) => Ok(Self(left, right)),
            (IVal::Finite(_, lval), IVal::Finite(_, rval)) => {
                if lval <= rval {
                    Ok(Self(left, right))
                } else {
                    Err(IError::BoundsError)
                }
            }
        }
    }

    pub fn new_unchecked(left: IVal<T>, right: IVal<T>) -> Self {
        Self(left, right)
    }

    pub fn open(left: T, right: T) -> IResult<Self> {
        Self::new(
            IVal::Finite(Bound::Open, left),
            IVal::Finite(Bound::Open, right),
        )
    }

    pub fn closed(left: T, right: T) -> IResult<Self> {
        Self::new(
            IVal::Finite(Bound::Closed, left),
            IVal::Finite(Bound::Closed, right),
        )
    }

    pub fn openclosed(left: T, right: T) -> IResult<Self> {
        Self::new(
            IVal::Finite(Bound::Open, left),
            IVal::Finite(Bound::Closed, right),
        )
    }

    pub fn closedopen(left: T, right: T) -> IResult<Self> {
        Self::new(
            IVal::Finite(Bound::Closed, left),
            IVal::Finite(Bound::Open, right),
        )
    }

    pub fn left(&self) -> IVal<T> {
        self.0
    }

    pub fn lbound(&self) -> Bound {
        match self.left() {
            IVal::Finite(bound, _) => bound,
            IVal::Unbounded => panic!("Can not get an unbounded bound")
        }
    }

    pub fn lval(&self) -> T {
        match self.left() {
            IVal::Finite(_, value) => value,
            IVal::Unbounded => panic!("Can not get an unbounded value")
        }
    }

    pub fn right(&self) -> IVal<T> {
        self.0
    }

    pub fn rbound(&self) -> Bound {
        match self.right() {
            IVal::Finite(bound, _) => bound,
            IVal::Unbounded => panic!("Can not get an unbounded bound")
        }
    }

    pub fn rval(&self) -> T {
        match self.right() {
            IVal::Finite(_, value) => value,
            IVal::Unbounded => panic!("Can not get an unbounded value")
        }
    }

}



impl<T: Copy + Ord + Add<Output = T> + Sub<Output = T> + Div<Output = T> + From<usize>>
    Interval<T>
{

    pub fn shifted(&self, offset: T) -> Self {
        Self {
            range: (self.lower() + offset, self.upper() + offset),
            bounds: self.bounds,
        }
    }

    pub fn contains(&self, value: T) -> bool {
        let (lower, upper) = self.range;
        if value < lower || value > upper {
            return false;
        }

        let (lbound, ubound) = self.bounds;

        let check_lower = match lbound {
            Bound::Closed => (value >= lower),
            Bound::Open => (value > lower),
        };

        let check_upper = match ubound {
            Bound::Closed => (value <= upper),
            Bound::Open => (value < upper),
        };

        check_lower && check_upper
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

    pub fn size(&self) -> T {
        let (lower, upper) = self.range;
        upper - lower
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

use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Open,
    Closed,
}

impl Bound {
    pub fn flip(self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn flip(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IVal<T> {
    pub(crate) bound: Bound,
    pub(crate) value: T,
}

impl<T: Copy> IVal<T> {
    pub fn new(bound: Bound, value: T) -> Self {
        IVal { bound, value }
    }

    pub fn get_bound(&self) -> Bound {
        self.bound
    }

    pub fn get_value(&self) -> T {
        self.value
    }

    pub fn flip(&self) -> Self {
        Self::new(self.bound.flip(), self.value)
    }

    fn map(self, func: impl Fn(T) -> T) -> Self {
        Self::new(self.bound, func(self.value))
    }

    fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        Self::new(self.bound, func(self.value, rhs))
    }
}

impl<T: PartialOrd> IVal<T> {
    pub fn contains(&self, side: Side, value: &T) -> bool {
        match side {
            Side::Left => match self.bound {
                Bound::Open => self.value < *value,
                Bound::Closed => self.value <= *value,
            },
            Side::Right => match self.bound {
                Bound::Open => *value < self.value,
                Bound::Closed => *value <= self.value,
            },
        }
    }
}

impl<T> Add<T> for IVal<T>
where
    T: Copy + Add<T, Output = T>,
{
    type Output = IVal<T>;

    fn add(self, rhs: T) -> Self::Output {
        self.binary_map(T::add, rhs)
    }
}

impl<T> Sub<T> for IVal<T>
where
    T: Copy + Sub<T, Output = T>,
{
    type Output = IVal<T>;

    fn sub(self, rhs: T) -> Self::Output {
        self.binary_map(T::sub, rhs)
    }
}

impl<T> Mul<T> for IVal<T>
where
    T: Copy + Mul<T, Output = T>,
{
    type Output = IVal<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.binary_map(T::mul, rhs)
    }
}

impl<T> Div<T> for IVal<T>
where
    T: Copy + Div<T, Output = T>,
{
    type Output = IVal<T>;

    fn div(self, rhs: T) -> Self::Output {
        self.binary_map(T::div, rhs)
    }
}
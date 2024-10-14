use std::ops::{Add, Div, Mul, Sub};

use crate::numeric::Numeric;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IVal<T> {
    pub(crate) bound: Bound,
    pub(crate) value: T,
}

impl<T> IVal<T> {
    pub fn new(bound: Bound, value: T) -> Self {
        IVal { bound, value }
    }

    pub fn closed(value: T) -> Self {
        Self::new(Bound::Closed, value)
    }

    pub fn open(value: T) -> Self {
        Self::new(Bound::Open, value)
    }

    pub fn into_raw(self) -> (Bound, T) {
        (self.bound, self.value)
    }

    pub fn get_bound(&self) -> Bound {
        self.bound
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }
}

impl<T: Numeric> IVal<T> {
    pub fn flip(&self) -> Self {
        Self::new(self.bound.flip(), self.value.clone())
    }

    pub fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        Self::new(self.bound, func(self.value, rhs))
    }

    pub fn min_left(a: &IVal<T>, b: &IVal<T>) -> IVal<T> {
        if a.contains(Side::Left, &b.value) {
            a.clone()
        } else {
            b.clone()
        }
    }

    pub fn min_right(a: &IVal<T>, b: &IVal<T>) -> IVal<T> {
        if a.contains(Side::Right, &b.value) {
            b.clone()
        } else {
            a.clone()
        }
    }

    pub fn max_left(a: &IVal<T>, b: &IVal<T>) -> IVal<T> {
        if a.contains(Side::Left, &b.value) {
            b.clone()
        } else {
            a.clone()
        }
    }

    pub fn max_right(a: &IVal<T>, b: &IVal<T>) -> IVal<T> {
        if a.contains(Side::Right, &b.value) {
            a.clone()
        } else {
            b.clone()
        }
    }

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

    pub fn normalized(self, side: Side) -> Self {
        if !T::numeric_set().in_integer() {
            return self;
        }

        match self.bound {
            Bound::Open => match side {
                Side::Left => match self.value.try_finite_add(&T::one()) {
                    Some(limit) => Self::new(Bound::Closed, limit),
                    None => self,
                },
                Side::Right => match self.value.try_finite_sub(&T::one()) {
                    Some(limit) => Self::new(Bound::Closed, limit),
                    None => self,
                },
            },
            Bound::Closed => self,
        }
    }
}

impl<T: Numeric> Add<T> for IVal<T> {
    type Output = IVal<T>;

    fn add(self, rhs: T) -> Self::Output {
        self.binary_map(T::add, rhs)
    }
}

impl<T: Numeric> Sub<T> for IVal<T> {
    type Output = IVal<T>;

    fn sub(self, rhs: T) -> Self::Output {
        self.binary_map(T::sub, rhs)
    }
}

impl<T: Numeric> Mul<T> for IVal<T> {
    type Output = IVal<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.binary_map(T::mul, rhs)
    }
}

impl<T: Numeric + Div<Output = T>> Div<T> for IVal<T> {
    type Output = IVal<T>;

    fn div(self, rhs: T) -> Self::Output {
        self.binary_map(T::div, rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}

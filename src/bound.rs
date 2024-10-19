use std::ops::{Add, Sub};

use crate::numeric::Domain;

/// Side( Left | Right ) on the number line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BoundType {
    Closed,
    Open,
}

impl BoundType {
    pub fn flip(self) -> Self {
        match self {
            Self::Closed => Self::Open,
            Self::Open => Self::Closed,
        }
    }
}

/// Defines the `Bound` or limit that constrains a Set.
///
/// An Open(limit) does not include limit as an element of the set,
/// while a Closed(limit) does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound<T>(BoundType, T);

impl<T: Clone + PartialOrd> Bound<T> {
    pub fn min_left(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Left, b.value()) {
            a.clone()
        } else {
            b.clone()
        }
    }

    pub fn min_right(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Right, b.value()) {
            b.clone()
        } else {
            a.clone()
        }
    }

    pub fn max_left(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Left, b.value()) {
            b.clone()
        } else {
            a.clone()
        }
    }

    pub fn max_right(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Right, b.value()) {
            a.clone()
        } else {
            b.clone()
        }
    }

    pub fn flip(&self) -> Self {
        Self(self.0.flip(), self.1.clone())
    }
}

impl<T> Bound<T> {
    pub fn closed(limit: T) -> Self {
        Self(BoundType::Closed, limit)
    }

    pub fn open(limit: T) -> Self {
        Self(BoundType::Open, limit)
    }

    pub fn map<U>(&self, func: impl FnOnce(&T) -> U) -> Bound<U> {
        Bound::<U>(self.0, func(self.value()))
    }

    pub fn is_open(&self) -> bool {
        matches!(self.0, BoundType::Open)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.0, BoundType::Closed)
    }

    pub fn bound_type(&self) -> BoundType {
        self.0
    }

    pub fn value(&self) -> &T {
        &self.1
    }

    pub fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        Self(self.0, func(self.1, rhs))
    }
}

impl<T: PartialOrd> Bound<T> {
    pub fn contains(&self, side: Side, value: &T) -> bool {
        match side {
            Side::Left => match self.0 {
                BoundType::Open => self.value() < value,
                BoundType::Closed => self.value() <= value,
            },
            Side::Right => match self.0 {
                BoundType::Open => value < self.value(),
                BoundType::Closed => value <= self.value(),
            },
        }
    }
}

impl<T: Domain> Bound<T> {
    pub fn normalized(self, side: Side) -> Self {
        match self.0 {
            BoundType::Open => match self.value().try_adjacent(side.flip()) {
                None => self,
                Some(new_limit) => Self::closed(new_limit),
            },
            BoundType::Closed => self,
        }
    }
}

impl<T: Domain + core::ops::Add<T, Output = T>> Add<T> for Bound<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        self.binary_map(T::add, rhs)
    }
}

impl<T: Domain + core::ops::Sub<T, Output = T>> Sub<T> for Bound<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        self.binary_map(T::sub, rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}

use std::ops::{Add, Sub};

use crate::numeric::Domain;

/*
pub trait MaybeBounded<T> {
    fn bound(&self, side: Side) -> Option<Bound<&T>>;

    fn left(&self) -> Option<Bound<&T>> {
        self.bound(Side::Left)
    }

    fn right(&self) -> Option<Bound<&T>> {
        self.bound(Side::Right)
    }

    fn lval(&self) -> Option<&T> {
        self.left().map(|v| v.value())
    }
}*/

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
pub enum Bound<T> {
    Open(T),
    Closed(T),
}

impl<T: Domain> Bound<T> {
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

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open(_))
    }

    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

    pub fn value(&self) -> &T {
        match self {
            Self::Open(limit) => limit,
            Self::Closed(limit) => limit,
        }
    }

    pub fn flip(&self) -> Self {
        match self {
            Self::Open(limit) => Self::Closed(limit.clone()),
            Self::Closed(limit) => Self::Open(limit.clone()),
        }
    }

    pub fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        match self {
            Self::Closed(limit) => Self::Closed(func(limit, rhs)),
            Self::Open(limit) => Self::Open(func(limit, rhs)),
        }
    }

    pub fn contains(&self, side: Side, value: &T) -> bool {
        match side {
            Side::Left => match self {
                Self::Open(limit) => limit < value,
                Self::Closed(limit) => limit <= value,
            },
            Side::Right => match self {
                Self::Open(limit) => value < limit,
                Self::Closed(limit) => value <= limit,
            },
        }
    }

    pub fn normalized(self, side: Side) -> Self {
        match self {
            Self::Open(ref limit) => match limit.try_adjacent(side.flip()) {
                None => self,
                Some(new_limit) => Self::Closed(new_limit),
            },
            Self::Closed(_) => self,
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

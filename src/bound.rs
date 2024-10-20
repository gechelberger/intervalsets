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

/// The BoundType determines the inclusivity of the limit element in a set.
///
/// `Closed` bounds include the limit value in the `Set`, `Open` bounds do not.
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
    /// Returns the smaller `Bound`, treating args as left hand (lower) bounds.
    pub fn min_left(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Left, b.value()) {
            a.clone()
        } else {
            b.clone()
        }
    }

    /// Returns the smaller `Bound`, treating args as right hand (upper) bounds.
    pub fn min_right(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Right, b.value()) {
            b.clone()
        } else {
            a.clone()
        }
    }

    /// Returns the larger `Bound`, treating args as left hand (lower) bounds.
    pub fn max_left(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Left, b.value()) {
            b.clone()
        } else {
            a.clone()
        }
    }

    /// Returns the larger `Bound`, treating args as right hand (upper) bounds.
    pub fn max_right(a: &Self, b: &Self) -> Self {
        if a.contains(Side::Right, b.value()) {
            a.clone()
        } else {
            b.clone()
        }
    }

    /// Return a new `Bound` with the same limit, but flipped `BoundType`.
    pub fn flip(&self) -> Self {
        Self(self.0.flip(), self.1.clone())
    }
}

impl<T> Bound<T> {
    /// Returns a new closed `Bound` constrained at `limit`.
    pub fn closed(limit: T) -> Self {
        Self(BoundType::Closed, limit)
    }

    /// Returns a new open `Bound` constrained at `limit`.
    pub fn open(limit: T) -> Self {
        Self(BoundType::Open, limit)
    }

    /// Returns a new `Bound`, retaining BoundType, with a new limit.
    ///
    /// # Examples
    /// ```
    /// use intervalsets::Bound;
    /// let bound = Bound::closed(10);
    /// let shift = bound.map(|limit| limit + 10);
    /// assert_eq!(shift, Bound::closed(20));
    ///
    /// let float = bound.map(|limit| *limit as f32);
    /// assert_eq!(float, Bound::closed(10.0));
    /// ```
    pub fn map<U>(&self, func: impl FnOnce(&T) -> U) -> Bound<U> {
        Bound::<U>(self.0, func(self.value()))
    }

    /// Returns `true` if this bound's `BoundType` is `Open`.
    pub fn is_open(&self) -> bool {
        matches!(self.0, BoundType::Open)
    }

    /// Returns `true` if this bound's `BoundType` is `Closed`
    pub fn is_closed(&self) -> bool {
        matches!(self.0, BoundType::Closed)
    }

    /// Return the `BoundType` of this `Bound`.
    pub fn bound_type(&self) -> BoundType {
        self.0
    }

    /// Returns a reference to this bound's limit value.
    pub fn value(&self) -> &T {
        &self.1
    }

    /// Map a binary operation over `Bound<T>`. `self` is consumed.
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

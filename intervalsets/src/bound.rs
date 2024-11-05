use std::borrow::Cow::{self, *};

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

    pub fn select<T>(self, left: T, right: T) -> T {
        match self {
            Self::Left => left,
            Self::Right => right,
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

impl<T: PartialOrd + Clone> Bound<T> {
    pub fn min_cow<'a>(
        side: Side,
        a: Cow<'a, Bound<T>>,
        b: Cow<'a, Bound<T>>,
    ) -> Cow<'a, Bound<T>> {
        if a.contains(side, b.value()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    pub fn max_cow<'a>(
        side: Side,
        a: Cow<'a, Bound<T>>,
        b: Cow<'a, Bound<T>>,
    ) -> Cow<'a, Bound<T>> {
        if a.contains(side, b.value()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }

    pub fn take_min(side: Side, a: Bound<T>, b: Bound<T>) -> Bound<T> {
        Self::min_cow(side, Owned(a), Owned(b)).into_owned()
    }

    pub fn take_max(side: Side, a: Bound<T>, b: Bound<T>) -> Bound<T> {
        Self::max_cow(side, Owned(a), Owned(b)).into_owned()
    }
}

impl<T: Clone + PartialOrd> Bound<T> {
    /// Returns the smaller `Bound`, treating args as left hand (lower) bounds.
    pub fn min_left(a: &Self, b: &Self) -> Self {
        Self::min_cow(Side::Left, Borrowed(a), Borrowed(b)).into_owned()
    }

    /// Returns the smaller `Bound`, treating args as right hand (upper) bounds.
    pub fn min_right(a: &Self, b: &Self) -> Self {
        Self::min_cow(Side::Right, Borrowed(a), Borrowed(b)).into_owned()
    }

    /// Returns the larger `Bound`, treating args as left hand (lower) bounds.
    pub fn max_left(a: &Self, b: &Self) -> Self {
        Self::max_cow(Side::Left, Borrowed(a), Borrowed(b)).into_owned()
    }

    /// Returns the larger `Bound`, treating args as right hand (upper) bounds.
    pub fn max_right(a: &Self, b: &Self) -> Self {
        Self::max_cow(Side::Right, Borrowed(a), Borrowed(b)).into_owned()
    }

    /// Return a new `Bound` keeps limit, flips `BoundType`. `self` is consumed.
    pub fn flip(self) -> Self {
        Self(self.0.flip(), self.1)
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

    /// Returns a new `Bound`, keeps BoundType, new limit; `self` is consumed.
    ///
    /// # Examples
    /// ```
    /// use intervalsets::Bound;
    /// let bound = Bound::closed(10);
    /// let shift = bound.clone().map(|limit| limit + 10);
    /// assert_eq!(shift, Bound::closed(20));
    ///
    /// let float = bound.map(|limit| limit as f32);
    /// assert_eq!(float, Bound::closed(10.0));
    /// ```
    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> Bound<U> {
        Bound::<U>(self.0, func(self.1))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bound_min_max() {
        assert_eq!(
            Bound::min_left(&Bound::closed(0), &Bound::closed(10)).clone(),
            //Bound::min_left(&Bound::closed(0), &Bound::closed(10))
            Bound::closed(0)
        );

        assert_eq!(
            Bound::min_left(&Bound::closed(0), &Bound::open(0)).clone(),
            //Bound::min_left(&Bound::closed(0), &Bound::open(0)),
            Bound::closed(0)
        );

        assert_eq!(
            Bound::max_left(&Bound::closed(0), &Bound::closed(10)).clone(),
            //Bound::max_left(&Bound::closed(0), &Bound::closed(10)),
            Bound::closed(10)
        );

        assert_eq!(
            Bound::max_left(&Bound::closed(0), &Bound::open(0)).clone(),
            Bound::open(0)
        );

        assert_eq!(
            Bound::min_right(&Bound::closed(0), &Bound::closed(10)).clone(),
            Bound::closed(0)
        );

        assert_eq!(
            Bound::min_right(&Bound::closed(0), &Bound::open(0)).clone(),
            Bound::open(0)
        );

        assert_eq!(
            Bound::max_right(&Bound::closed(0), &Bound::closed(10)).clone(),
            Bound::closed(10)
        );

        assert_eq!(
            Bound::max_right(&Bound::closed(0), &Bound::open(0)).clone(),
            Bound::closed(0)
        )
    }
}

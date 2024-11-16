use crate::numeric::Domain;

//type Extremum<T> = Option<FiniteBound<T>>;
//type Envelope<T> = (Extremum<T>, Extremum<T>);

/// todo...
pub trait SetBounds<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>>;

    //fn into_bounds(self) -> Option<Envelope<T>>;

    #[inline]
    fn left(&self) -> Option<&FiniteBound<T>> {
        self.bound(Side::Left)
    }

    #[inline]
    fn right(&self) -> Option<&FiniteBound<T>> {
        self.bound(Side::Right)
    }

    #[inline]
    fn lval(&self) -> Option<&T> {
        self.left().map(|x| x.value())
    }

    #[inline]
    fn rval(&self) -> Option<&T> {
        self.right().map(|x| x.value())
    }
}

/// Side( Left | Right ) on the number line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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

/// The BoundType determines the inclusivity of the constraining element in a set.
///
/// `Closed` bounds include the limit value in the `Set`, `Open` bounds do not.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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
///
/// # Note
///
/// No ordering implementation is provided because the correct order is
/// a function of this bound **and** which side of the interval it constrains.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct FiniteBound<T>(BoundType, T);

impl<T> FiniteBound<T> {
    pub fn new(bound_type: BoundType, limit: T) -> Self {
        Self(bound_type, limit)
    }

    /// Returns a new closed `Bound` constrained at `limit`.
    pub fn closed(limit: T) -> Self {
        Self(BoundType::Closed, limit)
    }

    /// Returns a new open `Bound` constrained at `limit`.
    pub fn open(limit: T) -> Self {
        Self(BoundType::Open, limit)
    }

    pub fn into_raw(self) -> (BoundType, T) {
        (self.0, self.1)
    }

    pub fn ord(&self, side: Side) -> ord::OrdBound<&T> {
        match self.bound_type() {
            BoundType::Closed => ord::OrdBound::Finite(self.value(), ord::OrdBoundFinite::Closed),
            BoundType::Open => ord::OrdBound::Finite(self.value(), ord::OrdBoundFinite::open(side)),
        }
    }

    pub fn into_ord(self, side: Side) -> ord::OrdBound<T> {
        let (bound_type, value) = self.into_raw();
        match bound_type {
            BoundType::Closed => ord::OrdBound::Finite(value, ord::OrdBoundFinite::Closed),
            BoundType::Open => ord::OrdBound::Finite(value, ord::OrdBoundFinite::open(side)),
        }
    }

    /// Returns a new `Bound`, keeps BoundType, new limit; `self` is consumed.
    ///
    /// # Examples
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let bound = FiniteBound::closed(10);
    /// let shift = bound.clone().map(|limit| limit + 10);
    /// assert_eq!(shift, FiniteBound::closed(20));
    ///
    /// let float = bound.map(|limit| limit as f32);
    /// assert_eq!(float, FiniteBound::closed(10.0));
    /// ```
    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> FiniteBound<U> {
        FiniteBound::<U>(self.0, func(self.1))
    }

    /// Return a new `Bound` keeps limit, flips `BoundType`. `self` is consumed.
    pub fn flip(self) -> Self {
        Self(self.0.flip(), self.1)
    }

    /// Returns `true` if this bound's `BoundType` is `Open`.
    pub fn is_open(&self) -> bool {
        self.0 == BoundType::Open
    }

    /// Returns `true` if this bound's `BoundType` is `Closed`
    pub fn is_closed(&self) -> bool {
        self.0 == BoundType::Closed
    }

    /// Return the `BoundType` of this `Bound`.
    #[inline(always)]
    pub fn bound_type(&self) -> BoundType {
        self.0
    }

    /// Returns a reference to this bound's limit value.
    #[inline(always)]
    pub fn value(&self) -> &T {
        &self.1
    }

    /// Map a binary operation over `Bound<T>`. `self` is consumed.
    pub fn binary_map(self, func: impl Fn(T, T) -> T, rhs: T) -> Self {
        Self(self.0, func(self.1, rhs))
    }
}

impl<T: PartialOrd> FiniteBound<T> {
    pub fn take_min(side: Side, a: FiniteBound<T>, b: FiniteBound<T>) -> FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    pub fn take_max(side: Side, a: FiniteBound<T>, b: FiniteBound<T>) -> FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }

    pub fn min<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    pub fn max<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }
}

impl<T: PartialOrd> FiniteBound<T> {
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

impl<T: Domain> FiniteBound<T> {
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

pub mod ord {
    use core::cmp::Ordering;

    use super::{BoundType, FiniteBound};

    /// todo...
    pub trait OrdBounded<T> {
        fn ord_bound_pair(&self) -> OrdBoundPair<&T>;
    }

    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
    )]
    pub enum OrdBound<T> {
        LeftUnbounded,
        Finite(T, OrdBoundFinite),
        RightUnbounded,
    }

    impl<T> OrdBound<T> {
        pub fn left_open(limit: T) -> Self {
            Self::Finite(limit, OrdBoundFinite::LeftOpen)
        }

        pub fn closed(limit: T) -> Self {
            Self::Finite(limit, OrdBoundFinite::Closed)
        }

        pub fn right_open(limit: T) -> Self {
            Self::Finite(limit, OrdBoundFinite::RightOpen)
        }
    }

    impl<T: PartialOrd> OrdBound<T> {
        pub fn partial_min(self, rhs: Self) -> Option<Self> {
            match self.partial_cmp(&rhs)? {
                Ordering::Less => Some(self),
                Ordering::Greater => Some(rhs),
                Ordering::Equal => Some(self),
            }
        }

        pub fn partial_max(self, rhs: Self) -> Option<Self> {
            match self.partial_cmp(&rhs)? {
                Ordering::Less => Some(rhs),
                Ordering::Greater => Some(self),
                Ordering::Equal => Some(self),
            }
        }
    }

    impl<'a, T> OrdBound<&'a T> {
        pub fn left(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::Finite(bound.value(), Closed),
                BoundType::Open => Self::Finite(bound.value(), LeftOpen),
            }
        }

        pub fn right(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::Finite(bound.value(), Closed),
                BoundType::Open => Self::Finite(bound.value(), RightOpen),
            }
        }
    }

    impl<T: Clone> OrdBound<&T> {
        pub fn cloned(self) -> OrdBound<T> {
            match self {
                Finite(value, order) => Finite(value.clone(), order),
                LeftUnbounded => LeftUnbounded,
                RightUnbounded => RightUnbounded,
            }
        }
    }

    impl<T> OrdBound<T> {
        pub fn map<F, U>(self, func: F) -> OrdBound<U>
        where
            F: FnOnce(T) -> U,
        {
            match self {
                Finite(value, case) => OrdBound::Finite(func(value), case),
                LeftUnbounded => LeftUnbounded,
                RightUnbounded => RightUnbounded,
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
    )]
    pub enum OrdBoundFinite {
        RightOpen,
        Closed,
        LeftOpen,
    }

    use OrdBound::*;
    use OrdBoundFinite::*;

    impl OrdBoundFinite {
        pub fn open(side: super::Side) -> Self {
            side.select(LeftOpen, RightOpen)
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
    )]
    pub struct OrdBoundPair<T>(OrdBound<T>, OrdBound<T>);

    impl<T: PartialEq> OrdBoundPair<T> {
        pub fn is_empty(&self) -> bool {
            *self == Self(LeftUnbounded, LeftUnbounded)
        }
    }

    impl<T> OrdBoundPair<T> {
        pub fn empty() -> Self {
            Self(LeftUnbounded, LeftUnbounded)
        }

        pub fn new(left: OrdBound<T>, right: OrdBound<T>) -> Self {
            match (left, right) {
                // use (LU, LU) to represent EMPTY and make it the lowest element
                (LeftUnbounded, LeftUnbounded) => Self::empty(),
                (left, right) => {
                    debug_assert!(!matches!(&left, RightUnbounded));
                    debug_assert!(!matches!(&left, Finite(_, RightOpen)));
                    debug_assert!(!matches!(&right, LeftUnbounded));
                    debug_assert!(!matches!(&right, Finite(_, LeftOpen)));
                    Self(left, right)
                }
            }
        }

        pub fn into_raw(self) -> (OrdBound<T>, OrdBound<T>) {
            (self.0, self.1)
        }
    }
}
#[cfg(test)]
mod test {
    use ord::OrdBound;

    use super::*;

    #[test]
    fn test_bound_min_max() {
        assert_eq!(
            FiniteBound::min(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::min(Side::Left, &FiniteBound::closed(0), &FiniteBound::open(0)),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::max(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(10)
        );

        assert_eq!(
            FiniteBound::max(Side::Left, &FiniteBound::closed(0), &FiniteBound::open(0)),
            &FiniteBound::open(0)
        );

        assert_eq!(
            FiniteBound::min(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::min(Side::Right, &FiniteBound::closed(0), &FiniteBound::open(0)),
            &FiniteBound::open(0)
        );

        assert_eq!(
            FiniteBound::max(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(10)
        );

        assert_eq!(
            FiniteBound::max(Side::Right, &FiniteBound::closed(0), &FiniteBound::open(0)),
            &FiniteBound::closed(0)
        )
    }

    #[test]
    fn test_partial_min_max() {
        let f0 = 0.0;
        let f1 = 100.0;

        assert_eq!(
            OrdBound::closed(f0).partial_min(OrdBound::closed(f1)),
            Some(OrdBound::closed(f0))
        );

        assert_eq!(
            OrdBound::closed(&f0).partial_min(OrdBound::closed(&f1)),
            Some(OrdBound::closed(&f0))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.partial_max(OrdBound::closed(f1)),
            Some(OrdBound::closed(f1))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.partial_max(OrdBound::closed(&f1)),
            Some(OrdBound::closed(&f1))
        )
    }
}

//! Bounds partition elements inside and outside of a `Set`.
//!
//! A finite `Set` bound requires three pieces of information:
//! * The finite limiting value
//! * The [`BoundType`]: whether the limit itself is an element of the `Set`
//! * Which [`Side`] of the bound contains elements of the `Set`.
//!
//! A [`FiniteBound`] encapsulates the first two pieces of information. The last
//! is encapsulated on a case by case basis depending on the kind of set.
//!
//! All `Set` types should implement the [`SetBounds`], and
//! [`OrdBounded`](ord::OrdBounded) traits.

use crate::numeric::Domain;

/// An interface to query the left and right bounds of a set.
pub trait SetBounds<T> {
    /// Return a reference to the left or right bound if it is finite.
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>>;

    //fn into_bounds(self) -> Option<Envelope<T>>;

    /// Return a reference to the left bound if it is finite.
    #[inline]
    fn left(&self) -> Option<&FiniteBound<T>> {
        self.bound(Side::Left)
    }

    /// Return a reference to the right bound if it is finite.
    #[inline]
    fn right(&self) -> Option<&FiniteBound<T>> {
        self.bound(Side::Right)
    }

    /// Return a reference to the left bound value if it is finite.
    #[inline]
    fn lval(&self) -> Option<&T> {
        self.left().map(|x| x.value())
    }

    /// Return a reference to the right bound value if it is finite.
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
    /// Generally the lower bound
    Left,
    /// Generally the upper bound
    Right,
}

impl Side {
    /// Flip left => right, right => left
    #[inline(always)]
    pub fn flip(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Return the left or right arg depending on the value of self.
    #[inline(always)]
    pub fn select<T>(self, left: T, right: T) -> T {
        match self {
            Self::Left => left,
            Self::Right => right,
        }
    }

    /// Invoke the left or right arg for the value of self and return the result.
    #[inline(always)]
    pub fn fn_select<T>(self, left: impl FnOnce() -> T, right: impl FnOnce() -> T) -> T {
        match self {
            Self::Left => left(),
            Self::Right => right(),
        }
    }
}

/// The BoundType determines the inclusivity of the constraining element in a set.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum BoundType {
    /// A Closed BoundType includes the limit element in the `Set`.
    Closed,
    /// An Open BoundType excludes the limit element from the `Set`.
    Open,
}

impl BoundType {
    /// Flips the bound type Open => Closed, Closed => Open
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
    /// Creates a new [`FiniteBound`]
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

    /// Unpack a [`FiniteBound`] into ([`BoundType`], `T`)
    pub fn into_raw(self) -> (BoundType, T) {
        (self.0, self.1)
    }

    /// Creates an `OrdBound<&T>`
    pub fn ord(&self, side: Side) -> ord::OrdBound<&T> {
        match self.bound_type() {
            BoundType::Closed => ord::OrdBound::Finite(self.value(), ord::OrdBoundFinite::Closed),
            BoundType::Open => ord::OrdBound::Finite(self.value(), ord::OrdBoundFinite::open(side)),
        }
    }

    /// Turns self into an `OrdBound<T>`
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
    #[inline(always)]
    pub fn flip(self) -> Self {
        Self(self.0.flip(), self.1)
    }

    /// Returns `true` if this bound's `BoundType` is `Open`.
    #[inline(always)]
    pub fn is_open(&self) -> bool {
        self.0 == BoundType::Open
    }

    /// Returns `true` if this bound's `BoundType` is `Closed`
    #[inline(always)]
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
    /// Consume a and b, returning the minimum bound.
    pub fn take_min(side: Side, a: FiniteBound<T>, b: FiniteBound<T>) -> FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    /// Consume a and b, returning the maximum bound.
    pub fn take_max(side: Side, a: FiniteBound<T>, b: FiniteBound<T>) -> FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }

    /// Return a reference to the minimum bound.
    pub fn min<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    /// Return a reference to the maximum bound.
    pub fn max<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        if a.contains(side, b.value()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }
}

impl<T: PartialOrd> FiniteBound<T> {
    /// Test if this partitions an element to be contained by the `Set`.
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
    /// For discrete types, normalize to closed form.
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

/// Helpers that define a total order for `Set` bounds.
pub mod ord {
    use super::{BoundType, FiniteBound};

    /// todo...
    pub trait OrdBounded<T> {
        fn ord_bound_pair(&self) -> OrdBoundPair<&T>;
    }

    /// A type that defines a total order for all possible bounds.
    ///
    /// ```text
    /// In relation to finite bounds:
    /// L(None) < R(Open(x)) < R(Closed(x)) <= L(Closed(x)) < L(Open(x)) < R(None)
    /// LeftUnbound < RightOpen(x) < Closed(x) < LeftOpen(x) < RightUnbound
    /// ```
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

    /// Ordered exclusivity cases for finite bounds.
    ///
    /// For a given finite value x, RightOpen(x) < Closed(x) < LeftOpen(x).
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

    /// An ordered pair of bounds where left <= right.
    ///
    /// The empty set is represented by (-inf, -inf).
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
    use crate::try_cmp::{TryMax, TryMin};

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
            OrdBound::closed(f0).try_min(OrdBound::closed(f1)),
            Some(OrdBound::closed(f0))
        );

        assert_eq!(
            OrdBound::closed(&f0).try_min(OrdBound::closed(&f1)),
            Some(OrdBound::closed(&f0))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed(f1)),
            Some(OrdBound::closed(f1))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed(&f1)),
            Some(OrdBound::closed(&f1))
        )
    }
}

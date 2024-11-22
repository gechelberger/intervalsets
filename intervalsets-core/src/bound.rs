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

use core::cmp::Ordering::{Equal, Greater, Less};

use crate::error::TotalOrderError;
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
    pub const fn flip(self) -> Self {
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
}

/// The BoundType determines the inclusivity of the constraining element in a set.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum BoundType {
    /// An Open BoundType excludes the limit element from the `Set`.
    Open = 0,
    /// A Closed BoundType includes the limit element in the `Set`.
    Closed = 1,
}

impl BoundType {
    /// Flips the bound type Open => Closed, Closed => Open
    pub fn flip(self) -> Self {
        match self {
            Self::Closed => Self::Open,
            Self::Open => Self::Closed,
        }
    }

    pub fn combine(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Closed, Self::Closed) => Self::Closed,
            _ => Self::Open,
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
    pub const fn new(bound_type: BoundType, limit: T) -> Self {
        Self(bound_type, limit)
    }

    /// Creates a new closed `FiniteBound` constrained at `limit`.
    pub const fn closed(limit: T) -> Self {
        Self(BoundType::Closed, limit)
    }

    /// Creates a new open `Bound` constrained at `limit`.
    pub const fn open(limit: T) -> Self {
        Self(BoundType::Open, limit)
    }

    /// Unpack a [`FiniteBound`] into ([`BoundType`], `T`)
    pub fn into_raw(self) -> (BoundType, T) {
        (self.0, self.1)
    }

    pub fn as_ref(&self) -> FiniteBound<&T> {
        FiniteBound::new(self.0, &self.1)
    }

    pub fn finite_ord(&self, side: Side) -> ord::FiniteOrdBound<&T> {
        match self.bound_type() {
            BoundType::Closed => ord::FiniteOrdBound::closed(self.value()),
            BoundType::Open => ord::FiniteOrdBound::open(side, self.value()),
        }
    }

    /// Creates an `OrdBound<&T>`
    pub fn ord(&self, side: Side) -> ord::OrdBound<&T> {
        ord::OrdBound::Finite(self.finite_ord(side))
    }

    /// Converts self into a `FiniteOrdBound<T>`
    pub fn into_finite_ord(self, side: Side) -> ord::FiniteOrdBound<T> {
        let (bound_type, value) = self.into_raw();
        match bound_type {
            BoundType::Closed => ord::FiniteOrdBound::closed(value),
            BoundType::Open => ord::FiniteOrdBound::open(side, value),
        }
    }

    /// Converts self into an `OrdBound<T>`
    pub fn into_ord(self, side: Side) -> ord::OrdBound<T> {
        ord::OrdBound::Finite(self.into_finite_ord(side))
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
        Self::strict_take_min(side, a, b).unwrap()
    }

    pub fn strict_take_min(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> Result<FiniteBound<T>, TotalOrderError> {
        if a.strict_contains_bound(side, &b)? {
            Ok(side.select(a, b))
        } else {
            Ok(side.select(b, a))
        }
    }

    /// Consume a and b, returning the maximum bound.
    pub fn take_max(side: Side, a: FiniteBound<T>, b: FiniteBound<T>) -> FiniteBound<T> {
        Self::strict_take_max(side, a, b).unwrap()
    }

    pub fn strict_take_max(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> Result<FiniteBound<T>, TotalOrderError> {
        if a.strict_contains_bound(side, &b)? {
            Ok(side.select(b, a))
        } else {
            Ok(side.select(a, b))
        }
    }

    /// Return a reference to the minimum bound.
    pub fn min<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        Self::strict_min(side, a, b).unwrap()
    }

    pub fn strict_min<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> Result<&'a FiniteBound<T>, TotalOrderError> {
        if a.strict_contains_bound(side, b)? {
            Ok(side.select(a, b))
        } else {
            Ok(side.select(b, a))
        }
    }

    /// Return a reference to the maximum bound.
    pub fn max<'a>(side: Side, a: &'a FiniteBound<T>, b: &'a FiniteBound<T>) -> &'a FiniteBound<T> {
        Self::strict_max(side, a, b).unwrap()
    }

    pub fn strict_max<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> Result<&'a FiniteBound<T>, TotalOrderError> {
        if a.strict_contains_bound(side, b)? {
            Ok(side.select(b, a))
        } else {
            Ok(side.select(a, b))
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

    pub fn strict_contains(&self, side: Side, test: &T) -> Result<bool, TotalOrderError> {
        let lhs = self.finite_ord(side);
        let rhs = ord::FiniteOrdBound::closed(test);
        let order = lhs
            .partial_cmp(&rhs)
            .ok_or(TotalOrderError::new("FiniteBound::strict_contains"))?;

        Ok(order == Equal || order == side.select(Less, Greater))
    }

    pub fn contains_bound(&self, side: Side, test: &FiniteBound<T>) -> bool {
        let lhs = self.finite_ord(side);
        let rhs = test.finite_ord(side);
        match side {
            Side::Left => lhs <= rhs,
            Side::Right => rhs <= lhs,
        }
    }

    pub fn strict_contains_bound(
        &self,
        side: Side,
        test: &FiniteBound<T>,
    ) -> Result<bool, TotalOrderError> {
        let lhs = self.finite_ord(side);
        let rhs = test.finite_ord(side);
        let order = lhs
            .partial_cmp(&rhs)
            .ok_or(TotalOrderError::new("FiniteBound::strict_contains_bound"))?;

        Ok(order == Equal || order == side.select(Less, Greater))
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

mod math {
    use core::ops::{Add, Mul};

    use num_traits::{ConstOne, ConstZero, One, Zero};

    use super::{BoundType, FiniteBound};

    impl<T: Add> Add for FiniteBound<T> {
        type Output = FiniteBound<<T as Add>::Output>;

        fn add(self, rhs: Self) -> Self::Output {
            FiniteBound::new(self.0.combine(rhs.0), self.1 + rhs.1)
        }
    }

    impl<T: Mul> Mul for FiniteBound<T> {
        type Output = FiniteBound<<T as Mul>::Output>;

        fn mul(self, rhs: Self) -> Self::Output {
            FiniteBound::new(self.0.combine(rhs.0), self.1 * rhs.1)
        }
    }

    impl<T: Zero> Zero for FiniteBound<T> {
        fn zero() -> Self {
            FiniteBound::closed(T::zero())
        }

        fn is_zero(&self) -> bool {
            self.0 == BoundType::Closed && self.1.is_zero()
        }
    }

    impl<T: One + PartialEq> One for FiniteBound<T> {
        fn one() -> Self {
            FiniteBound::closed(T::one())
        }
    }

    impl<T: ConstZero> ConstZero for FiniteBound<T> {
        const ZERO: Self = FiniteBound::closed(T::ZERO);
    }

    impl<T: ConstOne + PartialEq> ConstOne for FiniteBound<T> {
        const ONE: Self = FiniteBound::closed(T::ONE);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_const_zero_one() {
            const X: FiniteBound<i32> = FiniteBound::ONE;
            const Y: FiniteBound<i32> = FiniteBound::ZERO;
            assert_eq!(X + Y, FiniteBound::ONE);
        }

        #[test]
        fn test_add() {
            let c10 = FiniteBound::closed(10);
            let c20 = FiniteBound::closed(20);
            let o10 = FiniteBound::open(10);
            let o20 = FiniteBound::open(20);
            assert_eq!(c10 + c10, c20);
            assert_eq!(c10 + o10, o20);
            assert_eq!(o10 + c10, o20);
            assert_eq!(o10 + o10, o20);
        }

        #[test]
        fn test_mul() {
            let c10 = FiniteBound::closed(10);
            let c100 = FiniteBound::closed(100);
            let o10 = FiniteBound::open(10);
            let o100 = FiniteBound::open(100);
            assert_eq!(c10 * c10, c100);
            assert_eq!(c10 * o10, o100);
            assert_eq!(o10 * c10, o100);
            assert_eq!(o10 * o10, o100);
        }
    }
}

/// Helpers that define a total order for `Set` bounds.
pub mod ord {
    use super::{BoundType, FiniteBound};

    /// Any type with left and right bounds, following the standard total order.
    pub trait OrdBounded<T> {
        /// Create an ordered bound pair view of a set's bounds.
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
    #[allow(missing_docs)]
    pub enum OrdBound<T> {
        LeftUnbounded,
        Finite(FiniteOrdBound<T>),
        RightUnbounded,
    }

    impl<T> OrdBound<T> {
        pub const fn new_finite(limit: T, kind: FiniteOrdBoundKind) -> Self {
            Self::Finite(FiniteOrdBound::new(limit, kind))
        }

        /// Create a finite left open OrdBound
        pub const fn left_open(limit: T) -> Self {
            Self::new_finite(limit, FiniteOrdBoundKind::LeftOpen)
        }

        /// Create a finite closed OrdBound
        pub const fn closed(limit: T) -> Self {
            Self::new_finite(limit, FiniteOrdBoundKind::Closed)
        }

        /// Create a finite right open OrdBound
        pub const fn right_open(limit: T) -> Self {
            Self::new_finite(limit, FiniteOrdBoundKind::RightOpen)
        }
    }

    impl<'a, T> OrdBound<&'a T> {
        /// Create a left OrdBound view of a &FiniteBound.
        pub fn left(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::new_finite(bound.value(), Closed),
                BoundType::Open => Self::new_finite(bound.value(), LeftOpen),
            }
        }

        /// Create a right OrdBound view of a &FiniteBound.
        pub fn right(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::new_finite(bound.value(), Closed),
                BoundType::Open => Self::new_finite(bound.value(), RightOpen),
            }
        }
    }

    impl<T: Clone> OrdBound<&T> {
        /// Create an owned `OrdBound<T>` from an `OrdBound<&T>` view.
        pub fn cloned(self) -> OrdBound<T> {
            match self {
                Finite(inner) => Finite(inner.cloned()),
                LeftUnbounded => LeftUnbounded,
                RightUnbounded => RightUnbounded,
            }
        }
    }

    /*impl<T> OrdBound<T> {
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
    }*/

    /// Ordered exclusivity cases for finite bounds.
    ///
    /// For a given finite value x, RightOpen(x) < Closed(x) < LeftOpen(x).
    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
    )]
    #[allow(missing_docs)]
    pub enum FiniteOrdBoundKind {
        RightOpen,
        Closed,
        LeftOpen,
    }

    use FiniteOrdBoundKind::*;
    use OrdBound::*;

    impl FiniteOrdBoundKind {
        /// Create the correctly sided open ord bound type.
        pub fn open(side: super::Side) -> Self {
            side.select(LeftOpen, RightOpen)
        }
    }

    /// Finite bound with a total ordering
    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
    )]
    pub struct FiniteOrdBound<T>(pub T, pub FiniteOrdBoundKind);

    impl<T> FiniteOrdBound<T> {
        #[inline(always)]
        pub const fn new(limit: T, kind: FiniteOrdBoundKind) -> Self {
            Self(limit, kind)
        }

        #[inline(always)]
        pub const fn closed(limit: T) -> Self {
            Self::new(limit, FiniteOrdBoundKind::Closed)
        }

        #[inline(always)]
        pub const fn open(side: super::Side, limit: T) -> Self {
            Self::new(
                limit,
                match side {
                    super::Side::Left => LeftOpen,
                    super::Side::Right => RightOpen,
                },
            )
        }
    }

    impl<T: Clone> FiniteOrdBound<&T> {
        pub fn cloned(&self) -> FiniteOrdBound<T> {
            FiniteOrdBound::new(self.0.clone(), self.1)
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
        /// Test if this is the empty set.
        pub fn is_empty(&self) -> bool {
            *self == Self::empty()
        }
    }

    impl<T> OrdBoundPair<T> {
        /// Create a new empty set.
        pub const fn empty() -> Self {
            Self(LeftUnbounded, LeftUnbounded)
        }

        /// Creates a new totally ordered bound pair.
        pub fn new(left: OrdBound<T>, right: OrdBound<T>) -> Self {
            match (left, right) {
                // use (LU, LU) to represent EMPTY and make it the lowest element
                (LeftUnbounded, LeftUnbounded) => Self::empty(),
                (left, right) => {
                    debug_assert!(!matches!(&left, RightUnbounded));
                    debug_assert!(!matches!(&right, LeftUnbounded));
                    debug_assert!(!matches!(&left, Finite(FiniteOrdBound(_, RightOpen))));
                    debug_assert!(!matches!(&right, Finite(FiniteOrdBound(_, LeftOpen))));
                    Self(left, right)
                }
            }
        }

        /// Decompose into the pair of OrdBounds
        pub fn into_raw(self) -> (OrdBound<T>, OrdBound<T>) {
            (self.0, self.1)
        }
    }
}
#[cfg(test)]
mod test {
    use ord::OrdBound;

    use super::Side::*;
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
            Ok(OrdBound::closed(f0))
        );

        assert_eq!(
            OrdBound::closed(&f0).try_min(OrdBound::closed(&f1)),
            Ok(OrdBound::closed(&f0))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed(f1)),
            Ok(OrdBound::closed(f1))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed(&f1)),
            Ok(OrdBound::closed(&f1))
        )
    }

    #[test]
    pub fn test_strict_contains() {
        let x = FiniteBound::closed(0.0);

        assert_eq!(x.strict_contains(Left, &0.0).unwrap(), true);
        assert_eq!(x.strict_contains(Left, &1.0).unwrap(), true);
        assert_eq!(x.strict_contains(Left, &-1.0).unwrap(), false);
        assert_eq!(x.strict_contains(Left, &f64::NAN).is_err(), true);

        assert_eq!(x.strict_contains(Right, &0.0).unwrap(), true);
        assert_eq!(x.strict_contains(Right, &-1.0).unwrap(), true);
        assert_eq!(x.strict_contains(Right, &1.0).unwrap(), false);
        assert_eq!(x.strict_contains(Right, &f64::NAN).is_err(), true);

        let open = FiniteBound::open(0.0);

        assert_eq!(open.strict_contains(Left, &0.0).unwrap(), false);
        assert_eq!(open.strict_contains(Left, &1.0).unwrap(), true);
        assert_eq!(open.strict_contains(Left, &-1.0).unwrap(), false);
        assert_eq!(open.strict_contains(Left, &f64::NAN).is_err(), true);

        assert_eq!(open.strict_contains(Right, &0.0).unwrap(), false);
        assert_eq!(open.strict_contains(Right, &-1.0).unwrap(), true);
        assert_eq!(open.strict_contains(Right, &1.0).unwrap(), false);
        assert_eq!(open.strict_contains(Right, &f64::NAN).is_err(), true);
    }

    #[test]
    fn test_strict_contains_bound() {
        let cl_0 = FiniteBound::closed(0.0);
        let cl_p1 = FiniteBound::closed(1.0);
        let cl_n1 = FiniteBound::closed(-1.0);
        let cl_nan = FiniteBound::closed(f64::NAN);

        let op_0 = FiniteBound::open(0.0);
        let op_p1 = FiniteBound::open(1.0);
        let op_n1 = FiniteBound::open(-1.0);
        let op_nan = FiniteBound::open(f64::NAN);

        assert_eq!(cl_0.strict_contains_bound(Left, &cl_0).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Left, &cl_p1).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Left, &cl_n1).unwrap(), false);
        assert_eq!(cl_0.strict_contains_bound(Left, &cl_nan).is_err(), true);

        assert_eq!(cl_0.strict_contains_bound(Left, &op_0).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Left, &op_p1).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Left, &op_n1).unwrap(), false);
        assert_eq!(cl_0.strict_contains_bound(Left, &op_nan).is_err(), true);

        assert_eq!(op_0.strict_contains_bound(Left, &op_0).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Left, &op_p1).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Left, &op_n1).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Left, &op_nan).is_err(), true);

        assert_eq!(op_0.strict_contains_bound(Left, &cl_0).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Left, &cl_p1).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Left, &cl_n1).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Left, &cl_nan).is_err(), true);

        assert_eq!(cl_0.strict_contains_bound(Right, &cl_0).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Right, &cl_p1).unwrap(), false);
        assert_eq!(cl_0.strict_contains_bound(Right, &cl_n1).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Right, &cl_nan).is_err(), true);

        assert_eq!(cl_0.strict_contains_bound(Right, &op_0).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Right, &op_p1).unwrap(), false);
        assert_eq!(cl_0.strict_contains_bound(Right, &op_n1).unwrap(), true);
        assert_eq!(cl_0.strict_contains_bound(Right, &op_nan).is_err(), true);

        assert_eq!(op_0.strict_contains_bound(Right, &op_0).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Right, &op_p1).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Right, &op_n1).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Right, &op_nan).is_err(), true);

        assert_eq!(op_0.strict_contains_bound(Right, &cl_0).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Right, &cl_p1).unwrap(), false);
        assert_eq!(op_0.strict_contains_bound(Right, &cl_n1).unwrap(), true);
        assert_eq!(op_0.strict_contains_bound(Right, &cl_nan).is_err(), true);
    }
}

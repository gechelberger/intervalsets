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

use crate::error::{Error, TotalOrderError};
use crate::numeric::Element;

/// An interface to query the left and right bounds of a set.
pub trait SetBounds<T> {
    /// Return a reference to the left or right bound if it is finite.
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>>;

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
pub enum BoundType {
    /// An Open BoundType excludes the limit element from the `Set`.
    Open,
    /// A Closed BoundType includes the limit element in the `Set`.
    Closed,
}

impl BoundType {
    /// Flips the bound type Open => Closed, Closed => Open
    pub const fn flip(self) -> Self {
        match self {
            Self::Closed => Self::Open,
            Self::Open => Self::Closed,
        }
    }

    /// The open/closed lattice meet: returns `Closed` iff both inputs are
    /// `Closed`, else `Open`.
    ///
    /// Used in interval arithmetic to derive the resulting bound type when
    /// two endpoints combine — the result includes its endpoint only when
    /// both source endpoints are included.
    pub const fn meet(self, rhs: Self) -> Self {
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
#[cfg_attr(feature = "serde", serde(try_from = "RawFiniteBound<T>"))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))
)]
pub struct FiniteBound<T>(BoundType, T);

/// Wire-format mirror of [`FiniteBound`] used to drive validation
/// during `Deserialize`. Identical layout, no invariants.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "FiniteBound")]
struct RawFiniteBound<T>(BoundType, T);

#[cfg(feature = "serde")]
impl<T: Element> TryFrom<RawFiniteBound<T>> for FiniteBound<T> {
    type Error = Error;

    fn try_from(raw: RawFiniteBound<T>) -> Result<Self, Self::Error> {
        Self::try_new(raw.0, raw.1)
    }
}

impl<T> FiniteBound<T> {
    /// Tier 4 bypass: construct a `FiniteBound` without running
    /// `Element::validate`. Caller asserts that `limit` is a valid
    /// element. For panic-free op sites that can prove validity by
    /// local context (e.g. `T::zero()` or `T::min_value()` for types
    /// where those are always valid).
    #[inline(always)]
    pub(crate) const fn new_assume_valid(bound_type: BoundType, limit: T) -> Self {
        Self(bound_type, limit)
    }

    /// Unpack a [`FiniteBound`] into ([`BoundType`], `T`)
    pub fn into_raw(self) -> (BoundType, T) {
        (self.0, self.1)
    }

    /// Converts `&FiniteBound<T>` to `FiniteBound<&T>`.
    pub fn as_ref(&self) -> FiniteBound<&T> {
        FiniteBound(self.0, &self.1)
    }

    /// Creates a `FiniteOrdBound<&T>` view of this `FiniteBound<T>`.
    pub fn finite_ord(&self, side: Side) -> ord::FiniteOrdBound<&T> {
        match self.bound_type() {
            BoundType::Closed => ord::FiniteOrdBound::closed_assume_valid(self.value()),
            BoundType::Open => ord::FiniteOrdBound::open_assume_valid(side, self.value()),
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
            BoundType::Closed => ord::FiniteOrdBound::closed_assume_valid(value),
            BoundType::Open => ord::FiniteOrdBound::open_assume_valid(side, value),
        }
    }

    /// Converts self into an `OrdBound<T>`
    pub fn into_ord(self, side: Side) -> ord::OrdBound<T> {
        ord::OrdBound::Finite(self.into_finite_ord(side))
    }

    /// Returns a new `FiniteBound` that keeps the same limit value and
    /// flips its `BoundType`. Consumes `self`.
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
}

impl<T: PartialOrd> FiniteBound<T> {
    /// Returns `(a, b)` reordered so that the first element is the bound
    /// closer to `side` (the minimum for `Side::Left`, the maximum for
    /// `Side::Right`).
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn min_max_assume_valid(
        side: Side,
        mut a: FiniteBound<T>,
        mut b: FiniteBound<T>,
    ) -> (FiniteBound<T>, FiniteBound<T>) {
        debug_assert!(a.value().partial_cmp(b.value()).is_some());
        if a.contains_bound_assume_valid(side, b.as_ref()) {
            if side == Side::Right {
                core::mem::swap(&mut a, &mut b);
            }
        } else if side == Side::Left {
            core::mem::swap(&mut a, &mut b);
        }
        (a, b)
    }

    /// Consume a and b, returning the minimum bound.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn take_min_assume_valid(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> FiniteBound<T> {
        debug_assert!(a.value().partial_cmp(b.value()).is_some());
        if a.contains_bound_assume_valid(side, b.as_ref()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    /// Consume a and b, returning the min bound or Err if not comparable.
    pub fn try_take_min(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> Result<FiniteBound<T>, TotalOrderError> {
        if a.try_contains_bound(side, b.as_ref())? {
            Ok(side.select(a, b))
        } else {
            Ok(side.select(b, a))
        }
    }

    /// Consume a and b, returning the maximum bound.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn take_max_assume_valid(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> FiniteBound<T> {
        debug_assert!(a.value().partial_cmp(b.value()).is_some());
        if a.contains_bound_assume_valid(side, b.as_ref()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }

    /// Consumes a and b, returning the max bound or Err if not comparable.
    pub fn try_take_max(
        side: Side,
        a: FiniteBound<T>,
        b: FiniteBound<T>,
    ) -> Result<FiniteBound<T>, TotalOrderError> {
        if a.try_contains_bound(side, b.as_ref())? {
            Ok(side.select(b, a))
        } else {
            Ok(side.select(a, b))
        }
    }

    /// Return a reference to the minimum bound.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn min_assume_valid<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> &'a FiniteBound<T> {
        debug_assert!(a.value().partial_cmp(b.value()).is_some());
        if a.contains_bound_assume_valid(side, b.as_ref()) {
            side.select(a, b)
        } else {
            side.select(b, a)
        }
    }

    /// Return a ref to the min bound or Err if not comparable.
    pub fn try_min<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> Result<&'a FiniteBound<T>, TotalOrderError> {
        if a.try_contains_bound(side, b.as_ref())? {
            Ok(side.select(a, b))
        } else {
            Ok(side.select(b, a))
        }
    }

    /// Return a reference to the maximum bound.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn max_assume_valid<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> &'a FiniteBound<T> {
        debug_assert!(a.value().partial_cmp(b.value()).is_some());
        if a.contains_bound_assume_valid(side, b.as_ref()) {
            side.select(b, a)
        } else {
            side.select(a, b)
        }
    }

    /// Return a reference to the max bound or Err if not comparable.
    pub fn try_max<'a>(
        side: Side,
        a: &'a FiniteBound<T>,
        b: &'a FiniteBound<T>,
    ) -> Result<&'a FiniteBound<T>, TotalOrderError> {
        if a.try_contains_bound(side, b.as_ref())? {
            Ok(side.select(b, a))
        } else {
            Ok(side.select(a, b))
        }
    }
}

impl<T: PartialOrd> FiniteBound<T> {
    /// Test whether the `side`-half-plane defined by this bound contains
    /// `value`.
    ///
    /// # Preconditions
    ///
    /// `self` and `value` must be comparable. Violating this yields
    /// incorrect results but no undefined behavior.
    pub fn contains_assume_valid(&self, side: Side, value: &T) -> bool {
        debug_assert!(self.value().partial_cmp(value).is_some());
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

    /// Test whether the `side`-half-plane defined by this bound contains
    /// `test`. Returns `Err` if the values are not comparable.
    pub fn try_contains(&self, side: Side, test: &T) -> Result<bool, TotalOrderError> {
        let lhs = self.finite_ord(side);
        let rhs = ord::FiniteOrdBound::closed_assume_valid(test);
        let order = lhs.partial_cmp(&rhs).ok_or(TotalOrderError)?;

        Ok(order == Equal || order == side.select(Less, Greater))
    }

    /// Test whether the `side`-half-plane defined by `self` contains the
    /// `test` bound, oriented from the same `side`.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be comparable. Violating this yields incorrect
    /// results but no undefined behavior.
    pub fn contains_bound_assume_valid(&self, side: Side, test: FiniteBound<&T>) -> bool {
        let lhs = self.finite_ord(side);
        let rhs = test.into_finite_ord(side);
        debug_assert!(lhs.partial_cmp(&rhs).is_some());
        match side {
            Side::Left => lhs <= rhs,
            Side::Right => rhs <= lhs,
        }
    }

    /// Test whether the `side`-half-plane defined by `self` contains the
    /// `test` bound, oriented from the same `side`. Returns `Err` if the
    /// values are not comparable.
    pub fn try_contains_bound(
        &self,
        side: Side,
        test: FiniteBound<&T>,
    ) -> Result<bool, TotalOrderError> {
        let lhs = self.finite_ord(side);
        let rhs = test.into_finite_ord(side);
        let order = lhs.partial_cmp(&rhs).ok_or(TotalOrderError)?;

        Ok(order == Equal || order == side.select(Less, Greater))
    }
}

impl<T: Element> FiniteBound<T> {
    /// For discrete types, normalize to closed form.
    pub fn normalized(self, side: Side) -> Self {
        match self.0 {
            BoundType::Open => match self.value().try_adjacent(side.flip()) {
                None => self,
                Some(new_limit) => Self(BoundType::Closed, new_limit),
            },
            BoundType::Closed => self,
        }
    }

    /// Validates `limit` via [`Element::validate`] and constructs a
    /// `FiniteBound`. The single chokepoint where validation fires for
    /// every non-bypass construction path.
    ///
    /// Library float types (`f32`, `f64`, `OrderedFloat<f*>`,
    /// `NotNan<f*>`) reject `±INF` and `NaN` here.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`Error::InvalidBoundLimit`]
    /// when `T::validate` returns `None`.
    #[inline]
    pub fn try_new(bound_type: BoundType, limit: T) -> Result<Self, Error> {
        match limit.validate() {
            Some(v) => Ok(Self(bound_type, v)),
            None => Err(Error::InvalidBoundLimit),
        }
    }

    /// Validates `limit` and constructs a closed `FiniteBound`. See
    /// [`try_new`](Self::try_new).
    #[inline]
    pub fn try_closed(limit: T) -> Result<Self, Error> {
        Self::try_new(BoundType::Closed, limit)
    }

    /// Validates `limit` and constructs an open `FiniteBound`. See
    /// [`try_new`](Self::try_new).
    #[inline]
    pub fn try_open(limit: T) -> Result<Self, Error> {
        Self::try_new(BoundType::Open, limit)
    }

    /// Panicking constructor. Equivalent to
    /// [`try_new`](Self::try_new)`.unwrap()`.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is rejected by [`Element::validate`]
    /// (e.g. `NaN` or `±INF` on library float types).
    #[inline]
    pub fn new(bound_type: BoundType, limit: T) -> Self {
        Self::try_new(bound_type, limit).unwrap()
    }

    /// Panicking constructor for a closed bound. Equivalent to
    /// [`try_closed`](Self::try_closed)`.unwrap()`.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is rejected by [`Element::validate`].
    #[inline]
    pub fn closed(limit: T) -> Self {
        Self::try_closed(limit).unwrap()
    }

    /// Panicking constructor for an open bound. Equivalent to
    /// [`try_open`](Self::try_open)`.unwrap()`.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is rejected by [`Element::validate`].
    #[inline]
    pub fn open(limit: T) -> Self {
        Self::try_open(limit).unwrap()
    }
}

mod math {
    use core::fmt::Debug;
    use core::ops::{Add, Div, Mul, Sub};

    use num_traits::{ConstOne, ConstZero, One, Zero};

    use super::{BoundType, FiniteBound};
    use crate::error::Error;
    use crate::numeric::Element;
    use crate::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

    impl<T> TryAdd for FiniteBound<T>
    where
        T: Element + TryAdd<Output = T>,
        <T as TryAdd>::Error: Into<Error>,
    {
        type Output = FiniteBound<T>;
        type Error = Error;

        fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
            let (l_kind, l_val) = self.into_raw();
            let (r_kind, r_val) = rhs.into_raw();
            let val = l_val.try_add(r_val).map_err(Into::into)?;
            FiniteBound::try_new(l_kind.meet(r_kind), val)
        }
    }

    impl<T> TrySub for FiniteBound<T>
    where
        T: Element + TrySub<Output = T>,
        <T as TrySub>::Error: Into<Error>,
    {
        type Output = FiniteBound<T>;
        type Error = Error;

        fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
            let (l_kind, l_val) = self.into_raw();
            let (r_kind, r_val) = rhs.into_raw();
            let val = l_val.try_sub(r_val).map_err(Into::into)?;
            FiniteBound::try_new(l_kind.meet(r_kind), val)
        }
    }

    impl<T> TryMul for FiniteBound<T>
    where
        T: Element + Zero + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        type Output = FiniteBound<T>;
        type Error = Error;

        fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
            // Closed(0) absorbs: 0 * x = 0, the value 0 is achieved
            // regardless of the other operand's openness, so the result
            // bound is Closed even if the other operand is Open.
            let absorbing = (self.0 == BoundType::Closed && self.1.is_zero())
                || (rhs.0 == BoundType::Closed && rhs.1.is_zero());
            let (l_kind, l_val) = self.into_raw();
            let (r_kind, r_val) = rhs.into_raw();
            let val = l_val.try_mul(r_val).map_err(Into::into)?;
            let kind = if absorbing {
                BoundType::Closed
            } else {
                l_kind.meet(r_kind)
            };
            FiniteBound::try_new(kind, val)
        }
    }

    impl<T> TryDiv for FiniteBound<T>
    where
        T: Element + Zero + TryDiv<Output = T>,
        <T as TryDiv>::Error: Into<Error>,
    {
        type Output = FiniteBound<T>;
        type Error = Error;

        fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
            // Closed(0) numerator absorbs: 0 / nonzero = 0, regardless
            // of the denominator's openness.
            let absorbing = self.0 == BoundType::Closed && self.1.is_zero();
            let (l_kind, l_val) = self.into_raw();
            let (r_kind, r_val) = rhs.into_raw();
            let val = l_val.try_div(r_val).map_err(Into::into)?;
            let kind = if absorbing {
                BoundType::Closed
            } else {
                l_kind.meet(r_kind)
            };
            FiniteBound::try_new(kind, val)
        }
    }

    // Infix `+ - * /` for `FiniteBound<T>` is panicking sugar over
    // `try_*().unwrap()`. May panic per Tier 3b; the panic site is the
    // documented contract.

    impl<T> Add for FiniteBound<T>
    where
        Self: TryAdd<Output = Self>,
        <Self as TryAdd>::Error: Debug,
    {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            self.try_add(rhs).unwrap()
        }
    }

    impl<T> Sub for FiniteBound<T>
    where
        Self: TrySub<Output = Self>,
        <Self as TrySub>::Error: Debug,
    {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self::Output {
            self.try_sub(rhs).unwrap()
        }
    }

    impl<T> Mul for FiniteBound<T>
    where
        Self: TryMul<Output = Self>,
        <Self as TryMul>::Error: Debug,
    {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self::Output {
            self.try_mul(rhs).unwrap()
        }
    }

    impl<T> Div for FiniteBound<T>
    where
        Self: TryDiv<Output = Self>,
        <Self as TryDiv>::Error: Debug,
    {
        type Output = Self;
        fn div(self, rhs: Self) -> Self::Output {
            self.try_div(rhs).unwrap()
        }
    }

    // num_traits::Zero requires Self: Add<Self, Output = Self>; One
    // requires Self: Mul<Self, Output = Self>. The infix Add/Mul impls
    // above are sugar over Try*, so Zero/One pick up the same
    // Try*<Output = T> + Error: Debug bounds transitively.

    impl<T> Zero for FiniteBound<T>
    where
        T: Element + Zero + TryAdd<Output = T>,
        <T as TryAdd>::Error: Debug + Into<Error>,
    {
        fn zero() -> Self {
            FiniteBound(BoundType::Closed, T::zero())
        }

        fn is_zero(&self) -> bool {
            self.0 == BoundType::Closed && self.1.is_zero()
        }
    }

    impl<T> One for FiniteBound<T>
    where
        T: Element + One + Zero + PartialEq + TryMul<Output = T>,
        <T as TryMul>::Error: Debug + Into<Error>,
    {
        fn one() -> Self {
            FiniteBound(BoundType::Closed, T::one())
        }
    }

    impl<T> ConstZero for FiniteBound<T>
    where
        T: Element + ConstZero + TryAdd<Output = T>,
        <T as TryAdd>::Error: Debug + Into<Error>,
    {
        const ZERO: Self = FiniteBound(BoundType::Closed, T::ZERO);
    }

    impl<T> ConstOne for FiniteBound<T>
    where
        T: Element + ConstOne + Zero + PartialEq + TryMul<Output = T>,
        <T as TryMul>::Error: Debug + Into<Error>,
    {
        const ONE: Self = FiniteBound(BoundType::Closed, T::ONE);
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

        #[test]
        fn test_mul_closed_zero_absorbs() {
            // Closed(0) * x = 0 with the value 0 always achieved, so the
            // result bound is Closed(0) regardless of x's openness.
            let c0 = FiniteBound::closed(0);
            let c5 = FiniteBound::closed(5);
            let o5 = FiniteBound::open(5);
            let o0 = FiniteBound::open(0);

            // Closed(0) absorbs on either side
            assert_eq!(c0 * c5, FiniteBound::closed(0));
            assert_eq!(c5 * c0, FiniteBound::closed(0));
            assert_eq!(c0 * o5, FiniteBound::closed(0));
            assert_eq!(o5 * c0, FiniteBound::closed(0));
            assert_eq!(c0 * c0, FiniteBound::closed(0));

            // Open(0) does NOT absorb: 0 is not in the input bound
            // semantically, so the output is Open(0).
            assert_eq!(o0 * c5, FiniteBound::open(0));
            assert_eq!(c5 * o0, FiniteBound::open(0));
            assert_eq!(o0 * o5, FiniteBound::open(0));
            assert_eq!(o0 * o0, FiniteBound::open(0));
        }
    }
}

/// Helpers that define a total order for `Set` bounds.
pub mod ord {
    use core::cmp::Ordering::Greater;

    use super::{BoundType, FiniteBound};
    use crate::error::Error;
    use crate::try_cmp::TryCmp;

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
    #[allow(missing_docs)]
    pub enum OrdBound<T> {
        LeftUnbounded,
        Finite(FiniteOrdBound<T>),
        RightUnbounded,
    }

    impl<T> OrdBound<T> {
        /// Tier-4 bypass: constructs a finite `OrdBound<T>` without
        /// validating `limit`. See
        /// [`FiniteOrdBound::new_assume_valid`](FiniteOrdBound::new_assume_valid).
        pub(crate) const fn new_finite_assume_valid(limit: T, kind: FiniteOrdBoundKind) -> Self {
            Self::Finite(FiniteOrdBound::new_assume_valid(limit, kind))
        }

        /// Tier-4 bypass: closed finite `OrdBound<T>` without validating
        /// `limit`.
        pub(crate) const fn closed_assume_valid(limit: T) -> Self {
            Self::new_finite_assume_valid(limit, FiniteOrdBoundKind::Closed)
        }
    }

    impl<'a, T> OrdBound<&'a T> {
        /// Create a left `OrdBound<T>` view of a `&FiniteBound<T>`.
        pub fn left(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::new_finite_assume_valid(bound.value(), Closed),
                BoundType::Open => Self::new_finite_assume_valid(bound.value(), LeftOpen),
            }
        }

        /// Create a right `OrdBound<T>` view of a `&FiniteBound<T>`.
        pub fn right(bound: &'a FiniteBound<T>) -> Self {
            match bound.bound_type() {
                BoundType::Closed => Self::new_finite_assume_valid(bound.value(), Closed),
                BoundType::Open => Self::new_finite_assume_valid(bound.value(), RightOpen),
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
    pub struct FiniteOrdBound<T>(pub T, pub FiniteOrdBoundKind);

    impl<T> FiniteOrdBound<T> {
        /// Tier-4 bypass: constructs a `FiniteOrdBound<T>` without
        /// validating `limit`. The caller asserts that, for the
        /// downstream uses they intend, `limit` is acceptable —
        /// `FiniteOrdBound` itself enforces no invariant on its inner
        /// value, but conversions like `TryFrom<FiniteOrdBound<T>> for
        /// FiniteBound<T>` are fallible specifically because this
        /// guarantee is missing.
        #[inline(always)]
        pub(crate) const fn new_assume_valid(limit: T, kind: FiniteOrdBoundKind) -> Self {
            Self(limit, kind)
        }

        /// Tier-4 bypass: closed `FiniteOrdBound<T>` without validating
        /// `limit`. See [`new_assume_valid`](Self::new_assume_valid).
        #[inline(always)]
        pub(crate) const fn closed_assume_valid(limit: T) -> Self {
            Self::new_assume_valid(limit, FiniteOrdBoundKind::Closed)
        }

        /// Tier-4 bypass: side-oriented open `FiniteOrdBound<T>` without
        /// validating `limit`. See [`new_assume_valid`](Self::new_assume_valid).
        #[inline(always)]
        pub(crate) const fn open_assume_valid(side: super::Side, limit: T) -> Self {
            Self::new_assume_valid(
                limit,
                match side {
                    super::Side::Left => LeftOpen,
                    super::Side::Right => RightOpen,
                },
            )
        }
    }

    impl<T: Clone> FiniteOrdBound<&T> {
        /// Converts `FiniteOrdBound<&T>` to `FiniteOrdBound<T>`.
        pub fn cloned(&self) -> FiniteOrdBound<T> {
            FiniteOrdBound::new_assume_valid(self.0.clone(), self.1)
        }
    }

    /// A canonical, totally-ordered representation of an interval's
    /// bounds: the pair `(left, right)` with `left <= right` in the
    /// augmented total order on bounds. The empty set is represented
    /// by the sentinel `(LeftUnbounded, LeftUnbounded)` (the lowest
    /// element of the order).
    ///
    /// # Public role
    ///
    /// `OrdBoundPair` is conversion currency, not a primary user type.
    /// Two reasons it stays `pub`:
    ///
    /// 1. **Outbound** — every interval-shaped public type (`FiniteInterval`,
    ///    `HalfInterval`, `EnumInterval`, the outer crate's `Interval` and
    ///    `IntervalSet`) implements `From<&Self> for OrdBoundPair<&T>` (and
    ///    `From<Self> for OrdBoundPair<T>`). Code that wants a uniform
    ///    "two ordered endpoints" representation across mixed interval
    ///    types — e.g. for hashing, sorting, or comparing bounds without
    ///    branching on variant — uses this conversion.
    ///
    /// 2. **Inbound** — `TryFrom<OrdBoundPair<T>> for EnumInterval<T>`
    ///    (and the outer `Interval`/`IntervalSet`) reconstructs an
    ///    interval from a raw ord pair. The typical use is round-trip
    ///    after extracting via outbound conversion (Role 1) and
    ///    manipulating the bounds; constructing one from scratch is
    ///    rare and not the recommended pattern.
    ///
    /// # Constructors
    ///
    /// All three follow the [crate-wide constructor convention](crate#construction-at-boundaries):
    ///
    /// - [`empty`](Self::empty) — the canonical empty marker, no bound
    ///   on T, `const`.
    /// - [`new_assume_valid`](Self::new_assume_valid) — bypass; caller
    ///   guarantees the preconditions; no bound on T, `const`.
    /// - [`new`](Self::new) — panicking validating variant. Requires
    ///   `T: PartialOrd`.
    /// - [`try_new`](Self::try_new) — fallible validating variant.
    ///   Returns `Err` for any structural or value-level violation
    ///   (NaN / `left.value() > right.value()` / structurally invalid
    ///   `(LeftUnbounded, _)` etc). Requires `T: PartialOrd`.
    ///
    /// `OrdBoundPair` does **not** derive `serde::Serialize` /
    /// `serde::Deserialize`; it is not part of the documented public
    /// wire-format contract. If you want to round-trip bound pair data,
    /// serialize the interval type that contains them.
    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct OrdBoundPair<T>(OrdBound<T>, OrdBound<T>);

    impl<T> OrdBoundPair<T> {
        /// Test if this is the empty set.
        ///
        /// The empty marker is `(LeftUnbounded, LeftUnbounded)`. Detection
        /// requires only matching on the discriminants, so this method
        /// imposes no bound on `T`.
        pub fn is_empty(&self) -> bool {
            matches!(self.0, LeftUnbounded) && matches!(self.1, LeftUnbounded)
        }

        /// Create a new empty set.
        pub const fn empty() -> Self {
            Self(LeftUnbounded, LeftUnbounded)
        }

        /// Creates an `OrdBoundPair` without validating its invariants.
        ///
        /// # Preconditions
        ///
        /// 1. `left` is not `RightUnbounded`.
        /// 2. `right` is not `LeftUnbounded`, except in the canonical empty
        ///    pair `(LeftUnbounded, LeftUnbounded)`.
        /// 3. If `left` is `Finite`, its kind is not `RightOpen`.
        /// 4. If `right` is `Finite`, its kind is not `LeftOpen`.
        /// 5. When both ends are `Finite`, `left.value() <= right.value()`
        ///    in total order (no NaN, not swapped).
        ///
        /// Violating any precondition yields incorrect results downstream
        /// but no undefined behavior. Preconditions 1–4 are checked by
        /// `debug_assert!` in debug builds; precondition 5 is not asserted
        /// here (it requires `T: PartialOrd`) and is covered transitively
        /// by callers that route through validated paths.
        #[inline]
        pub const fn new_assume_valid(left: OrdBound<T>, right: OrdBound<T>) -> Self {
            debug_assert!(
                !matches!(left, RightUnbounded),
                "OrdBoundPair: left must not be RightUnbounded"
            );
            debug_assert!(
                !matches!(right, LeftUnbounded) || matches!(left, LeftUnbounded),
                "OrdBoundPair: right must not be LeftUnbounded outside the canonical empty pair"
            );
            debug_assert!(
                !matches!(left, Finite(FiniteOrdBound(_, RightOpen))),
                "OrdBoundPair: left Finite must not be RightOpen"
            );
            debug_assert!(
                !matches!(right, Finite(FiniteOrdBound(_, LeftOpen))),
                "OrdBoundPair: right Finite must not be LeftOpen"
            );
            Self(left, right)
        }

        /// Decompose into the pair of OrdBounds
        pub fn into_raw(self) -> (OrdBound<T>, OrdBound<T>) {
            (self.0, self.1)
        }
    }

    impl<T: PartialOrd> OrdBoundPair<T> {
        /// Creates a new totally ordered bound pair.
        ///
        /// # Panics
        ///
        /// Panics if the inputs violate `OrdBoundPair` invariants. Use
        /// [`try_new`](Self::try_new) for the fallible variant.
        pub fn new(left: OrdBound<T>, right: OrdBound<T>) -> Self {
            Self::try_new(left, right).expect("OrdBoundPair invariants violated")
        }

        /// Creates a new totally ordered bound pair, returning `Err` on
        /// any structural or value-level invariant violation.
        ///
        /// Rejects:
        /// - `RightUnbounded` on the left;
        /// - `LeftUnbounded` on the right (except canonical empty);
        /// - finite-left with `RightOpen` kind;
        /// - finite-right with `LeftOpen` kind;
        /// - incomparable values (NaN) → [`TotalOrderError`](crate::error::TotalOrderError);
        /// - swapped value order (`left.value() > right.value()`).
        pub fn try_new(left: OrdBound<T>, right: OrdBound<T>) -> Result<Self, Error> {
            match (left, right) {
                // (LU, LU) is the canonical empty marker and the lowest element.
                (LeftUnbounded, LeftUnbounded) => Ok(Self::empty()),
                (left, right) => {
                    if matches!(&left, RightUnbounded)
                        || matches!(&right, LeftUnbounded)
                        || matches!(&left, Finite(FiniteOrdBound(_, RightOpen)))
                        || matches!(&right, Finite(FiniteOrdBound(_, LeftOpen)))
                    {
                        return Err(Error::InvalidBoundPair);
                    }
                    if let (Finite(FiniteOrdBound(lv, _)), Finite(FiniteOrdBound(rv, _))) =
                        (&left, &right)
                    {
                        // try_cmp raises TotalOrderError on NaN.
                        if lv.try_cmp(rv)? == Greater {
                            return Err(Error::InvalidBoundPair);
                        }
                    }
                    Ok(Self::new_assume_valid(left, right))
                }
            }
        }
    }

    impl From<FiniteOrdBoundKind> for BoundType {
        fn from(value: FiniteOrdBoundKind) -> Self {
            match value {
                FiniteOrdBoundKind::Closed => BoundType::Closed,
                FiniteOrdBoundKind::LeftOpen | FiniteOrdBoundKind::RightOpen => BoundType::Open,
            }
        }
    }

    /// Fallible conversion: a `FiniteOrdBound<T>` can carry any `T`
    /// (its Tier-4 constructors accept `NaN`/`±INF` on library float
    /// types), so reaching `FiniteBound<T>`'s validation invariant
    /// requires running `Element::validate` here. The `Side`
    /// orientation carried by `FiniteOrdBoundKind::{LeftOpen, RightOpen}`
    /// is discarded; `FiniteBound` only records `Open` vs `Closed`.
    impl<T: crate::numeric::Element> TryFrom<FiniteOrdBound<T>> for FiniteBound<T> {
        type Error = Error;

        fn try_from(value: FiniteOrdBound<T>) -> Result<Self, Self::Error> {
            FiniteBound::try_new(BoundType::from(value.1), value.0)
        }
    }
}
#[cfg(test)]
mod test {
    use ord::OrdBound;

    use super::Side::*;
    use super::*;
    use crate::try_cmp::TryCmp;

    mod ord_bound_pair {
        use crate::bound::ord::FiniteOrdBoundKind::*;
        use crate::bound::ord::OrdBound::*;
        use crate::bound::ord::{FiniteOrdBound, OrdBound, OrdBoundPair};
        use crate::error::Error;

        #[test]
        fn empty_round_trips_via_try_new() {
            let pair = OrdBoundPair::<i32>::try_new(LeftUnbounded, LeftUnbounded).unwrap();
            assert_eq!(pair, OrdBoundPair::<i32>::empty());
        }

        #[test]
        fn unbounded_pair_accepted() {
            OrdBoundPair::<i32>::try_new(LeftUnbounded, RightUnbounded).unwrap();
        }

        #[test]
        fn closed_equal_values_accepted() {
            OrdBoundPair::<i32>::try_new(
                OrdBound::closed_assume_valid(5),
                OrdBound::closed_assume_valid(5),
            )
            .unwrap();
        }

        #[test]
        fn rejects_right_unbounded_on_left() {
            let err =
                OrdBoundPair::<i32>::try_new(RightUnbounded, OrdBound::closed_assume_valid(0))
                    .unwrap_err();
            assert!(matches!(err, Error::InvalidBoundPair));
        }

        #[test]
        fn rejects_left_unbounded_on_right() {
            let err = OrdBoundPair::<i32>::try_new(OrdBound::closed_assume_valid(0), LeftUnbounded)
                .unwrap_err();
            assert!(matches!(err, Error::InvalidBoundPair));
        }

        #[test]
        fn rejects_right_open_kind_on_left() {
            let left = Finite(FiniteOrdBound(0_i32, RightOpen));
            let right = OrdBound::closed_assume_valid(10);
            let err = OrdBoundPair::try_new(left, right).unwrap_err();
            assert!(matches!(err, Error::InvalidBoundPair));
        }

        #[test]
        fn rejects_left_open_kind_on_right() {
            let left = OrdBound::closed_assume_valid(0);
            let right = Finite(FiniteOrdBound(10_i32, LeftOpen));
            let err = OrdBoundPair::try_new(left, right).unwrap_err();
            assert!(matches!(err, Error::InvalidBoundPair));
        }

        #[test]
        fn rejects_swapped_value_order() {
            let err = OrdBoundPair::<i32>::try_new(
                OrdBound::closed_assume_valid(10),
                OrdBound::closed_assume_valid(0),
            )
            .unwrap_err();
            assert!(matches!(err, Error::InvalidBoundPair));
        }

        #[test]
        fn rejects_nan_value() {
            let err = OrdBoundPair::<f32>::try_new(
                OrdBound::closed_assume_valid(f32::NAN),
                OrdBound::closed_assume_valid(f32::NAN),
            )
            .unwrap_err();
            assert!(matches!(err, Error::InvalidBoundLimit));
        }

        #[test]
        #[should_panic(expected = "OrdBoundPair invariants violated")]
        fn new_panics_on_malformed() {
            let _ = OrdBoundPair::<i32>::new(
                OrdBound::closed_assume_valid(10),
                OrdBound::closed_assume_valid(0),
            );
        }

        // Debug-mode tripwires on Tier 4 `new_assume_valid` bypass.
        // Compiled out in release; release behavior is "wrong answer, no UB."
        #[cfg(debug_assertions)]
        mod assume_valid_tripwires {
            use super::*;

            #[test]
            #[should_panic(expected = "left must not be RightUnbounded")]
            fn rejects_right_unbounded_on_left() {
                let _ = OrdBoundPair::<i32>::new_assume_valid(
                    RightUnbounded,
                    OrdBound::closed_assume_valid(0),
                );
            }

            #[test]
            #[should_panic(
                expected = "right must not be LeftUnbounded outside the canonical empty pair"
            )]
            fn rejects_left_unbounded_on_right() {
                let _ = OrdBoundPair::<i32>::new_assume_valid(
                    OrdBound::closed_assume_valid(0),
                    LeftUnbounded,
                );
            }

            #[test]
            #[should_panic(expected = "left Finite must not be RightOpen")]
            fn rejects_right_open_kind_on_left() {
                let _ = OrdBoundPair::<i32>::new_assume_valid(
                    Finite(FiniteOrdBound(0, RightOpen)),
                    OrdBound::closed_assume_valid(10),
                );
            }

            #[test]
            #[should_panic(expected = "right Finite must not be LeftOpen")]
            fn rejects_left_open_kind_on_right() {
                let _ = OrdBoundPair::<i32>::new_assume_valid(
                    OrdBound::closed_assume_valid(0),
                    Finite(FiniteOrdBound(10, LeftOpen)),
                );
            }
        }
    }

    #[test]
    fn test_bound_min_max() {
        assert_eq!(
            FiniteBound::min_assume_valid(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::min_assume_valid(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::open(0)
            ),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::max_assume_valid(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(10)
        );

        assert_eq!(
            FiniteBound::max_assume_valid(
                Side::Left,
                &FiniteBound::closed(0),
                &FiniteBound::open(0)
            ),
            &FiniteBound::open(0)
        );

        assert_eq!(
            FiniteBound::min_assume_valid(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(0)
        );

        assert_eq!(
            FiniteBound::min_assume_valid(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::open(0)
            ),
            &FiniteBound::open(0)
        );

        assert_eq!(
            FiniteBound::max_assume_valid(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::closed(10)
            ),
            &FiniteBound::closed(10)
        );

        assert_eq!(
            FiniteBound::max_assume_valid(
                Side::Right,
                &FiniteBound::closed(0),
                &FiniteBound::open(0)
            ),
            &FiniteBound::closed(0)
        )
    }

    #[test]
    fn test_partial_min_max() {
        let f0 = 0.0;
        let f1 = 100.0;

        assert_eq!(
            OrdBound::closed_assume_valid(f0).try_min(OrdBound::closed_assume_valid(f1)),
            Ok(OrdBound::closed_assume_valid(f0))
        );

        assert_eq!(
            OrdBound::closed_assume_valid(&f0).try_min(OrdBound::closed_assume_valid(&f1)),
            Ok(OrdBound::closed_assume_valid(&f0))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed_assume_valid(f1)),
            Ok(OrdBound::closed_assume_valid(f1))
        );

        assert_eq!(
            OrdBound::LeftUnbounded.try_max(OrdBound::closed_assume_valid(&f1)),
            Ok(OrdBound::closed_assume_valid(&f1))
        )
    }

    #[test]
    pub fn test_try_contains() {
        let x = FiniteBound::closed(0.0);

        assert!(x.try_contains(Left, &0.0).unwrap());
        assert!(x.try_contains(Left, &1.0).unwrap());
        assert!(!x.try_contains(Left, &-1.0).unwrap());
        assert!(x.try_contains(Left, &f64::NAN).is_err());

        assert!(x.try_contains(Right, &0.0).unwrap());
        assert!(x.try_contains(Right, &-1.0).unwrap());
        assert!(!x.try_contains(Right, &1.0).unwrap());
        assert!(x.try_contains(Right, &f64::NAN).is_err());

        let open = FiniteBound::open(0.0);

        assert!(!open.try_contains(Left, &0.0).unwrap());
        assert!(open.try_contains(Left, &1.0).unwrap());
        assert!(!open.try_contains(Left, &-1.0).unwrap());
        assert!(open.try_contains(Left, &f64::NAN).is_err());

        assert!(!open.try_contains(Right, &0.0).unwrap());
        assert!(open.try_contains(Right, &-1.0).unwrap());
        assert!(!open.try_contains(Right, &1.0).unwrap());
        assert!(open.try_contains(Right, &f64::NAN).is_err());
    }

    #[test]
    fn test_min_max() {
        let a = FiniteBound::closed(0.0);
        let b = FiniteBound::open(0.0);

        assert_eq!(FiniteBound::min_max_assume_valid(Side::Left, a, b), (a, b));

        assert_eq!(FiniteBound::min_max_assume_valid(Side::Left, b, a), (a, b));

        assert_eq!(FiniteBound::min_max_assume_valid(Side::Right, a, b), (b, a));

        assert_eq!(FiniteBound::min_max_assume_valid(Side::Right, b, a), (b, a))
    }

    mod try_new_validates_limit {
        use super::*;
        use crate::error::Error;

        #[test]
        fn rejects_positive_infinity_f64() {
            let r = FiniteBound::<f64>::try_closed(f64::INFINITY);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));

            let r = FiniteBound::<f64>::try_open(f64::INFINITY);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));

            let r = FiniteBound::<f64>::try_new(BoundType::Closed, f64::INFINITY);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn rejects_negative_infinity_f64() {
            let r = FiniteBound::<f64>::try_closed(f64::NEG_INFINITY);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn rejects_nan_f64() {
            let r = FiniteBound::<f64>::try_closed(f64::NAN);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));
        }

        #[test]
        fn rejects_non_finite_f32() {
            assert!(matches!(
                FiniteBound::<f32>::try_closed(f32::INFINITY),
                Err(Error::InvalidBoundLimit)
            ));
            assert!(matches!(
                FiniteBound::<f32>::try_closed(f32::NEG_INFINITY),
                Err(Error::InvalidBoundLimit)
            ));
            assert!(matches!(
                FiniteBound::<f32>::try_closed(f32::NAN),
                Err(Error::InvalidBoundLimit)
            ));
        }

        #[test]
        fn accepts_finite_f64() {
            assert_eq!(
                FiniteBound::<f64>::try_closed(0.0).unwrap(),
                FiniteBound::closed(0.0)
            );
            assert_eq!(
                FiniteBound::<f64>::try_open(-1.5).unwrap(),
                FiniteBound::open(-1.5)
            );
        }

        #[test]
        fn default_validate_accepts_integers() {
            assert_eq!(
                FiniteBound::<i64>::try_closed(5).unwrap(),
                FiniteBound::closed(5)
            );
            assert_eq!(
                FiniteBound::<i32>::try_new(BoundType::Open, -100).unwrap(),
                FiniteBound::open(-100)
            );
        }

        #[test]
        fn factory_paths_reject_infinity() {
            use crate::factory::TryFiniteFactory;
            use crate::sets::FiniteInterval;

            let r = FiniteInterval::<f64>::try_closed(0.0, f64::INFINITY);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));

            let r = FiniteInterval::<f64>::try_open(f64::NEG_INFINITY, 0.0);
            assert!(matches!(r, Err(Error::InvalidBoundLimit)));
        }
    }
}

use core::cmp::Ordering::{self, *};

use num_traits::One;

use super::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
use super::bound::Side::{self, Left, Right};
use super::bound::{FiniteBound, SetBounds};
use crate::bound::ord::FiniteOrdBound;
use crate::error::{Error, TotalOrderError};
use crate::factory::FiniteFactory;
use crate::numeric::{Element, Zero};
use crate::try_cmp::TryCmp;

/// Internal storage for [`FiniteInterval`]: either empty or a pair
/// of finite bounds `(lhs, rhs)` with `lhs <= rhs`.
///
/// `Deserialize` is intentionally **not** derived: validation is performed
/// by [`FiniteInterval`]'s `try_from` proxy so that no path produces an
/// unvalidated inner. `Serialize` is derived because the outer type's
/// writer path delegates here.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
enum FiniteIntervalInner<T> {
    Empty,
    Bounded(FiniteBound<T>, FiniteBound<T>),
}

impl<T> OrdBounded<T> for FiniteIntervalInner<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Empty => OrdBoundPair::empty(),
            Self::Bounded(lhs, rhs) => {
                // Bounded is a validated FiniteInterval pair: invariants hold.
                OrdBoundPair::new_assume_valid(lhs.ord(Side::Left), rhs.ord(Side::Right))
            }
        }
    }
}

impl<T> SetBounds<T> for FiniteIntervalInner<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        match self {
            Self::Bounded(lhs, rhs) => Some(side.select(lhs, rhs)),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawFiniteInterval<T>"))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "T: Element + serde::Deserialize<'de>")))]
pub struct FiniteInterval<T>(FiniteIntervalInner<T>);

/// Wire-format mirror of [`FiniteInterval`] used to drive validation
/// during `Deserialize`. Identical layout, no invariants.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "FiniteInterval")]
struct RawFiniteInterval<T>(RawFiniteIntervalInner<T>);

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "FiniteIntervalInner")]
enum RawFiniteIntervalInner<T> {
    Empty,
    Bounded(FiniteBound<T>, FiniteBound<T>),
}

#[cfg(feature = "serde")]
impl<T: Element> TryFrom<RawFiniteInterval<T>> for FiniteInterval<T> {
    type Error = Error;

    fn try_from(raw: RawFiniteInterval<T>) -> Result<Self, Self::Error> {
        match raw.0 {
            RawFiniteIntervalInner::Empty => Ok(Self::empty()),
            RawFiniteIntervalInner::Bounded(lhs, rhs) => {
                let result = Self::try_new(lhs, rhs)?;
                if result.is_empty() {
                    // try_new normalizes a swapped-order pair to empty;
                    // deserialize is strict and rejects malformed input.
                    Err(Error::InvalidBoundPair)
                } else {
                    Ok(result)
                }
            }
        }
    }
}

impl<T: Element> FiniteInterval<T> {
    /// Creates a FiniteInterval.
    ///
    /// # Panics
    ///
    /// Panics if lhs and rhs are not comparable
    pub fn new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        Self::try_new(lhs, rhs).unwrap()
    }

    /// Creates a new FiniteInterval or Error; Should never panic.
    pub fn try_new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self, Error> {
        let lhs = lhs.normalized(Left);
        let rhs = rhs.normalized(Right);
        let order = lhs.value().try_cmp(rhs.value())?;

        if order == Less || (order == Equal && lhs.is_closed() && rhs.is_closed()) {
            // normalized & comparable & lhs <= rhs
            Ok(Self::new_assume_valid(lhs, rhs))
        } else {
            Ok(Self::empty())
        }
    }
}

impl<T: PartialOrd> FiniteInterval<T> {
    /// Creates a FiniteInterval; assuming normalized & comparable.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be normalized (discrete types in closed form) and
    /// comparable. Violating this yields incorrect results but no
    /// undefined behavior.
    #[inline]
    pub fn new_assume_normed(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        debug_assert!(lhs.value().partial_cmp(rhs.value()).is_some());
        if lhs.value() < rhs.value()
            || (lhs.value() == rhs.value() && lhs.is_closed() && rhs.is_closed())
        {
            Self::new_assume_valid(lhs, rhs)
        } else {
            Self::empty()
        }
    }

    /// Creates a FiniteInterval; assumes normalized.
    ///
    /// # Preconditions
    ///
    /// Both bounds must be normalized. Bounds checking is done via
    /// [`TryCmp`] but may not be correct if not normalized. No undefined
    /// behavior on violation.
    #[inline(always)]
    pub fn try_new_assume_normed(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self, Error> {
        let order = lhs.value().try_cmp(rhs.value())?;
        if order == Less || (order == Equal && lhs.is_closed() && rhs.is_closed()) {
            Ok(Self::new_assume_valid(lhs, rhs))
        } else {
            Ok(Self::empty())
        }
    }
}

impl<T> FiniteInterval<T> {
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(FiniteIntervalInner::Empty)
    }

    /// Constructs without checking invariants.
    ///
    /// # Preconditions
    ///
    /// 1. lhs <= rhs
    /// 2. discrete bounds are normalized to closed form.
    ///
    /// Violating either yields incorrect results but no undefined behavior.
    #[inline]
    pub const fn new_assume_valid(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        Self(FiniteIntervalInner::Bounded(lhs, rhs))
    }

    #[inline]
    pub fn into_raw(self) -> Option<(FiniteBound<T>, FiniteBound<T>)> {
        match self.0 {
            FiniteIntervalInner::Bounded(lhs, rhs) => Some((lhs, rhs)),
            FiniteIntervalInner::Empty => None,
        }
    }

    #[inline]
    pub fn view_raw(&self) -> Option<(&FiniteBound<T>, &FiniteBound<T>)> {
        match self.0 {
            FiniteIntervalInner::Bounded(ref lhs, ref rhs) => Some((lhs, rhs)),
            FiniteIntervalInner::Empty => None,
        }
    }
}

impl<T> FiniteInterval<T> {
    pub fn is_empty(&self) -> bool {
        core::mem::discriminant(&self.0) == core::mem::discriminant(&FiniteIntervalInner::Empty)
    }

    pub fn is_fully_bounded(&self) -> bool {
        !self.is_empty()
    }
}

impl<T> OrdBounded<T> for FiniteInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        self.0.ord_bound_pair()
    }
}

impl<T> SetBounds<T> for FiniteInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        self.0.bound(side)
    }
}

/// An interval bounded on exactly one side. The `side` field marks
/// which end is finite; the other end is implicitly unbounded.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawHalfInterval<T>"))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "T: Element + serde::Deserialize<'de>")))]
pub struct HalfInterval<T> {
    side: Side,
    bound: FiniteBound<T>,
}

/// Wire-format mirror of [`HalfInterval`] used to drive validation
/// during `Deserialize`.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "HalfInterval")]
struct RawHalfInterval<T> {
    side: Side,
    bound: FiniteBound<T>,
}

#[cfg(feature = "serde")]
impl<T: Element> TryFrom<RawHalfInterval<T>> for HalfInterval<T> {
    type Error = Error;

    fn try_from(raw: RawHalfInterval<T>) -> Result<Self, Self::Error> {
        Self::try_new(raw.side, raw.bound)
    }
}

impl<T> HalfInterval<T> {
    /// Creates a new half interval without checking invariants.
    ///
    /// # Preconditions
    ///
    /// `bound` must be comparable (e.g., a non-NaN float). This is
    /// assumed if the bound is taken from an existing set. Violating
    /// this yields incorrect results but no undefined behavior.
    pub const fn new_assume_valid(side: Side, bound: FiniteBound<T>) -> Self {
        Self { side, bound }
    }
}

impl<T: Element> HalfInterval<T> {
    pub fn new(side: Side, bound: FiniteBound<T>) -> Self {
        Self::try_new(side, bound).expect("Bound should have been comparable")
    }

    pub fn try_new(side: Side, bound: FiniteBound<T>) -> Result<Self, Error> {
        // probe comparability without requiring T: Zero - a value compared to
        // itself is Some(Equal) for any properly-ordered type and None for NaN.
        let _ = bound
            .value()
            .partial_cmp(bound.value())
            .ok_or(TotalOrderError)?;
        let bound = bound.normalized(side);
        Ok(Self { side, bound })
    }

    pub fn left(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Left, bound)
    }

    pub fn right(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Right, bound)
    }
}

impl<T> HalfInterval<T> {
    #[inline(always)]
    pub fn into_raw(self) -> (Side, FiniteBound<T>) {
        (self.side, self.bound)
    }

    pub fn into_finite_ord_bound(self) -> FiniteOrdBound<T> {
        self.bound.into_finite_ord(self.side)
    }

    pub fn into_ord_bound(self) -> OrdBound<T> {
        self.bound.into_ord(self.side)
    }

    pub fn finite_ord_bound(&self) -> FiniteOrdBound<&T> {
        self.bound.finite_ord(self.side)
    }

    pub fn ord_bound(&self) -> OrdBound<&T> {
        self.bound.ord(self.side)
    }

    #[inline(always)]
    pub fn side(&self) -> Side {
        self.side
    }

    /// Returns the finite bound of the HalfBounded interval.
    #[inline(always)]
    pub fn finite_bound(&self) -> &FiniteBound<T> {
        &self.bound
    }
}

impl<T> OrdBounded<T> for HalfInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self.side {
            Side::Left => {
                let left = OrdBound::left(&self.bound);
                OrdBoundPair::new_assume_valid(left, OrdBound::RightUnbounded)
            }
            Side::Right => {
                let right = OrdBound::right(&self.bound);
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, right)
            }
        }
    }
}

impl<T> SetBounds<T> for HalfInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        if self.side == side {
            Some(&self.bound)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawEnumInterval<T>"))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "T: Element + serde::Deserialize<'de>")))]
#[allow(missing_docs)]
pub enum EnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

/// Wire-format mirror of [`EnumInterval`]. The variants hold the
/// already-validated public types, so the `TryFrom` is total.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "EnumInterval")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
enum RawEnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

#[cfg(feature = "serde")]
impl<T: Element> From<RawEnumInterval<T>> for EnumInterval<T> {
    fn from(raw: RawEnumInterval<T>) -> Self {
        match raw {
            RawEnumInterval::Finite(inner) => Self::Finite(inner),
            RawEnumInterval::Half(inner) => Self::Half(inner),
            RawEnumInterval::Unbounded => Self::Unbounded,
        }
    }
}

impl<T> EnumInterval<T> {
    /// Creates a new empty EnumInterval.
    pub const fn empty() -> Self {
        Self::Finite(FiniteInterval::empty())
    }
}

impl<T> EnumInterval<T> {
    pub fn is_fully_bounded(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_fully_bounded(),
            _ => false,
        }
    }
}

impl<T> OrdBounded<T> for EnumInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Finite(inner) => inner.ord_bound_pair(),
            Self::Half(inner) => inner.ord_bound_pair(),
            Self::Unbounded => {
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
            }
        }
    }
}

impl<T> SetBounds<T> for EnumInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        match self {
            Self::Finite(inner) => inner.bound(side),
            Self::Half(inner) => inner.bound(side),
            Self::Unbounded => None,
        }
    }
}

// num_traits::Zero requires Self: Add<Self, Output = Self>; the infix
// Add impls on FiniteInterval/EnumInterval require T: Ord. Likewise One
// requires Self: Mul<Self, Output = Self>, so T must satisfy the Mul
// bounds too (Ord + Clone + Zero).
impl<T: Element + Ord + Zero> Zero for FiniteInterval<T> {
    fn zero() -> Self {
        Self::closed(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        let zero = T::zero();
        self.lval() == Some(&zero) && self.rval() == Some(&zero)
    }
}

impl<T: Element + Ord + Zero> Zero for EnumInterval<T> {
    fn zero() -> Self {
        Self::from(FiniteInterval::<T>::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_zero(),
            _ => false,
        }
    }
}

impl<T: Element + Ord + Clone + Zero + One> One for FiniteInterval<T> {
    fn one() -> Self {
        FiniteInterval::closed(T::one(), T::one())
    }
}

impl<T: Element + Ord + Clone + Zero + One> One for EnumInterval<T> {
    fn one() -> Self {
        EnumInterval::from(FiniteInterval::one())
    }
}

impl<T> Default for FiniteInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Default for EnumInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

macro_rules! impl_interval_cmp {
    ($($t:ident), +) => {
        $(
            impl<T: PartialOrd> PartialOrd for $t<T> {
                fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
                    let lhs = self.ord_bound_pair();
                    let rhs = rhs.ord_bound_pair();
                    lhs.partial_cmp(&rhs)
                }
            }

            impl<T: Ord> Ord for $t<T> {
                fn cmp(&self, rhs: &Self) -> Ordering {
                    let lhs = self.ord_bound_pair();
                    let rhs = rhs.ord_bound_pair();
                    lhs.cmp(&rhs)
                }
            }
        )+
    }
}

impl_interval_cmp!(FiniteIntervalInner, HalfInterval, EnumInterval);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn test_set_bounds_trait() {
        let x = EnumInterval::closed(0, 10);

        assert_eq!(x.left().unwrap(), &FiniteBound::closed(0));
        assert_eq!(x.right().unwrap(), &FiniteBound::closed(10));
    }

    #[test]
    fn test_ord_bounded_trait() {
        let x = EnumInterval::closed(0, 10);

        fn by_ref(y: &EnumInterval<i32>) {
            let ob = y.ord_bound_pair();
            assert_eq!(
                ob,
                OrdBoundPair::new(OrdBound::closed(&0), OrdBound::closed(&10))
            );
        }

        fn by_val(y: EnumInterval<i32>) {
            let ob = y.ord_bound_pair();
            assert_eq!(
                ob,
                OrdBoundPair::new(OrdBound::closed(&0), OrdBound::closed(&10))
            );
        }

        by_ref(&x);
        by_val(x);
    }
}

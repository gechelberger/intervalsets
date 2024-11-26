use core::cmp::Ordering::{self, *};

use super::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
use super::bound::Side::{self, Left, Right};
use super::bound::{FiniteBound, SetBounds};
use crate::bound::ord::FiniteOrdBound;
use crate::error::{Error, TotalOrderError};
use crate::numeric::{Element, Zero};
use crate::try_cmp::TryCmp;

/// todo
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
enum FiniteIntervalInner<T> {
    Empty,
    Bounded(FiniteBound<T>, FiniteBound<T>),
}

impl<T> OrdBounded<T> for FiniteIntervalInner<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Empty => OrdBoundPair::empty(),
            Self::Bounded(lhs, rhs) => OrdBoundPair::new(lhs.ord(Side::Left), rhs.ord(Side::Right)),
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
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct FiniteInterval<T>(FiniteIntervalInner<T>);

impl<T: Element> FiniteInterval<T> {
    /// Creates a FiniteInterval.
    ///
    /// # Panics
    ///
    /// Panics if lhs and rhs are not comparable
    pub fn new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        let lhs = lhs.normalized(Left);
        let rhs = rhs.normalized(Right);
        unsafe { Self::new_assume_normed(lhs, rhs) }
    }

    /// Creates a new FiniteInterval or Error; Should never panic.
    pub fn new_strict(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self, Error> {
        let lhs = lhs.normalized(Left);
        let rhs = rhs.normalized(Right);
        let order = lhs
            .value()
            .partial_cmp(rhs.value())
            .ok_or(TotalOrderError)?;

        if order == Less || (order == Equal && lhs.is_closed() && rhs.is_closed()) {
            // SAFETY: normalized & comparable & lhs <= rhs
            unsafe { Ok(Self::new_unchecked(lhs, rhs)) }
        } else {
            Ok(Self::empty())
        }
    }
}

impl<T: PartialOrd> FiniteInterval<T> {
    /// Creates a FiniteInterval; assuming normalized & comparable.
    ///
    /// # Panics
    ///
    /// Panics if lhs and rhs are not comparable.
    ///
    /// # Safety
    ///
    /// The user is responsible for ensuring that invariants are satisfied.
    #[inline]
    pub unsafe fn new_assume_normed(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        match Self::new_strict_assume_normed(lhs, rhs) {
            Ok(finterval) => finterval,
            Err(e) => panic!("assumed normalized and comparable: {}", e),
        }
    }

    /// Creates a FiniteInterval; assumes normalized.
    ///
    /// # Safety
    ///
    /// The user is responsible for ensuring that bounds are normed.
    /// Bounds checking is done but may not be correct if not normed.
    #[inline(always)]
    pub unsafe fn new_strict_assume_normed(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self, Error> {
        let order = lhs.value().try_cmp(rhs.value())?;
        if order == Less || (order == Equal && lhs.is_closed() && rhs.is_closed()) {
            Ok(Self::new_unchecked(lhs, rhs))
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

    /// # Safety
    ///
    /// The user must ensure invariants are satisfied:
    /// 1. lhs <= rhs
    /// 2. discrete bounds are normalized to closed form.
    #[inline]
    pub const unsafe fn new_unchecked(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
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

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     self.map(|lhs, rhs| (Some(lhs), Some(rhs))).ok()
    // }
}

/// todo...
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct HalfInterval<T> {
    side: Side,
    bound: FiniteBound<T>,
}

impl<T> HalfInterval<T> {
    /// Creates a new half interval without checking invariants.
    ///
    /// # Safety
    ///
    /// The user is responsible for ensuring that `bound` is comparable. This
    /// is assumed if the bound is taken from an existing set.
    pub const unsafe fn new_unchecked(side: Side, bound: FiniteBound<T>) -> Self {
        Self { side, bound }
    }
}

impl<T: Element + Zero> HalfInterval<T> {
    pub fn new(side: Side, bound: FiniteBound<T>) -> Self {
        Self::new_strict(side, bound).expect("Bound should have been comparable")
    }

    pub fn new_strict(side: Side, bound: FiniteBound<T>) -> Result<Self, Error> {
        // make sure bound is comparable
        let _ = T::zero()
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
                OrdBoundPair::new(left, OrdBound::RightUnbounded)
            }
            Side::Right => {
                let right = OrdBound::right(&self.bound);
                OrdBoundPair::new(OrdBound::LeftUnbounded, right)
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

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     let bounds = match self.side {
    //         Side::Left => (Some(self.bound), None),
    //         Side::Right => (None, Some(self.bound)),
    //     };
    //     Some(bounds)
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[allow(missing_docs)]
pub enum EnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
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
            Self::Unbounded => OrdBoundPair::new(OrdBound::LeftUnbounded, OrdBound::RightUnbounded),
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

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     match self {
    //         Self::Finite(inner) => inner.into_bounds(),
    //         Self::Half(inner) => inner.into_bounds(),
    //         Self::Unbounded => Some((None, None)),
    //     }
    // }
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

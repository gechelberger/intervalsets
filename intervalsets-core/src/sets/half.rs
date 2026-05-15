use core::cmp::Ordering;

use crate::bound::ord::{FiniteOrdBound, OrdBound, OrdBoundPair, OrdBounded};
use crate::bound::{FiniteBound, SetBounds, Side};
use crate::error::Error;
use crate::numeric::Element;

/// An interval bounded on exactly one side. The `side` field marks
/// which end is finite; the other end is implicitly unbounded.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawHalfInterval<T>"))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))
)]
pub struct HalfInterval<T> {
    side: Side,
    bound: FiniteBound<T>,
}

/// Wire-format mirror of [`HalfInterval`] used to drive validation
/// during `Deserialize`.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "HalfInterval")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
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

impl<T: Element> HalfInterval<T> {
    /// Creates a `HalfInterval`. **Strict** — panics if the bound's
    /// value is rejected by
    /// [`Element::validate`]
    /// (NaN / ±INF on library float types). Discrete bounds are
    /// normalized to closed form.
    ///
    /// `HalfInterval` has only one bound; there is no pair invariant,
    /// so strict and coercive degenerate to the same behavior here.
    pub fn new(side: Side, bound: FiniteBound<T>) -> Self {
        Self::try_new(side, bound).expect("Bound should have been comparable")
    }

    /// Fallible strict construction: returns `Err` for invalid bound
    /// limits. Discrete bounds are normalized to closed form.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidElement`] —
    ///   bound value is incomparable (e.g. NaN).
    pub fn try_new(side: Side, bound: FiniteBound<T>) -> Result<Self, Error> {
        let bound = bound.normalized(side);
        Ok(Self { side, bound })
    }

    /// Constructs a left-bounded `HalfInterval` `[a, ..)` or `(a, ..)`.
    /// Panics on invalid bound limit. Convenience for
    /// `HalfInterval::new(Side::Left, bound)`.
    pub fn left(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Left, bound)
    }

    /// Constructs a right-bounded `HalfInterval` `(.., b]` or `(.., b)`.
    /// Panics on invalid bound limit. Convenience for
    /// `HalfInterval::new(Side::Right, bound)`.
    pub fn right(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Right, bound)
    }

    /// Constructs without checking invariants. Tier 4 bypass.
    ///
    /// # Preconditions
    ///
    /// 1. **I2** — the bound's value is comparable (no NaN).
    /// 2. **I4** — for discrete `T`, the bound is in closed form.
    ///
    /// Violating either yields incorrect results but no undefined
    /// behavior. Debug builds trip `debug_assert!` tripwires; release
    /// builds do no checking.
    ///
    /// `#[doc(hidden)]` — maintainer-context only.
    #[doc(hidden)]
    pub fn new_assume_valid(side: Side, bound: FiniteBound<T>) -> Self {
        debug_assert!(
            bound.value().partial_cmp(bound.value()).is_some(),
            "I2: bound value must be comparable (NaN check)"
        );
        debug_assert!(
            bound.is_closed() || bound.value().try_adjacent(side.flip()).is_none(),
            "I4: bound must be discrete-normalized to closed"
        );
        Self { side, bound }
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

impl<T: PartialOrd> PartialOrd for HalfInterval<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.partial_cmp(&rhs)
    }
}

impl<T: Ord> Ord for HalfInterval<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.cmp(&rhs)
    }
}

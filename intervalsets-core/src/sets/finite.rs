use core::cmp::Ordering::{self, *};

use num_traits::One;

use crate::bound::ord::{OrdBoundPair, OrdBounded};
use crate::bound::Side::{self, Left, Right};
use crate::bound::{FiniteBound, SetBounds};
use crate::error::Error;
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
pub(super) enum FiniteIntervalInner<T> {
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
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))
)]
pub struct FiniteInterval<T>(FiniteIntervalInner<T>);

/// Wire-format mirror of [`FiniteInterval`] used to drive validation
/// during `Deserialize`. Identical layout, no invariants.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "FiniteInterval")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
struct RawFiniteInterval<T>(RawFiniteIntervalInner<T>);

#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "FiniteIntervalInner")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
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
            // try_new is strict about crossed bounds, which is what
            // we want here — deserialize never legitimately receives
            // a Bounded with lhs > rhs.
            RawFiniteIntervalInner::Bounded(lhs, rhs) => Self::try_new(lhs, rhs),
        }
    }
}

impl<T: Element> FiniteInterval<T> {
    /// Creates a `FiniteInterval`. **Strict** — panics on malformed
    /// input. Discrete bounds are normalized to closed form first;
    /// after normalization, crossed bounds (`lhs > rhs`), or open-open
    /// at the same point, panic.
    ///
    /// # Panics
    ///
    /// Panics if either bound's value is rejected by
    /// [`Element::validate`]
    /// (NaN / ±INF on library float types), or if the normalized
    /// pair is not a non-empty `Bounded`.
    ///
    /// For coercive ("crossed → `Empty`") semantics, use
    /// [`SatisfyFiniteInterval::satisfy_bounds`](crate::factory::SatisfyFiniteInterval::satisfy_bounds).
    pub fn new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        Self::try_new(lhs, rhs).unwrap()
    }

    /// Strict validating constructor: returns `Err` for any malformed
    /// input. Discrete bounds are normalized to closed form before
    /// validation.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidElement`] —
    ///   a bound value is incomparable (e.g. NaN).
    /// - [`Error::InvalidBoundPair`] —
    ///   after normalization, the pair is not a non-empty `Bounded`
    ///   (`lhs > rhs`, or open-open at the same point).
    ///
    /// For coercive semantics — return `Empty` on crossed input —
    /// use
    /// [`TrySatisfyFiniteInterval::try_satisfy_bounds`](crate::factory::TrySatisfyFiniteInterval::try_satisfy_bounds).
    pub fn try_new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self, Error> {
        let lhs = lhs.normalized(Left);
        let rhs = rhs.normalized(Right);
        let order = lhs.value().try_cmp(rhs.value())?;

        if order == Less || (order == Equal && lhs.is_closed() && rhs.is_closed()) {
            // normalized & comparable & lhs <= rhs
            Ok(Self::new_assume_valid(lhs, rhs))
        } else {
            Err(Error::InvalidBoundPair)
        }
    }
}

impl<T: Element> FiniteInterval<T> {
    /// Constructs without checking invariants. Tier 4 bypass.
    ///
    /// # Preconditions
    ///
    /// 1. **I2** — each bound's value is comparable (no NaN).
    /// 2. **I4** — discrete bounds are normalized to closed form.
    /// 3. **I5** — `lhs.value() <= rhs.value()`, with equality
    ///    requiring both bounds to be `Closed`.
    ///
    /// Violating any yields incorrect results but no undefined
    /// behavior. Debug builds trip `debug_assert!` tripwires for
    /// each precondition; release builds do no checking.
    ///
    /// `#[doc(hidden)]` because this is maintainer-context only —
    /// callers reach it via paths that already go through
    /// normalizing constructors like [`try_new`](Self::try_new).
    /// Embedded users that want a guaranteed panic-free release path
    /// can rely on this constructor's release-mode no-op contract.
    #[doc(hidden)]
    #[inline]
    pub fn new_assume_valid(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        debug_assert!(
            lhs.value().partial_cmp(rhs.value()).is_some(),
            "I2: bounds must be comparable (NaN check)"
        );
        debug_assert!(
            lhs.is_closed() || lhs.value().try_adjacent(Side::Right).is_none(),
            "I4: lhs must be discrete-normalized to closed"
        );
        debug_assert!(
            rhs.is_closed() || rhs.value().try_adjacent(Side::Left).is_none(),
            "I4: rhs must be discrete-normalized to closed"
        );
        debug_assert!(
            lhs.value() < rhs.value()
                || (lhs.value() == rhs.value() && lhs.is_closed() && rhs.is_closed()),
            "I5: bounds must satisfy lhs <= rhs (closed-closed at equality)"
        );
        Self(FiniteIntervalInner::Bounded(lhs, rhs))
    }
}

impl<T> FiniteInterval<T> {
    #[inline(always)]
    pub const fn empty() -> Self {
        Self(FiniteIntervalInner::Empty)
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

// num_traits::Zero requires Self: Add<Self, Output = Self>; the infix
// Add impl on FiniteInterval is sugar over try_add, so T must satisfy
// `TryAdd<Output = T>`. Likewise One requires Self: Mul<Self, Output = Self>,
// so T must satisfy `TryMul<Output = T>`.
impl<T> Zero for FiniteInterval<T>
where
    T: Element + Zero + crate::ops::math::TryAdd<Output = T>,
    <T as crate::ops::math::TryAdd>::Error: core::fmt::Debug + Into<Error>,
{
    fn zero() -> Self {
        Self::closed(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        let zero = T::zero();
        self.lval() == Some(&zero) && self.rval() == Some(&zero)
    }
}

impl<T> One for FiniteInterval<T>
where
    T: Element + Clone + Zero + One + crate::ops::math::TryMul<Output = T>,
    <T as crate::ops::math::TryMul>::Error: core::fmt::Debug + Into<Error>,
{
    fn one() -> Self {
        FiniteInterval::closed(T::one(), T::one())
    }
}

impl<T> Default for FiniteInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

// PartialOrd/Ord delegate to OrdBoundPair. Defined on the inner because
// the outer derives them via the inner's layout.
impl<T: PartialOrd> PartialOrd for FiniteIntervalInner<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.partial_cmp(&rhs)
    }
}

impl<T: Ord> Ord for FiniteIntervalInner<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.cmp(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod try_new_strict {
        use super::*;

        #[test]
        fn try_new_errors_on_crossed_continuous() {
            let result =
                FiniteInterval::try_new(FiniteBound::closed(10.0_f32), FiniteBound::closed(0.0));
            assert!(matches!(result, Err(Error::InvalidBoundPair)));
        }

        #[test]
        fn try_new_errors_on_crossed_discrete_after_normalization() {
            // open(10, 10) for i32 normalizes to closed(11, 9) which is crossed.
            let result = FiniteInterval::try_new(FiniteBound::open(10_i32), FiniteBound::open(10));
            assert!(matches!(result, Err(Error::InvalidBoundPair)));
        }

        #[test]
        fn try_satisfy_bounds_returns_empty_on_crossed() {
            use crate::factory::TrySatisfyFiniteInterval;
            let result = FiniteInterval::try_satisfy_bounds(
                FiniteBound::closed(10.0_f32),
                FiniteBound::closed(0.0),
            )
            .unwrap();
            assert_eq!(result, FiniteInterval::empty());
        }

        #[test]
        #[should_panic(expected = "InvalidBoundPair")]
        fn new_panics_on_crossed() {
            let _ = FiniteInterval::new(FiniteBound::closed(10_i32), FiniteBound::closed(0));
        }

        #[test]
        #[should_panic]
        fn factory_open_panics_on_crossed() {
            // Factory is strict-by-default: crossed bounds panic.
            // For coercive semantics, use SatisfyFiniteInterval::satisfy_bounds.
            let _ = FiniteInterval::<f32>::open(10.0, 0.0);
        }

        #[test]
        fn satisfy_bounds_returns_empty_on_crossed() {
            use crate::factory::SatisfyFiniteInterval;
            let x =
                FiniteInterval::satisfy_bounds(FiniteBound::open(10.0_f32), FiniteBound::open(0.0));
            assert_eq!(x, FiniteInterval::empty());
        }

        #[test]
        #[allow(clippy::reversed_empty_ranges)]
        fn from_reversed_range_is_empty() {
            // Rust's Range semantics: reversed → iterates nothing.
            // The From impl preserves that.
            let x: FiniteInterval<i32> = (10..0).into();
            assert_eq!(x, FiniteInterval::empty());

            let x: FiniteInterval<i32> = (10..=0).into();
            assert_eq!(x, FiniteInterval::empty());
        }
    }

    /// Debug-mode tripwires on Tier 4 `*_assume_valid` bypass.
    ///
    /// Each test constructs a deliberately invariant-violating input
    /// and confirms the corresponding `debug_assert!` panics. Gated
    /// to `cfg(debug_assertions)` because the asserts are compiled
    /// out in release; release behavior is exercised by the
    /// `#[cfg(not(debug_assertions))]` tests in `category.rs`.
    #[cfg(debug_assertions)]
    mod assume_valid_tripwires {
        use super::*;

        #[test]
        #[should_panic(expected = "lhs <= rhs")]
        fn finite_interval_new_assume_valid_panics_on_crossed() {
            let _ = FiniteInterval::new_assume_valid(
                FiniteBound::closed(10_i32),
                FiniteBound::closed(0_i32),
            );
        }

        #[test]
        #[should_panic(expected = "lhs <= rhs")]
        fn finite_interval_new_assume_valid_panics_on_equal_open() {
            // (5.0, 5.0) violates the closed-closed-at-equality clause.
            // Use f32 (continuous) so the I4 normalization tripwire
            // doesn't fire first — open is legitimate for continuous T.
            let _ = FiniteInterval::new_assume_valid(
                FiniteBound::open(5.0_f32),
                FiniteBound::open(5.0_f32),
            );
        }
    }
}

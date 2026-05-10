//! Factory traits for constructing intervals.
//!
//! Each interval type ([`FiniteInterval`], [`HalfInterval`],
//! [`EnumInterval`]) implements the relevant factory traits directly;
//! there are no parameterized factory marker types. Pull the traits
//! into scope via [`traits`] and call associated functions:
//!
//! ```
//! use intervalsets_core::prelude::*;
//!
//! let x = EnumInterval::<u32>::closed(0, 10);
//! let y = EnumInterval::<u32>::closed(5, 15);
//! assert_eq!(x.intersection(y), EnumInterval::closed(5, 10));
//! ```
//!
//! Wrapped storage types (`OrderedFloat<f32>`, `NotNan<f32>`,
//! `BigDecimal`, etc.) are constructed at the call site and passed
//! through directly:
//!
//! ```ignore
//! use intervalsets_core::prelude::*;
//! use ordered_float::NotNan;
//!
//! let x = EnumInterval::open(
//!     NotNan::<f32>::new(0.0).unwrap(),
//!     NotNan::<f32>::new(10.0).unwrap(),
//! );
//! ```
//!
//! # Trait shape
//!
//! The fallible / panicking pair on `FiniteFactory` and
//! `HalfBoundedFactory` is split: implementors only provide the
//! fallible `Try*` half, and pick up the panicking surface for free
//! via blanket impl. See [`TryFiniteFactory`] / [`FiniteFactory`].
//!
//! # Strict by default; coercive is opt-in
//!
//! Every public factory entry point is **strict** by default. Crossed
//! bounds (`lhs > rhs` after normalization, or open-open at the same
//! point) produce `Err(InvalidBoundPair)` on the fallible path and
//! panic on the panicking path. Empty-set construction is reachable
//! through [`EmptyFactory::empty`], not through silent coercion.
//!
//! For code that legitimately needs "compute the set satisfying these
//! bounds, possibly empty" — intersection-shape ops, range
//! conversions, split, rebound — the parallel
//! [`SatisfyFiniteInterval`] / [`TrySatisfyFiniteInterval`] family
//! exposes a single coercive operation
//! ([`satisfy_bounds`](SatisfyFiniteInterval::satisfy_bounds) /
//! [`try_satisfy_bounds`](TrySatisfyFiniteInterval::try_satisfy_bounds))
//! at the `FiniteBound`-taking layer. There are no value-taking
//! coercive conveniences; if the caller wants the coercive path,
//! they construct `FiniteBound`s explicitly.

use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Re-exports for `use factory::traits::*` at user call sites.
pub mod traits {
    pub use super::{
        EmptyFactory, Factory, FiniteFactory, HalfBoundedFactory, SatisfyFiniteInterval,
        TryFiniteFactory, TryHalfBoundedFactory, TrySatisfyFiniteInterval, UnboundedFactory,
    };
}

/// Shared base for every factory trait in this module. Declares
/// `Output` and `Error` once so the `Empty` / `Unbounded` /
/// `Try{Finite,HalfBounded}Factory` (and their panicking siblings)
/// for a given `Self` all agree on what they produce and what error
/// they surface.
///
/// The trait is unconstrained on `T` — `Empty` and `Unbounded` don't
/// touch `T` values, so no `T: Element` bound belongs here. The
/// `Try*` subtraits add the `T: Element` bound where it's actually
/// load-bearing.
pub trait Factory<T> {
    /// The type that this factory produces.
    type Output;

    /// The error type for fallible (`try_*`) factory fns. Required to
    /// be `From<Error>` so factory methods can propagate
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)
    /// without per-method conversion plumbing.
    type Error: From<Error>;
}

/// Panic message for the panicking convenience methods that funnel
/// through `FiniteBound::try_new`. The backtrace already names the
/// originating method; the message just states the cause.
const REJECT_PANIC: &str = "bound limit rejected by Element::validate";

/// Constructs the empty set.
pub trait EmptyFactory<T>: Factory<T> {
    /// Returns a new empty set.
    ///
    /// {} = {x | x not in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::<i32>::empty();
    /// assert_eq!(x.contains(&10), false);
    /// ```
    fn empty() -> Self::Output;
}

/// Fallible finite-interval constructors. **Strict** — crossed
/// bounds (or open-open at the same point) produce
/// `Err(InvalidBoundPair)`. The implementor provides
/// [`try_fully_bounded`](Self::try_fully_bounded); the value-taking
/// defaults (`try_closed` / `try_open` / etc.) compose on it via
/// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new), so
/// validation runs at the bound layer for every entry point.
///
/// For coercive (crossed → `Empty`) semantics, see
/// [`TrySatisfyFiniteInterval`].
///
/// The panicking sibling [`FiniteFactory`] has a blanket impl over
/// every `TryFiniteFactory`.
pub trait TryFiniteFactory<T: Element>: Factory<T> {
    /// Creates a new finite interval. **Strict** — crossed bounds
    /// produce `Err(InvalidBoundPair)`. On success the result is a
    /// non-empty `Bounded` pair. Both `FiniteBound` inputs are already
    /// validated — they can only have been built via
    /// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new)
    /// (or its `try_closed` / `try_open` aliases, or the panicking
    /// convenience ctors that delegate to them).
    ///
    /// For coercive semantics (crossed bounds collapse to `Empty`),
    /// use
    /// [`TrySatisfyFiniteInterval::try_satisfy_bounds`].
    fn try_fully_bounded(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error>;

    /// Fallible closed-closed finite interval, validating each limit
    /// via [`FiniteBound::try_closed`]. Strict — crossed bounds error.
    fn try_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(lhs)?;
        let rhs = FiniteBound::try_closed(rhs)?;
        Self::try_fully_bounded(lhs, rhs)
    }

    /// Fallible open-open finite interval. Strict — crossed bounds error.
    fn try_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(lhs)?;
        let rhs = FiniteBound::try_open(rhs)?;
        Self::try_fully_bounded(lhs, rhs)
    }

    /// Fallible open-closed finite interval. Strict — crossed bounds error.
    fn try_open_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(lhs)?;
        let rhs = FiniteBound::try_closed(rhs)?;
        Self::try_fully_bounded(lhs, rhs)
    }

    /// Fallible closed-open finite interval. Strict — crossed bounds error.
    fn try_closed_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(lhs)?;
        let rhs = FiniteBound::try_open(rhs)?;
        Self::try_fully_bounded(lhs, rhs)
    }

    /// Fallible singleton (closed-closed at a single point).
    fn try_singleton(value: T) -> Result<Self::Output, Self::Error>
    where
        T: Clone,
    {
        Self::try_closed(value.clone(), value)
    }
}

/// Panicking sibling of [`TryFiniteFactory`]. Blanket-implemented for
/// every type that implements `TryFiniteFactory` — implementors do
/// not (and cannot) provide their own impl. Each method is the
/// `try_*(...).unwrap_or_else(|_| panic!(REJECT_PANIC))` of its
/// fallible counterpart.
///
/// **Strict** at every entry point. Crossed bounds panic; for
/// coercive semantics, see [`SatisfyFiniteInterval`].
pub trait FiniteFactory<T: Element>: TryFiniteFactory<T> {
    /// Creates a new finite interval from a pair of bounds. **Strict**
    /// — panics if the bounds are crossed
    /// ([`Error::InvalidBoundPair`](crate::error::Error::InvalidBoundPair))
    /// or if either limit is rejected by
    /// [`Element::validate`](crate::numeric::Element::validate)
    /// ([`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)).
    ///
    /// For coercive semantics (crossed → `Empty`), see
    /// [`SatisfyFiniteInterval::satisfy_bounds`].
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::open(0, 100);
    /// let y = EnumInterval::fully_bounded(
    ///     x.right().unwrap().clone().flip(),
    ///     FiniteBound::closed(200)
    /// );
    /// assert_eq!(y, EnumInterval::closed(100, 200));
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets_core::prelude::*;
    /// // Crossed bounds panic under strict semantics.
    /// let _ = EnumInterval::open(10, 0);
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets_core::prelude::*;
    /// // NaN is rejected by `Element::validate` and panics.
    /// let _ = EnumInterval::closed(f32::NAN, 0.0);
    /// ```
    fn fully_bounded(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output;

    /// Returns a new closed finite interval. **Strict** — panics on
    /// crossed bounds.
    ///
    /// [a, b] = { x in T | a <= x <= b }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::closed(10, 20);
    /// assert_eq!(x.contains(&10), true);
    /// assert_eq!(x.contains(&15), true);
    /// assert_eq!(x.contains(&20), true);
    /// assert_eq!(x.contains(&0), false);
    /// ```
    fn closed(left: T, right: T) -> Self::Output;

    /// Returns a new open finite interval. **Strict** — panics on
    /// crossed bounds.
    ///
    /// For discrete data types T, open bounds are **normalized** to
    /// closed form. Continuous(ish) types (like f32, or
    /// `chrono::DateTime`) are left as is.
    ///
    /// (a, b) = { x in T | a < x < b }
    ///
    /// # Example
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::open(0.0, 10.0);
    /// assert_eq!(x.contains(&0.0), false);
    /// assert_eq!(x.contains(&5.0), true);
    ///
    /// let y = EnumInterval::open(0, 10);
    /// assert_eq!(y.contains(&0), false);
    /// assert_eq!(y.contains(&5), true);
    /// assert_eq!(y, EnumInterval::closed(1, 9));
    /// ```
    fn open(left: T, right: T) -> Self::Output;

    /// Creates a left-open / right-closed finite interval. **Strict**
    /// — panics on crossed bounds.
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output;

    /// Creates a left-closed / right-open finite interval. **Strict**
    /// — panics on crossed bounds.
    ///
    ///  [a, b) = { x in T | a <= x < b }
    fn closed_open(left: T, right: T) -> Self::Output;

    /// Creates a singleton finite interval.
    ///
    /// [a, a] = { x | x == a }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::singleton(10);
    /// assert_eq!(x.contains(&10), true);
    /// assert_eq!(x.intersects(&FiniteInterval::closed(0, 20)), true);
    /// ```
    fn singleton(value: T) -> Self::Output
    where
        T: Clone;
}

impl<T: Element, F: TryFiniteFactory<T>> FiniteFactory<T> for F {
    #[inline]
    fn fully_bounded(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        Self::try_fully_bounded(lhs, rhs).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn closed(left: T, right: T) -> Self::Output {
        Self::try_closed(left, right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn open(left: T, right: T) -> Self::Output {
        Self::try_open(left, right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn open_closed(left: T, right: T) -> Self::Output {
        Self::try_open_closed(left, right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn closed_open(left: T, right: T) -> Self::Output {
        Self::try_closed_open(left, right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn singleton(value: T) -> Self::Output
    where
        T: Clone,
    {
        Self::try_singleton(value).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
}

/// Coercive bound-pair → finite-interval construction. Provides one
/// operation: build the set satisfying a pair of bounds, returning
/// `Empty` if no values satisfy them.
///
/// Implementors provide [`try_satisfy_bounds`](Self::try_satisfy_bounds);
/// the panicking sibling [`SatisfyFiniteInterval`] is
/// blanket-implemented.
///
/// Use this trait when you have a candidate `(lhs, rhs)` pair from a
/// computation that may legitimately collapse — `From<Range>`
/// conversions, intersection-shape ops, split, rebound — and "no
/// solutions" is a meaningful outcome of the operation.
///
/// For strict semantics (crossed bounds error), use
/// [`TryFiniteFactory::try_fully_bounded`].
///
/// Note: there is no value-taking surface (no `try_satisfy_closed`,
/// no `try_satisfy_open`). Coercive semantics is reachable only with
/// explicit `FiniteBound` construction. The ergonomic value-taking
/// surface (`closed`, `open`, etc. on [`FiniteFactory`]) is strict.
pub trait TrySatisfyFiniteInterval<T: Element>: Factory<T> {
    /// Builds the finite interval whose elements satisfy both
    /// `lhs` and `rhs`. **Coercive** — crossed bounds (or open-open
    /// at the same point) collapse to `Ok(Empty)`. Bound inputs are
    /// already validated (no construction path bypasses
    /// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new)),
    /// so this method itself is infallible on well-formed
    /// `FiniteBound` inputs.
    fn try_satisfy_bounds(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error>;
}

/// Panicking sibling of [`TrySatisfyFiniteInterval`].
/// Blanket-implemented; implementors do not provide their own impl.
pub trait SatisfyFiniteInterval<T: Element>: TrySatisfyFiniteInterval<T> {
    /// Builds the finite interval whose elements satisfy both bounds,
    /// or [`Empty`](EmptyFactory::empty) if no element satisfies them.
    /// Panics only on `Element::validate` failure.
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// // Crossed bounds collapse to Empty under coercive semantics.
    /// let x = FiniteInterval::satisfy_bounds(
    ///     FiniteBound::open(10),
    ///     FiniteBound::open(0),
    /// );
    /// assert_eq!(x, FiniteInterval::empty());
    /// ```
    fn satisfy_bounds(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output;
}

impl<T: Element, F: TrySatisfyFiniteInterval<T>> SatisfyFiniteInterval<T> for F {
    #[inline]
    fn satisfy_bounds(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        Self::try_satisfy_bounds(lhs, rhs).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
}

/// Fallible half-bounded constructors. Implementors provide
/// [`try_half_bounded`](Self::try_half_bounded); the side-specific
/// and open/closed convenience methods compose on it via
/// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new).
///
/// The panicking sibling [`HalfBoundedFactory`] has a blanket impl
/// over every `TryHalfBoundedFactory`.
pub trait TryHalfBoundedFactory<T: Element>: Factory<T> {
    /// Creates a new half-bounded interval, returning `Err` if the
    /// bound limit is rejected by `Element::validate`.
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error>;

    /// Fallible left-bounded helper.
    fn try_left_bounded(bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Left, bound)
    }

    /// Fallible right-bounded helper.
    fn try_right_bounded(bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Right, bound)
    }

    /// Fallible (a, ->) — open left bound, unbounded right.
    fn try_open_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::try_open(lhs)?)
    }

    /// Fallible [a, ->) — closed left bound, unbounded right.
    fn try_closed_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::try_closed(lhs)?)
    }

    /// Fallible (<-, b) — unbounded left, open right bound.
    fn try_unbound_open(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::try_open(rhs)?)
    }

    /// Fallible (<-, b] — unbounded left, closed right bound.
    fn try_unbound_closed(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::try_closed(rhs)?)
    }
}

/// Panicking sibling of [`TryHalfBoundedFactory`]. Blanket-implemented
/// for every type that implements `TryHalfBoundedFactory`.
pub trait HalfBoundedFactory<T: Element>: TryHalfBoundedFactory<T> {
    /// Returns a new half-bounded interval.
    ///
    /// # Example
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::unbound_open(0);
    /// let y = EnumInterval::half_bounded(Side::Left, x.right().unwrap().clone().flip());
    /// let z = x.merge_connected(y).unwrap();
    /// assert_eq!(z, EnumInterval::Unbounded);
    /// ```
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output;

    /// Right-bounded interval (`(<-, b]` or `(<-, b)`).
    fn right_bounded(bound: FiniteBound<T>) -> Self::Output;

    /// Left-bounded interval (`[a, ->)` or `(a, ->)`).
    fn left_bounded(bound: FiniteBound<T>) -> Self::Output;

    /// Returns a new open, right-unbound interval `(a, ->)`.
    fn open_unbound(left: T) -> Self::Output;

    /// Returns a new closed, right-unbound interval `[a, ->)`.
    fn closed_unbound(left: T) -> Self::Output;

    /// Returns a new open, left-unbound interval `(<-, b)`.
    fn unbound_open(right: T) -> Self::Output;

    /// Returns a new closed, left-unbound interval `(<-, b]`.
    fn unbound_closed(right: T) -> Self::Output;
}

impl<T: Element, F: TryHalfBoundedFactory<T>> HalfBoundedFactory<T> for F {
    #[inline]
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        Self::try_half_bounded(side, bound).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn right_bounded(bound: FiniteBound<T>) -> Self::Output {
        Self::try_right_bounded(bound).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn left_bounded(bound: FiniteBound<T>) -> Self::Output {
        Self::try_left_bounded(bound).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn open_unbound(left: T) -> Self::Output {
        Self::try_open_unbound(left).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn closed_unbound(left: T) -> Self::Output {
        Self::try_closed_unbound(left).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn unbound_open(right: T) -> Self::Output {
        Self::try_unbound_open(right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn unbound_closed(right: T) -> Self::Output {
        Self::try_unbound_closed(right).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
}

/// Constructs the unbounded interval `(<-, ->)`.
pub trait UnboundedFactory<T>: Factory<T> {
    /// Returns a new unbounded interval.
    ///
    /// An unbounded interval contains every element in `T`,
    /// and therefore is a superset of all sets of `T`.
    ///
    /// (<-, ->) = { x in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::<f32>::unbounded();
    /// assert_eq!(x.contains(&10.0), true);
    /// assert_eq!(x.contains(&EnumInterval::empty()), true);
    /// ```
    fn unbounded() -> Self::Output;
}

impl<T: Element> Factory<T> for FiniteInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T> for FiniteInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T> for FiniteInterval<T> {
    fn try_fully_bounded(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        Self::try_new(lhs, rhs)
    }
}

impl<T: Element> TrySatisfyFiniteInterval<T> for FiniteInterval<T> {
    fn try_satisfy_bounds(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        match Self::try_new(lhs, rhs) {
            Err(Error::InvalidBoundPair) => Ok(Self::empty()),
            other => other,
        }
    }
}

impl<T: Element> Factory<T> for HalfInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> TryHalfBoundedFactory<T> for HalfInterval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_new(side, bound)
    }
}

impl<T: Element> Factory<T> for EnumInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T> for EnumInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T> for EnumInterval<T> {
    fn try_fully_bounded(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_fully_bounded(lhs, rhs).map(Self::Output::from)
    }
}

impl<T: Element> TrySatisfyFiniteInterval<T> for EnumInterval<T> {
    fn try_satisfy_bounds(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_satisfy_bounds(lhs, rhs).map(Self::Output::from)
    }
}

impl<T: Element> TryHalfBoundedFactory<T> for EnumInterval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound).map(Self::Output::from)
    }
}

impl<T: Element> UnboundedFactory<T> for EnumInterval<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_factory_strict() {
        // Factory is strict: crossed bounds error.
        assert!(EnumInterval::try_open(10.0, 0.0).is_err());
        assert!(EnumInterval::try_closed(10, 0).is_err());

        // NaN surfaces as InvalidBoundLimit at the bound chokepoint.
        assert_eq!(EnumInterval::try_singleton(f32::NAN).ok(), None);
        assert_eq!(EnumInterval::try_unbound_open(f32::NAN).ok(), None);

        // Well-formed input still works.
        assert_eq!(
            EnumInterval::try_closed_unbound(0.0).ok(),
            Some(EnumInterval::closed_unbound(0.0))
        );
    }

    #[test]
    fn test_try_satisfy_bounds_coercive() {
        // Coercive entry point: crossed bounds collapse to Empty.
        assert_eq!(
            EnumInterval::try_satisfy_bounds(FiniteBound::open(10.0), FiniteBound::open(0.0))
                .unwrap(),
            EnumInterval::empty()
        );
        assert_eq!(
            FiniteInterval::satisfy_bounds(FiniteBound::open(10), FiniteBound::open(0)),
            FiniteInterval::empty()
        );

        // Non-crossed input builds the corresponding interval.
        assert_eq!(
            FiniteInterval::satisfy_bounds(FiniteBound::closed(0), FiniteBound::closed(10)),
            FiniteInterval::closed(0, 10)
        );
    }
}

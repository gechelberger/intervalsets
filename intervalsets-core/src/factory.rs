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

use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Re-exports for `use factory::traits::*` at user call sites.
pub mod traits {
    pub use super::{
        ConvertingFactory, EmptyFactory, FiniteFactory, HalfBoundedFactory, TryFiniteFactory,
        TryHalfBoundedFactory, UnboundedFactory,
    };
}

/// Common base for the factory traits — declares the produced type
/// and the error type for `try_*` constructors.
pub trait ConvertingFactory<T> {
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
pub trait EmptyFactory<T>: ConvertingFactory<T> {
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

/// Fallible finite-interval constructors. The single method an
/// implementor must provide is [`try_finite`](Self::try_finite); the
/// `try_closed` / `try_open` / etc. defaults compose on it via
/// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new), so
/// validation runs at the bound layer for every entry point.
///
/// The panicking sibling [`FiniteFactory`] has a blanket impl over
/// every `TryFiniteFactory`, so implementors get the panicking
/// surface for free.
pub trait TryFiniteFactory<T: Element>: ConvertingFactory<T> {
    /// Creates a new finite interval. **Coercive** — bounds that
    /// describe an empty set produce `Ok(Empty)`. Reaching this
    /// method via the factory `try_*` defaults guarantees both
    /// bounds have already been validated by `Element::validate`;
    /// direct callers passing `FiniteBound::closed(NaN)` via the
    /// Tier-4 bypass can still surface
    /// [`Error::TotalOrderError`](crate::error::Error::TotalOrderError)
    /// from the underlying `try_cmp`.
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error>;

    /// Fallible closed-closed finite interval, validating each limit
    /// via [`FiniteBound::try_closed`].
    fn try_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(lhs)?;
        let rhs = FiniteBound::try_closed(rhs)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible open-open finite interval.
    fn try_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(lhs)?;
        let rhs = FiniteBound::try_open(rhs)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible open-closed finite interval.
    fn try_open_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(lhs)?;
        let rhs = FiniteBound::try_closed(rhs)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible closed-open finite interval.
    fn try_closed_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(lhs)?;
        let rhs = FiniteBound::try_open(rhs)?;
        Self::try_finite(lhs, rhs)
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
pub trait FiniteFactory<T: Element>: TryFiniteFactory<T> {
    /// Creates a new finite interval. Panics if either bound's limit
    /// is rejected by [`Element::validate`](crate::numeric::Element::validate);
    /// see [`TryFiniteFactory::try_finite`] for the fallible variant.
    ///
    /// **Coercive.** Bounds that describe an empty set (crossed
    /// values after normalization, or open-at-the-same-point) silently
    /// produce `Empty`.
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::open(0, 100);
    /// let y = EnumInterval::finite(
    ///     x.right().unwrap().clone().flip(),
    ///     FiniteBound::closed(200)
    /// );
    /// assert_eq!(y, EnumInterval::closed(100, 200));
    ///
    /// // Same-point open bounds describe an empty set.
    /// let x = EnumInterval::open(10, 10);
    /// assert_eq!(x, EnumInterval::empty());
    ///
    /// // Crossed bounds also describe an empty set.
    /// let x = EnumInterval::open(10, 0);
    /// assert_eq!(x, EnumInterval::empty());
    /// ```
    ///
    /// ```should_panic
    /// use intervalsets_core::prelude::*;
    /// // NaN is rejected by `Element::validate` and panics.
    /// let _ = EnumInterval::closed(f32::NAN, 0.0);
    /// ```
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output;

    /// Returns a new closed finite interval or `Empty`.
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

    /// Returns a new open finite interval or `Empty`.
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

    /// Creates a left-open / right-closed finite interval or `Empty`.
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output;

    /// Creates a left-closed / right-open finite interval or `Empty`.
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
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        Self::try_finite(lhs, rhs).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
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

/// Fallible half-bounded constructors. Implementors provide
/// [`try_half_bounded`](Self::try_half_bounded); the side-specific
/// and open/closed convenience methods compose on it via
/// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new).
///
/// The panicking sibling [`HalfBoundedFactory`] has a blanket impl
/// over every `TryHalfBoundedFactory`.
pub trait TryHalfBoundedFactory<T: Element>: ConvertingFactory<T> {
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
pub trait UnboundedFactory<T>: ConvertingFactory<T> {
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

impl<T: Element> ConvertingFactory<T> for FiniteInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T> for FiniteInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T> for FiniteInterval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_new_or_empty(lhs, rhs)
    }
}

impl<T: Element> ConvertingFactory<T> for HalfInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> TryHalfBoundedFactory<T> for HalfInterval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_new(side, bound)
    }
}

impl<T: Element> ConvertingFactory<T> for EnumInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T> for EnumInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T> for EnumInterval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_finite(lhs, rhs).map(Self::Output::from)
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
    fn test_try_factory() {
        assert_eq!(EnumInterval::try_singleton(f32::NAN).ok(), None);
        // Factory is coercive: crossed bounds → Ok(empty).
        // Use FiniteInterval::try_new directly for strict validation.
        assert_eq!(
            EnumInterval::try_open(10.0, 0.0).unwrap(),
            EnumInterval::empty()
        );
        assert_eq!(EnumInterval::try_unbound_open(f32::NAN).ok(), None);
        assert_eq!(
            EnumInterval::try_closed_unbound(0.0).ok(),
            Some(EnumInterval::closed_unbound(0.0))
        );
    }
}

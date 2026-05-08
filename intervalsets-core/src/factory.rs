//! Factories for intervals.
//!
//! Factory types are not intended to be constructed. Rather, they provide
//! associated functions to build intervals of an associated type and may
//! coerce the input types for convenience using a generic [`Converter`].
//!
//! The Factory traits are intended to provide a common
//! interface for creating the full spectrum of possible
//! intervals. [`EnumInterval`] itself is a factory using
//! the [`Identity`] converter. Use [`EIFactory`] to supply
//! a custom converter.
//!
//! Sometimes it is preferable for the underlying storage
//! to be a wrapper or NewType. [`Converter`] provides a mechanism
//! to do so with less boiler plate.
//!
//! # Examples
//! ```
//! use intervalsets_core::prelude::*;
//! let x = EnumInterval::<u32>::closed(0, 10);
//! let y = EnumInterval::<u32>::closed(5, 15);
//! assert_eq!(x.intersection(y), EnumInterval::closed(5, 10));
//! ```
//!
//! ```no-compile
//! use intervalsets_core::prelude::*;
//! use intervalsets_core::factory::EIFactory;
//! use ordered_float::NotNan;
//!
//! // explicit
//! let x = EnumInterval::open(
//!     NotNan::<f32>::new(0.0).unwrap(),
//!     NotNan::<f32>::new(10.0).unwrap()
//! );
//!
//! // factory with converter
//! type Fct = EIFactory<f32, NotNan<f32>>;
//! let y = Fct::open(0.0, 10.0);
//!
//! assert_eq!(x, y);
//! ```

use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Can be used instead of the prelude to pull in all factory traits.
pub mod traits {
    pub use super::{
        EmptyFactory, FiniteFactory, HalfBoundedFactory, TryFiniteFactory, TryHalfBoundedFactory,
        UnboundedFactory,
    };
}

/// Convert an arbitrary type to one implementing [`Element`].
///
/// The [`Converter`] trait provides a mechanism to wrap
/// or coerce a type into one that is compatible with interval bounds. This is
/// particularly useful when working with 3rd party crates with unsupported types.
///
/// # Structure
///
/// ```text
/// type UserDefinedFactory = IFactory<X, C<From = X>>;
/// X: the type users will invoke factory methods with
/// C: the type which implements Converter
///     - C can implement converter for multiple From types
///     - X must be one of those implementations
/// C<X>::To: the underlying storage type for intervals created by the factory.
/// ```
///
/// # Note
///
/// The type that this trait is implemented on is somewhat arbitrary and can
/// therefore be confusing. It is structured as it is to provide flexibility in
/// satisfying the [orphan rule](https://github.com/Ixrec/rust-orphan-rules).
/// This allows users to easily create a converter for foreign types.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::factory::{EIFactory, Converter};
///
/// // Local type converter
///
/// #[derive(Copy, Clone)]
/// struct Timestamp{
///     seconds: u32,
///     nanos: u32
/// };
/// let a = Timestamp{ seconds: 0, nanos: 0};
/// let b = Timestamp{ seconds: 10, nanos: 0};
///
/// impl Converter<Timestamp> for u64 {
///     type To = u64; // impl Element
///     fn convert(value: Timestamp) -> Option<Self::To> {
///         Some((value.seconds as u64) << 32 | value.nanos as u64)
///     }
/// }
///
/// type Fct = EIFactory<Timestamp, u64>;
/// let x = Fct::closed(a, b);
///
/// // Foreign types
///
/// struct CharCvt;
///
/// // char and u32 are not local so we can't implement Converter on either one.
/// impl Converter<char> for CharCvt {
///     type To = u32;
///     fn convert(value: char) -> Option<Self::To> {
///         Some(value as u32)
///     }
/// }
///
/// type Fct2 = EIFactory<char, CharCvt>;
/// let x = Fct2::closed('a', 'z');
/// assert_eq!(x.contains(&CharCvt::convert('c').unwrap()), true);
/// assert_eq!(x.contains(&CharCvt::convert('C').unwrap()), false);
/// assert_eq!(x.contains(&CharCvt::convert('0').unwrap()), false);
/// ```
pub trait Converter<From> {
    /// The underlying storage type.
    type To: Element;

    /// Convert `value` into the storage type, or return `None` if the
    /// input cannot be wrapped (e.g. `NotNan::new(NaN)` would panic).
    /// `None` collapses into
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)
    /// at the factory boundary, matching the
    /// [`Element::validate`](crate::numeric::Element::validate)
    /// rejection path. Conversions that cannot fail return
    /// `Some(...)` unconditionally.
    fn convert(value: From) -> Option<Self::To>;
}

/// [`Identity`] is the default [`Converter`] implementation and is a NOOP.
pub struct Identity;

impl<T: Element> Converter<T> for Identity {
    type To = T;

    #[inline]
    fn convert(value: T) -> Option<Self::To> {
        Some(value)
    }
}

/// Enforces a single production type for a factory.
pub trait ConvertingFactory<T, C = Identity>
where
    C: Converter<T>,
    C::To: Element,
{
    /// The type that this factory produces
    type Output;

    /// The error type for fallible (try_*) factory fns. Required to be
    /// `From<Error>` so the factory's bound-validation chokepoint can
    /// propagate
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)
    /// without per-method conversion plumbing.
    type Error: From<Error>;

    /// Apply the factory's [`Converter`] and surface failure as
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit).
    /// Convenience for `try_*` constructors that take raw `T` inputs;
    /// keeps the conversion-failure path aligned with the
    /// [`Element::validate`](crate::numeric::Element::validate)
    /// rejection path.
    #[inline]
    fn try_convert(value: T) -> Result<C::To, Error> {
        C::convert(value).ok_or(Error::InvalidBoundLimit)
    }
}

/// Panic message for the panicking convenience methods that funnel
/// through `FiniteBound::try_new`. The backtrace already names the
/// originating method; the message just states the cause.
const REJECT_PANIC: &str = "bound limit rejected by Element::validate";

/// Constructs the empty set.
pub trait EmptyFactory<T, C>: ConvertingFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Returns a new Empty Set
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
pub trait TryFiniteFactory<T, C>: ConvertingFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Creates a new finite interval. **Coercive** — bounds that
    /// describe an empty set produce `Ok(Empty)`; only NaN /
    /// incomparable values surface as `Err`.
    ///
    /// For strict validation that errors on crossed bounds, use
    /// [`FiniteInterval::try_new`](crate::sets::FiniteInterval::try_new)
    /// directly.
    fn try_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error>;

    /// Fallible closed-closed finite interval, validating each limit
    /// via [`FiniteBound::try_closed`].
    fn try_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(Self::try_convert(lhs)?)?;
        let rhs = FiniteBound::try_closed(Self::try_convert(rhs)?)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible open-open finite interval.
    fn try_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(Self::try_convert(lhs)?)?;
        let rhs = FiniteBound::try_open(Self::try_convert(rhs)?)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible open-closed finite interval.
    fn try_open_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_open(Self::try_convert(lhs)?)?;
        let rhs = FiniteBound::try_closed(Self::try_convert(rhs)?)?;
        Self::try_finite(lhs, rhs)
    }

    /// Fallible closed-open finite interval.
    fn try_closed_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        let lhs = FiniteBound::try_closed(Self::try_convert(lhs)?)?;
        let rhs = FiniteBound::try_open(Self::try_convert(rhs)?)?;
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
/// every type that implements `TryFiniteFactory` — implementors do not
/// (and cannot) provide their own impl. Each method is the
/// `try_*(...).unwrap_or_else(|_| panic!(...))` of its fallible
/// counterpart.
pub trait FiniteFactory<T, C>: TryFiniteFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Creates a new finite interval of the factory's associated type.
    ///
    /// **Coercive.** Bounds that describe an empty set (crossed values
    /// after normalization, or open-at-the-same-point) silently produce
    /// `Empty`. NaN / `±INF` / `Element::validate`-rejected values panic.
    ///
    /// For strict validation that distinguishes "crossed bounds" from
    /// "bounds describe empty," call
    /// [`FiniteInterval::try_new`](crate::sets::FiniteInterval::try_new)
    /// directly.
    ///
    /// # Panics
    ///
    /// Panics if either bound's limit is rejected by
    /// [`Element::validate`](crate::numeric::Element::validate). Use
    /// [`try_finite`](TryFiniteFactory::try_finite) to surface that as
    /// an `Err`.
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
    /// // NaN still panics.
    /// let _ = EnumInterval::closed(f32::NAN, 0.0);
    /// ```
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output;

    /// Returns a new closed finite interval or Empty.
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

    /// Returns a new open finite interval or Empty.
    ///
    /// For discrete data types T, open bounds are **normalized** to closed form.
    /// Continuous(ish) types (like f32, or chrono::DateTime) are left as is.
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

    /// Creates a left open finite interval or Empty.
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output;

    /// Creates a right open finite interval or Empty.
    ///
    ///  [a, b) = { x in T | a <= x < b }
    fn closed_open(left: T, right: T) -> Self::Output;

    /// Creates a new singleton finite interval.
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

impl<T, C, F> FiniteFactory<T, C> for F
where
    F: TryFiniteFactory<T, C>,
    C: Converter<T>,
    C::To: Element,
{
    #[inline]
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
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
/// [`try_half_bounded`](Self::try_half_bounded); the side-specific and
/// open/closed convenience methods compose on it via
/// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new).
///
/// The panicking sibling [`HalfBoundedFactory`] has a blanket impl
/// over every `TryHalfBoundedFactory`.
pub trait TryHalfBoundedFactory<T, C>: ConvertingFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Creates a new half-bounded interval, returning `Err` if the
    /// bound limit is rejected by `Element::validate`.
    fn try_half_bounded(side: Side, bound: FiniteBound<C::To>)
        -> Result<Self::Output, Self::Error>;

    /// Fallible left-bounded helper.
    fn try_left_bounded(bound: FiniteBound<C::To>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Left, bound)
    }

    /// Fallible right-bounded helper.
    fn try_right_bounded(bound: FiniteBound<C::To>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Right, bound)
    }

    /// Fallible (a, ->) — open left bound, unbounded right.
    fn try_open_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::try_open(Self::try_convert(lhs)?)?)
    }

    /// Fallible [a, ->) — closed left bound, unbounded right.
    fn try_closed_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::try_closed(Self::try_convert(lhs)?)?)
    }

    /// Fallible (<-, b) — unbounded left, open right bound.
    fn try_unbound_open(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::try_open(Self::try_convert(rhs)?)?)
    }

    /// Fallible (<-, b] — unbounded left, closed right bound.
    fn try_unbound_closed(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::try_closed(Self::try_convert(rhs)?)?)
    }
}

/// Panicking sibling of [`TryHalfBoundedFactory`]. Blanket-implemented
/// for every type that implements `TryHalfBoundedFactory`.
pub trait HalfBoundedFactory<T, C>: TryHalfBoundedFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Returns a new half bounded interval.
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
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output;

    /// Right-bounded interval (`(<-, b]` or `(<-, b)`).
    fn right_bounded(bound: FiniteBound<C::To>) -> Self::Output;

    /// Left-bounded interval (`[a, ->)` or `(a, ->)`).
    fn left_bounded(bound: FiniteBound<C::To>) -> Self::Output;

    /// Returns a new open, right-unbound interval `(a, ->)`.
    fn open_unbound(left: T) -> Self::Output;

    /// Returns a new closed, right-unbound interval `[a, ->)`.
    fn closed_unbound(left: T) -> Self::Output;

    /// Returns a new open, left-unbound interval `(<-, b)`.
    fn unbound_open(right: T) -> Self::Output;

    /// Returns a new closed, left-unbound interval `(<-, b]`.
    fn unbound_closed(right: T) -> Self::Output;
}

impl<T, C, F> HalfBoundedFactory<T, C> for F
where
    F: TryHalfBoundedFactory<T, C>,
    C: Converter<T>,
    C::To: Element,
{
    #[inline]
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        Self::try_half_bounded(side, bound).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn right_bounded(bound: FiniteBound<C::To>) -> Self::Output {
        Self::try_right_bounded(bound).unwrap_or_else(|_| panic!("{REJECT_PANIC}"))
    }
    #[inline]
    fn left_bounded(bound: FiniteBound<C::To>) -> Self::Output {
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

pub trait UnboundedFactory<T, C = Identity>: ConvertingFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Returns a new unbounded interval.
    ///
    /// An unbounded interval contains every element in `T``,
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

impl<T: Element> ConvertingFactory<T, Identity> for FiniteInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T, Identity> for FiniteInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T, Identity> for FiniteInterval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_new_or_empty(lhs, rhs)
    }
}

impl<T: Element> ConvertingFactory<T, Identity> for HalfInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> TryHalfBoundedFactory<T, Identity> for HalfInterval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        Self::try_new(side, bound)
    }
}

impl<T: Element> ConvertingFactory<T, Identity> for EnumInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> EmptyFactory<T, Identity> for EnumInterval<T> {
    fn empty() -> Self::Output {
        Self::empty()
    }
}

impl<T: Element> TryFiniteFactory<T, Identity> for EnumInterval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_finite(lhs, rhs).map(Self::Output::from)
    }
}

impl<T: Element> TryHalfBoundedFactory<T, Identity> for EnumInterval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound).map(Self::Output::from)
    }
}

impl<T: Element> UnboundedFactory<T, Identity> for EnumInterval<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

/// A factory type for EnumIntervals.
///
/// Use this factory instead of EnumInterval if a custom [`Converter`] is needed.
pub struct EIFactory<T, C = Identity>(core::marker::PhantomData<(T, C)>);

impl<T, C> ConvertingFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    type Output = EnumInterval<C::To>;
    type Error = Error;
}

impl<T, C> EmptyFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn empty() -> Self::Output {
        EnumInterval::empty()
    }
}

impl<T, C> TryFiniteFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn try_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs).map(EnumInterval::from)
    }
}

impl<T, C> TryHalfBoundedFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn try_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound).map(EnumInterval::from)
    }
}

impl<T, C> UnboundedFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_factory() {
        let a = EIFactory::<u32, Identity>::closed(0, 10);
        let b = EnumInterval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }

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

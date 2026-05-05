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
    pub use super::{EmptyFactory, FiniteFactory, HalfBoundedFactory, UnboundedFactory};
}

/// Convert an arbitrary type to one implemnting [`Element`].
///
/// The [`Converter`] trait provides a mechanism to wrap
/// or coerse a type into one that is compatible with interval bounds. This is
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
///     fn convert(value: Timestamp) -> Self::To {
///         (value.seconds as u64) << 32 | value.nanos as u64
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
///     fn convert(value: char) -> Self::To {
///         value as u32
///     }
/// }
///
/// type Fct2 = EIFactory<char, CharCvt>;
/// let x = Fct2::closed('a', 'z');
/// assert_eq!(x.contains(&CharCvt::convert('c')), true);
/// assert_eq!(x.contains(&CharCvt::convert('C')), false);
/// assert_eq!(x.contains(&CharCvt::convert('0')), false);
/// ```
pub trait Converter<From> {
    /// The underlying storage type.
    type To: Element;

    /// Creates a new value of the associated type.
    fn convert(value: From) -> Self::To;
}

/// [`Identity`] is the default [`Converter`] implementation and is a NOOP.
pub struct Identity;

impl<T: Element> Converter<T> for Identity {
    type To = T;

    fn convert(value: T) -> Self::To {
        value
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

    /// The error type for fallible (try_*) factory fns
    type Error;
}

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

pub trait FiniteFactory<T, C>: ConvertingFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    /// Creates a new finite interval of the factory's associated type.
    ///
    /// **Coercive.** Bounds that describe an empty set (crossed values
    /// after normalization, or open-at-the-same-point) silently produce
    /// `Empty`. NaN / incomparable values panic.
    ///
    /// For strict validation that distinguishes "crossed bounds" from
    /// "bounds describe empty," call
    /// [`FiniteInterval::try_new`](crate::sets::FiniteInterval::try_new)
    /// directly.
    ///
    /// # Panics
    ///
    /// Panics if either bound is incomparable (e.g. NaN). Use
    /// [`try_finite`](Self::try_finite) to surface that as an `Err`.
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

    /// Returns a new closed finite interval or Empty
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
    fn closed(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::closed(C::convert(left)),
            FiniteBound::closed(C::convert(right)),
        )
    }

    fn try_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_finite(
            FiniteBound::closed(C::convert(lhs)),
            FiniteBound::closed(C::convert(rhs)),
        )
    }

    /// Creates a new singleton finite interval
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
        T: Clone,
    {
        Self::closed(value.clone(), value)
    }

    fn try_singleton(value: T) -> Result<Self::Output, Self::Error>
    where
        T: Clone,
    {
        Self::try_closed(value.clone(), value)
    }

    /// Returns a new open finite interval or Empty
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
    fn open(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::open(C::convert(left)),
            FiniteBound::open(C::convert(right)),
        )
    }

    fn try_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_finite(
            FiniteBound::open(C::convert(lhs)),
            FiniteBound::open(C::convert(rhs)),
        )
    }

    /// Creates a left open finite interval or Empty
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::open(C::convert(left)),
            FiniteBound::closed(C::convert(right)),
        )
    }

    fn try_open_closed(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_finite(
            FiniteBound::open(C::convert(lhs)),
            FiniteBound::closed(C::convert(rhs)),
        )
    }

    /// Creates a right open finite interval or Empty
    ///
    ///  [a, b) = { x in T | a <= x < b }
    fn closed_open(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::closed(C::convert(left)),
            FiniteBound::open(C::convert(right)),
        )
    }

    fn try_closed_open(lhs: T, rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_finite(
            FiniteBound::closed(C::convert(lhs)),
            FiniteBound::open(C::convert(rhs)),
        )
    }
}

pub trait HalfBoundedFactory<T, C>: ConvertingFactory<T, C>
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

    /// Creates a new half-bounded interval, returning `Err` if the
    /// bound is not comparable (e.g. NaN). The panic-free counterpart
    /// of [`half_bounded`](Self::half_bounded).
    fn try_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error>;

    fn right_bounded(bound: FiniteBound<C::To>) -> Self::Output {
        Self::half_bounded(Side::Right, bound)
    }

    fn left_bounded(bound: FiniteBound<C::To>) -> Self::Output {
        Self::half_bounded(Side::Left, bound)
    }

    fn try_left_bounded(bound: FiniteBound<C::To>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Left, bound)
    }

    fn try_right_bounded(bound: FiniteBound<C::To>) -> Result<Self::Output, Self::Error> {
        Self::try_half_bounded(Side::Right, bound)
    }

    /// Returns a new open, right-unbound interval
    ///
    ///  (a, ->) = { x in T | a < x }
    fn open_unbound(left: T) -> Self::Output {
        Self::left_bounded(FiniteBound::open(C::convert(left)))
    }

    fn try_open_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::open(C::convert(lhs)))
    }

    /// Returns a new closed, right-unbound interval
    ///
    ///  [a, ->) = {x in T | a <= x }
    fn closed_unbound(left: T) -> Self::Output {
        Self::half_bounded(Side::Left, FiniteBound::closed(C::convert(left)))
    }

    fn try_closed_unbound(lhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_left_bounded(FiniteBound::closed(C::convert(lhs)))
    }

    /// Returns a new open, left-unbound interval
    ///
    /// (a, ->) = { x in T | a < x }
    fn unbound_open(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, FiniteBound::open(C::convert(right)))
    }

    fn try_unbound_open(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::open(C::convert(rhs)))
    }

    /// Returns a new closed, left-unbound interval
    ///
    ///  [a, ->) = { x in T | a <= x }
    fn unbound_closed(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, FiniteBound::closed(C::convert(right)))
    }

    fn try_unbound_closed(rhs: T) -> Result<Self::Output, Self::Error> {
        Self::try_right_bounded(FiniteBound::closed(C::convert(rhs)))
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

impl<T: Element> FiniteFactory<T, Identity> for FiniteInterval<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        // Coercive: crossed bounds → empty, NaN → panic.
        Self::try_new_or_empty(lhs, rhs).unwrap()
    }

    fn try_finite(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        Self::try_new_or_empty(lhs, rhs)
    }
}

impl<T: Element> ConvertingFactory<T, Identity> for HalfInterval<T> {
    type Output = Self;
    type Error = Error;
}

impl<T: Element> HalfBoundedFactory<T, Identity> for HalfInterval<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        Self::new(side, bound)
    }

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

impl<T: Element> FiniteFactory<T, Identity> for EnumInterval<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        FiniteInterval::finite(lhs, rhs).into()
    }

    fn try_finite(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_finite(lhs, rhs).map(Self::Output::from)
    }
}

impl<T: Element> HalfBoundedFactory<T, Identity> for EnumInterval<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

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

impl<T, C> FiniteFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
        // Coercive: crossed bounds → empty, NaN → panic.
        FiniteInterval::try_new_or_empty(lhs, rhs).unwrap().into()
    }

    fn try_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs).map(EnumInterval::from)
    }
}

impl<T, C> HalfBoundedFactory<T, C> for EIFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

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

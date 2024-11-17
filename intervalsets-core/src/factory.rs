//use ordered_float::{NotNan, OrderedFloat};

use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

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
/// use intervalsets_core::factory::{IFactory, Converter};
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
///     type To = u64; // impl Domain & Zero
///     fn convert(value: Timestamp) -> Self::To {
///         (value.seconds as u64) << 32 | value.nanos as u64
///     }
/// }
///
/// type Fct = IFactory<Timestamp, u64>;
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
/// type Fct2 = IFactory<char, CharCvt>;
/// let x = Fct2::closed('a', 'z');
/// assert!(x.contains(&CharCvt::convert('c')));
/// assert!(!x.contains(&CharCvt::convert('C')));
/// assert!(!x.contains(&CharCvt::convert('0')));
/// ```
pub trait Converter<From> {
    type To;
    fn convert(value: From) -> Self::To;
}

/// [`Identity`] is the default [`Converter`] implementation and is a NOOP.
pub struct Identity;

impl<T> Converter<T> for Identity {
    type To = T;

    fn convert(value: T) -> Self::To {
        value
    }
}

/// The [`Factory`] trait is intended to provide a common
/// interface for creating the full spectrum of possible
/// intervals. [`EnumInterval`] itself is a factory using
/// the [`Identity`] converter. Use [`IFactory`] to supply
/// a custom converter.
///
/// Sometimes it is preferable for the underlying storage
/// to be a wrapper or NewType. [`Converter`] provides a mechanism
/// to do so with less boiler plate.
///
/// # Examples
/// ```
/// use intervalsets_core::prelude::*;
/// type Fct = EnumInterval<u32>;
/// let x = Fct::closed(0, 10);
/// let y = Fct::closed(5, 15);
/// assert_eq!(x.intersection(y), Fct::closed(5, 10))
/// ```
///
/// This example uses the optional [`ordered-float`] feature.
///
/// ```no_compile
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::factory::IFactory;
/// use ordered_float::NotNan;
///
/// // explicit
/// let x = EnumInterval::open(
///     NotNan::<f32>::new(0.0).unwrap(),
///     NotNan::<f32>::new(10.0).unwrap()
/// );
///
/// // factory with converter
/// type Fct = IFactory<f32, NotNan<f32>>;
/// let y = Fct::open(0.0, 10.0);
///
/// assert_eq!(x, y);
/// ```
pub trait Factory<T, C = Identity>
where
    C: Converter<T>,
    C::To: Domain,
{
    type Output;

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

    /// Returns a new finite interval.
    ///
    /// If there are no elements that satisfy both left and right bounds
    /// then an `Empty` interval is returned. Otherwise the result will
    /// be fully bounded.
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
    /// let x = EnumInterval::open(10, 10);
    /// assert_eq!(x, EnumInterval::empty());
    /// ```
    fn finite(left: FiniteBound<C::To>, right: FiniteBound<C::To>) -> Self::Output;

    /// Returns a new half bounded interval.
    ///
    /// # Example
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::unbound_open(0);
    /// let y = EnumInterval::half_bounded(Side::Left, x.right().unwrap().clone().flip());
    /// let z = x.try_merge(y).unwrap();
    /// assert_eq!(z, EnumInterval::Unbounded);
    /// ```
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output;

    /// todo: ...
    fn right_bounded(bound: FiniteBound<C::To>) -> Self::Output {
        Self::half_bounded(Side::Right, bound)
    }

    /// todo: ...
    fn left_bounded(bound: FiniteBound<C::To>) -> Self::Output {
        Self::half_bounded(Side::Left, bound)
    }

    /// Returns a new unbounded interval.
    ///
    /// An unbounded interval contains every element in T,
    /// as well as every set of T except the `Empty` set.
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
    /// assert_eq!(x.contains(&EnumInterval::empty()), false);
    /// ```
    fn unbounded() -> Self::Output;

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

    /// Returns a new left open finite interval or Empty
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::open(C::convert(left)),
            FiniteBound::closed(C::convert(right)),
        )
    }

    /// Returns a new right open finite interval or Empty
    ///
    ///  [a, b) = { x in T | a <= x < b }
    fn closed_open(left: T, right: T) -> Self::Output {
        Self::finite(
            FiniteBound::closed(C::convert(left)),
            FiniteBound::open(C::convert(right)),
        )
    }

    /// Returns a new open, right-unbound interval
    ///
    ///  (a, ->) = { x in T | a < x }
    fn open_unbound(left: T) -> Self::Output {
        Self::half_bounded(Side::Left, FiniteBound::open(C::convert(left)))
    }

    /// Returns a new closed, right-unbound interval
    ///
    ///  [a, ->) = {x in T | a <= x }
    fn closed_unbound(left: T) -> Self::Output {
        Self::half_bounded(Side::Left, FiniteBound::closed(C::convert(left)))
    }

    /// Returns a new open, left-unbound interval
    ///
    /// (a, ->) = { x in T | a < x }
    fn unbound_open(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, FiniteBound::open(C::convert(right)))
    }

    /// Returns a new closed, left-unbound interval
    ///
    ///  [a, ->) = { x in T | a <= x }
    fn unbound_closed(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, FiniteBound::closed(C::convert(right)))
    }
}

impl<T> Factory<T, Identity> for FiniteInterval<T>
where
    T: Domain,
{
    type Output = Self;

    fn empty() -> Self::Output {
        Self::empty()
    }

    fn finite(left: FiniteBound<T>, right: FiniteBound<T>) -> Self::Output {
        Self::new(left, right)
    }

    fn half_bounded(_: Side, _: FiniteBound<T>) -> Self::Output {
        panic!("todo")
    }

    fn unbounded() -> Self::Output {
        panic!("todo")
    }
}

pub struct IFactory<T, C = Identity>(core::marker::PhantomData<(T, C)>);

impl<T, C> Factory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    type Output = EnumInterval<C::To>;

    fn empty() -> Self::Output {
        FiniteInterval::Empty.into()
    }

    fn finite(left: FiniteBound<C::To>, right: FiniteBound<C::To>) -> Self::Output {
        FiniteInterval::new(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

impl<T: Domain> Factory<T, Identity> for EnumInterval<T> {
    type Output = Self;

    fn empty() -> Self::Output {
        FiniteInterval::Empty.into()
    }

    fn finite(left: FiniteBound<T>, right: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_factory() {
        let a = IFactory::<u32, Identity>::closed(0, 10);
        let b = EnumInterval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }
}

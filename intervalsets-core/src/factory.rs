use ordered_float::{NotNan, OrderedFloat};

use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};

/// The [`Converter`] trait provides a mechanism to wrap
/// or coerse a convenient type into one that meets
/// the requirements for sets.
///
/// Examples
///
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::factory::{IFactory, Converter};
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

impl<T: num_traits::float::FloatCore> Converter<T> for NotNan<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        NotNan::new(value).unwrap()
    }
}

impl<T: num_traits::float::FloatCore> Converter<T> for OrderedFloat<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        OrderedFloat(value)
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
/// ```
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

    /// Returns a ew half bounded interval.
    ///
    /// # Example
    /// ```
    /// use intervalsets_core::prelude::*;
    ///
    /// let x = EnumInterval::unbound_open(0);
    /// let y = EnumInterval::half_bounded(Side::Left, x.right().unwrap().clone().flip());
    /// assert_eq!(x.complement(), y.into());
    /// ```
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output;

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
        Self::new(left, right).expect("todo")
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
        FiniteInterval::new(left, right).unwrap().into()
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
        FiniteInterval::new(left, right).unwrap().into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded
    }
}

pub struct SFactory<T, C>(core::marker::PhantomData<(T, C)>);

impl<T, C> Factory<T, C> for SFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain + Ord,
{
    type Output = StackSet<C::To>;

    fn empty() -> Self::Output {
        Self::Output::empty()
    }

    fn finite(left: FiniteBound<C::To>, right: FiniteBound<C::To>) -> Self::Output {
        EnumInterval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        EnumInterval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::unbounded().into()
    }
}

impl<T: Domain + Ord> Factory<T, Identity> for StackSet<T> {
    type Output = Self;

    fn empty() -> Self::Output {
        EnumInterval::empty().into()
    }

    fn finite(left: FiniteBound<T>, right: FiniteBound<T>) -> Self::Output {
        EnumInterval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        EnumInterval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::unbounded().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_interval_factory() {
        let a = IFactory::<u32, Identity>::closed(0, 10);
        let b = EnumInterval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_interval_set_factory() -> Result<(), Error> {
        let x = StackSet::closed(0, 10);
        assert_eq!(x.expect_interval()?, EnumInterval::closed(0, 10));

        Ok(())
    }
}

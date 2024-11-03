use crate::detail::{BoundCase, Finite, HalfBounded};
use crate::numeric::Domain;
use crate::{Bound, Interval, IntervalSet, Side};

/// The [`Cvt`] trait provides a mechanism to wrap
/// or coerse a convenient type into one that meets
/// the requirements for sets.
///
/// Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// #[derive(Copy, Clone)]
/// struct Timestamp{
///     seconds: u32,
///     nanos: u32
/// };
/// let a = Timestamp{ seconds: 0, nanos: 0};
/// let b = Timestamp{ seconds: 10, nanos: 0};
///
/// impl Cvt<Timestamp> for u64 {
///     type To = u64;
///     fn convert_to(value: Timestamp) -> Self::To {
///         (value.seconds as u64) << 32 | value.nanos as u64
///     }
/// }
///
/// type Fct = IFactory<Timestamp, u64>;
/// let x = Fct::closed(a, b);
/// ```
pub trait Cvt<From> {
    type To;
    fn convert_to(value: From) -> Self::To;
}

/// [`Identity`] is the default [`Cvt`] implementation and is a NOOP.
pub struct Identity;

impl<T> Cvt<T> for Identity {
    type To = T;

    fn convert_to(value: T) -> Self::To {
        value
    }
}

/// The [`Factory`] trait is intended to provide a common
/// interface for creating the full spectrum of possible
/// intervals. [`Interval`] itself is a factory using
/// the [`Identity`] converter. Use [`IFactory`] to supply
/// a custom converter.
///
/// Sometimes it is preferable for the underlying storage
/// to be a wrapper or NewType. [`Cvt`] provides a mechanism
/// to do so with less boiler plate.
///
/// # Examples
/// ```
/// use intervalsets::prelude::*;
/// type Fct = Interval<u32>;
/// let x = Fct::closed(0, 10);
/// let y = Fct::closed(5, 15);
/// assert_eq!(x.intersection(y), Fct::closed(5, 10))
/// ```
///
/// This example uses the optional [`ordered-float`] feature.
///
/// ```ignore
/// use intervalsets::prelude::*;
/// use ordered_float::NotNan;
///
/// // explicit
/// let x = Interval::open(
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
    C: Cvt<T>,
    C::To: Domain,
{
    type Output;

    /// Returns a new Empty [`Interval`]
    ///
    /// {} = {x | x not in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::Contains;
    ///
    /// let x = Interval::<i32>::empty();
    /// assert_eq!(x.contains(&10), false);
    /// ```
    fn empty() -> Self::Output;

    /// Returns a new finite [`Interval`].
    ///
    /// If there are no elements that satisfy both left and right bounds
    /// then an `Empty` interval is returned. Otherwise the result will
    /// be fully bounded.
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Bound, Interval, Bounding, Factory};
    ///
    /// let x = Interval::open(0, 100);
    /// let y = Interval::finite(
    ///     x.right().unwrap().clone().flip(),
    ///     Bound::closed(200)
    /// );
    /// assert_eq!(y, Interval::closed(100, 200));
    ///
    /// let x = Interval::open(10, 10);
    /// assert_eq!(x, Interval::empty());
    /// ```
    fn finite(left: Bound<C::To>, right: Bound<C::To>) -> Self::Output;

    /// Returns a ew half bounded [`Interval`].
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Bound, Bounding, Side, Factory};
    /// use intervalsets::ops::Complement;
    ///
    /// let x = Interval::unbound_open(0);
    /// let y = Interval::half_bounded(Side::Left, x.right().unwrap().clone().flip());
    /// assert_eq!(x.complement(), y.into());
    /// ```
    fn half_bounded(side: Side, bound: Bound<C::To>) -> Self::Output;

    /// Returns a new unbounded [`Interval`].
    ///
    /// An unbounded interval contains every element in T,
    /// as well as every set of T except the `Empty` set.
    ///
    /// (<-, ->) = { x in T }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::Contains;
    ///
    /// let x = Interval::<f32>::unbounded();
    /// assert_eq!(x.contains(&10.0), true);
    /// assert_eq!(x.contains(&Interval::empty()), false);
    /// ```
    fn unbounded() -> Self::Output;

    /// Returns a new closed finite [`Interval`] or Empty
    ///
    /// [a, b] = { x in T | a <= x <= b }
    ///
    /// # Example
    ///
    /// ```
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::Contains;
    ///
    /// let x = Interval::closed(10, 20);
    /// assert_eq!(x.contains(&10), true);
    /// assert_eq!(x.contains(&15), true);
    /// assert_eq!(x.contains(&20), true);
    /// assert_eq!(x.contains(&0), false);
    /// ```
    fn closed(left: T, right: T) -> Self::Output {
        Self::finite(
            Bound::closed(C::convert_to(left)),
            Bound::closed(C::convert_to(right)),
        )
    }

    /// Returns a new open finite [`Interval`] or Empty
    ///
    /// For discrete data types T, open bounds are **normalized** to closed form.
    /// Continuous(ish) types (like f32, or chrono::DateTime) are left as is.
    ///
    /// (a, b) = { x in T | a < x < b }
    ///
    /// # Example
    /// ```
    /// use intervalsets::{Interval, Factory};
    /// use intervalsets::ops::Contains;
    ///
    /// let x = Interval::open(0.0, 10.0);
    /// assert_eq!(x.contains(&0.0), false);
    /// assert_eq!(x.contains(&5.0), true);
    ///
    /// let y = Interval::open(0, 10);
    /// assert_eq!(y.contains(&0), false);
    /// assert_eq!(y.contains(&5), true);
    /// assert_eq!(y, Interval::closed(1, 9));
    /// ```
    fn open(left: T, right: T) -> Self::Output {
        Self::finite(
            Bound::open(C::convert_to(left)),
            Bound::open(C::convert_to(right)),
        )
    }

    /// Returns a new left open finite [`Interval`] or Empty
    ///
    ///  (a, b] = { x in T | a < x <= b }
    fn open_closed(left: T, right: T) -> Self::Output {
        Self::finite(
            Bound::open(C::convert_to(left)),
            Bound::closed(C::convert_to(right)),
        )
    }

    /// Returns a new right open finite [`Interval`] or Empty
    ///
    ///  [a, b) = { x in T | a <= x < b }
    fn closed_open(left: T, right: T) -> Self::Output {
        Self::finite(
            Bound::closed(C::convert_to(left)),
            Bound::open(C::convert_to(right)),
        )
    }

    /// Returns a new open, right-unbound [`Interval`]
    ///
    ///  (a, ->) = { x in T | a < x }
    fn open_unbound(left: T) -> Self::Output {
        Self::half_bounded(Side::Left, Bound::open(C::convert_to(left)))
    }

    /// Returns a new closed, right-unbound [`Interval`]
    ///
    ///  [a, ->) = {x in T | a <= x }
    fn closed_unbound(left: T) -> Self::Output {
        Self::half_bounded(Side::Left, Bound::closed(C::convert_to(left)))
    }

    /// Returns a new open, left-unbound [`Interval`]
    ///
    /// (a, ->) = { x in T | a < x }
    fn unbound_open(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, Bound::open(C::convert_to(right)))
    }

    /// Returns a new closed, left-unbound [`Interval`]
    ///
    ///  [a, ->) = { x in T | a <= x }
    fn unbound_closed(right: T) -> Self::Output {
        Self::half_bounded(Side::Right, Bound::closed(C::convert_to(right)))
    }
}

pub struct IFactory<T, C = Identity>(std::marker::PhantomData<(T, C)>);

impl<T, C> Factory<T, C> for IFactory<T, C>
where
    C: Cvt<T>,
    C::To: Domain,
{
    type Output = Interval<C::To>;

    fn empty() -> Self::Output {
        Finite::Empty.into()
    }

    fn finite(left: Bound<C::To>, right: Bound<C::To>) -> Self::Output {
        Finite::new(left, right).into()
    }

    fn half_bounded(side: Side, bound: Bound<C::To>) -> Self::Output {
        HalfBounded::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        BoundCase::Unbounded.into()
    }
}

impl<T: Domain> Factory<T, Identity> for Interval<T> {
    type Output = Interval<T>;

    fn empty() -> Self::Output {
        Finite::Empty.into()
    }

    fn finite(left: Bound<T>, right: Bound<T>) -> Self::Output {
        Finite::new(left, right).into()
    }

    fn half_bounded(side: Side, bound: Bound<T>) -> Self::Output {
        HalfBounded::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        BoundCase::Unbounded.into()
    }
}

pub struct SFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> Factory<T, C> for SFactory<T, C>
where
    C: Cvt<T>,
    C::To: Domain,
{
    type Output = IntervalSet<C::To>;

    fn empty() -> Self::Output {
        Self::Output::empty()
    }

    fn finite(left: Bound<C::To>, right: Bound<C::To>) -> Self::Output {
        Interval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: Bound<C::To>) -> Self::Output {
        Interval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        Interval::unbounded().into()
    }
}

impl<T: Domain> Factory<T, Identity> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn empty() -> Self::Output {
        Interval::empty().into()
    }

    fn finite(left: Bound<T>, right: Bound<T>) -> Self::Output {
        Interval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: Bound<T>) -> Self::Output {
        Interval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        Interval::unbounded().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_factory() {
        let a = IFactory::<u32, Identity>::closed(0, 10);
        let b = Interval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_interval_set_factory() {
        let x = IntervalSet::closed(0, 10);
        assert_eq!(x.expect_interval(), Interval::closed(0, 10));
    }
}

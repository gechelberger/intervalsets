use super::Measurement;
use crate::error::Error;
use crate::numeric::{Element, Zero};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Defines the counting measure of a [`Countable`] Set.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::measure::Count;
///
/// let x = EnumInterval::closed(0, 10);
/// assert_eq!(x.count().finite(), 11);
/// ```
///
/// # Restricted to types implementing Countable
/// ```compile_fail
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::measure::Count;
///
/// // f32 does not implement [`Countable`]
/// let x = EnumInterval::closed(0.0, 10.0).count();
/// ```
pub trait Count {
    type Output;

    /// Compute the counting measure of this set.
    ///
    /// # Panics
    ///
    /// Panics if the count cannot be represented in `Self::Output`
    /// (e.g. counting `[i32::MIN, i32::MAX]` overflows `i32`). For
    /// panic-free counting, use [`try_count`](Count::try_count).
    fn count(&self) -> Measurement<Self::Output> {
        self.try_count().unwrap()
    }

    /// Compute the counting measure of this set, returning `Err` if
    /// the count cannot be represented in `Self::Output`.
    fn try_count(&self) -> Result<Measurement<Self::Output>, Error>;
}

/// Defines whether a set of type T is countable.
///
/// [`Count`] delegates to the underlying type that implements [`Countable`].
///
/// # Example
/// ```
/// use intervalsets_core::numeric::Element;
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::default_countable_impl;
/// use intervalsets_core::measure::{Count, Countable};
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// struct MyInt(i32);
///
/// impl core::ops::Add for MyInt {
///     type Output = Self;
///     fn add(self, rhs: Self) -> Self {
///         MyInt(self.0 + rhs.0)
///     }
/// }
///
/// impl core::ops::Sub for MyInt {
///     type Output = Self;
///     fn sub(self, rhs: Self) -> Self {
///         MyInt(self.0 - rhs.0)
///     }
/// }
///
/// impl num_traits::Zero for MyInt {
///     fn zero() -> Self {
///         MyInt(0)
///     }
///
///     fn is_zero(&self) -> bool {
///         self.0 == 0
///     }
/// }
///
/// impl Element for MyInt {
///     fn try_adjacent(&self, side: Side) -> Option<Self> {
///         Some(match side {
///             Side::Left => MyInt(self.0 - 1),
///             Side::Right => MyInt(self.0 + 1),
///         })
///     }
/// }
///
/// default_countable_impl!(MyInt);
///
/// /*
/// impl Countable for MyInt {
///     type Output = Self;
///     fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
///         Some(MyInt(right.0 - left.0 + 1))
///     }
/// }*/
///
/// let interval = FiniteInterval::closed(MyInt(0), MyInt(10));
/// assert_eq!(interval.count().finite(), MyInt(11));
/// ```
pub trait Countable: Element {
    type Output;

    fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output>;
}

#[macro_export]
macro_rules! default_countable_impl {
    ($t_in_out:ty) => {
        impl $crate::measure::Countable for $t_in_out {
            type Output = $t_in_out;

            fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
                //Some(right - left + 1)
                if let Some(upper) = right.try_adjacent($crate::bound::Side::Right) {
                    return Some(upper - left.clone());
                }

                if let Some(lower) = left.try_adjacent($crate::bound::Side::Left) {
                    return Some(right.clone() - lower);
                }

                // Both adjacents overflow (e.g. [MIN, MAX]). The count
                // mathematically exists but cannot fit in Self::Output.
                None
            }
        }
    }
}

default_countable_impl!(u8);
default_countable_impl!(u16);
default_countable_impl!(u32);
default_countable_impl!(u64);
default_countable_impl!(u128);
default_countable_impl!(usize);

default_countable_impl!(i8);
default_countable_impl!(i16);
default_countable_impl!(i32);
default_countable_impl!(i64);
default_countable_impl!(i128);
default_countable_impl!(isize);

impl<T> Count for FiniteInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Error> {
        match self.view_raw() {
            Some((left, right)) => match T::count_inclusive(left.value(), right.value()) {
                Some(count) => Ok(Measurement::Finite(count)),
                None => Err(Error::CountOverflow),
            },
            None => Ok(Measurement::Finite(Self::Output::zero())),
        }
    }
}

impl<T> Count for HalfInterval<T> {
    type Output = ();

    fn try_count(&self) -> Result<Measurement<Self::Output>, Error> {
        Ok(Measurement::Infinite)
    }
}

impl<T> Count for EnumInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Error> {
        match self {
            Self::Finite(inner) => inner.try_count(),
            _ => Ok(Measurement::Infinite),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn test_count() {
        let x = EnumInterval::closed(0, 10);
        assert_eq!(x.count().finite(), 11);
    }

    #[test]
    fn test_try_count_overflow() {
        // [i32::MIN, i32::MAX] has 2^32 elements which doesn't fit in i32.
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        assert!(x.try_count().is_err());
    }

    #[test]
    #[should_panic]
    fn test_count_overflow_panics() {
        // count() is the panicking sibling of try_count() and is
        // documented to panic when the count overflows Self::Output.
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        let _ = x.count();
    }
}

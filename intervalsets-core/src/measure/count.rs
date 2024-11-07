use core::ops::Add;

use super::Measurement;
use crate::numeric::{Domain, Zero};
use crate::sets::{FiniteInterval, HalfInterval, EnumInterval, StackSet};

/// Defines the counting measure of a [`Countable`] Set.
///
/// # Example
/// ```
/// use intervalsets::{Interval, IntervalSet, Factory};
/// use intervalsets::ops::Union;
/// use intervalsets::measure::Count;
///
/// let x = Interval::closed(1, 10);
/// assert_eq!(x.count().finite(), 10);
///
/// let x: IntervalSet<_> = x.union(Interval::closed(101, 200));
/// assert_eq!(x.count().finite(), 110);
/// ```
///
/// # Restricted to types implementing Countable
/// ```compile_fail
/// use intervalsets::Interval;
/// use intervalsets::measure::Count;
///
/// // f32 does not implement [`Countable`]
/// let x = Interval::closed(0.0, 10.0).count();
/// ```
pub trait Count {
    type Output;

    fn count(&self) -> Measurement<Self::Output>;
}

/// Defines whether a set of type T is countable.
///
/// [`Count`] delegates to the underlying type that implements [`Countable`].
///
/// # Example
/// ```
/// use intervalsets::numeric::Domain;
/// use intervalsets::{Interval, Factory, Side, default_countable_impl};
/// use intervalsets::measure::{Count, Countable};
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
/// impl intervalsets::numeric::Zero for MyInt {
///     fn zero() -> Self {
///         MyInt(0)
///     }
///
///     fn is_zero(&self) -> bool {
///         self.0 == 0
///     }
/// }
///
/// impl Domain for MyInt {
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
/// let interval = Interval::closed(MyInt(0), MyInt(10));
/// assert_eq!(interval.count().finite(), MyInt(11));
/// ```
pub trait Countable: Domain {
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

                panic!("Countable type overflow; or Domain adjacent not implemented for Countable type.");
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

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Bounded(left, right) => {
                let count = T::count_inclusive(left.value(), right.value())
                    .expect("Count should be Some since interval is FullyBounded");
                Measurement::Finite(count)
            }
            Self::Empty => Measurement::Finite(Self::Output::zero()),
        }
    }
}

impl<T> Count for EnumInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;

    fn count(&self) -> crate::measure::Measurement<Self::Output> {
        match self {
            Self::Finite(inner) => inner.count(),
            _ => Measurement::Infinite,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        //let x: Interval<i64> = Interval::closed(0.0, 10.0);
        //assert_eq!(x.count().finite(), 11);
    }
}

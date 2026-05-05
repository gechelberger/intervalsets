use super::Measurement;
use crate::numeric::{Element, Zero};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// The counting measure of a set cannot be represented by the
/// [`Countable::Output`] type (e.g. counting `[i32::MIN, i32::MAX]`
/// overflows `i32`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("count overflows the Countable Output type")]
pub struct CountOverflowError;

/// Defines the counting measure of a [`Countable`] Set.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::measure::Count;
///
/// let x = EnumInterval::closed(0, 10);
/// assert_eq!(x.count().finite(), 11u128);
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
    type Error: core::error::Error;

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
    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error>;
}

/// Defines whether a set of type T is countable.
///
/// [`Count`] delegates to the underlying type that implements [`Countable`].
///
/// # Example
/// ```
/// use intervalsets_core::numeric::{CheckedSub, Element};
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
/// impl CheckedSub for MyInt {
///     fn checked_sub(&self, rhs: &Self) -> Option<Self> {
///         self.0.checked_sub(rhs.0).map(MyInt)
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
///         match side {
///             Side::Left => self.0.checked_sub(1).map(MyInt),
///             Side::Right => self.0.checked_add(1).map(MyInt),
///         }
///     }
/// }
///
/// default_countable_impl!(MyInt);
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
                // count = (right - left) + 1, computed via try_adjacent so we
                // can fall back when one endpoint sits at the type's limit.
                // Both subtractions go through CheckedSub: if the count itself
                // doesn't fit in Self::Output, we return None instead of
                // panicking on overflow.
                if let Some(upper) = right.try_adjacent($crate::bound::Side::Right) {
                    return $crate::numeric::CheckedSub::checked_sub(&upper, left);
                }
                if let Some(lower) = left.try_adjacent($crate::bound::Side::Left) {
                    return $crate::numeric::CheckedSub::checked_sub(right, &lower);
                }
                None
            }
        }
    }
}

/// Implements [`Countable`] for native primitive integer types narrower than
/// 128 bits. Output is always [`u128`]; the input is widened to [`i128`]
/// before subtraction, so no intermediate overflow is possible. Always
/// returns `Some`.
macro_rules! primitive_countable_impl {
    ($($t:ty),+ $(,)?) => {
        $(
            impl $crate::measure::Countable for $t {
                type Output = u128;

                fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
                    let diff = (*right as i128) - (*left as i128);
                    Some(diff as u128 + 1)
                }
            }
        )+
    };
}

primitive_countable_impl!(u8, u16, u32, u64, usize);
primitive_countable_impl!(i8, i16, i32, i64, isize);

// 128-bit types need bespoke handling: the count of `[MIN, MAX]` is 2^128,
// which is one past the u128 representable range.
impl Countable for u128 {
    type Output = u128;

    fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
        right.checked_sub(*left).and_then(|d| d.checked_add(1))
    }
}

impl Countable for i128 {
    type Output = u128;

    fn count_inclusive(left: &Self, right: &Self) -> Option<Self::Output> {
        // Interval invariant: right >= left, so the wrapping i128 difference
        // reinterpreted as u128 yields the true unsigned distance, up to 2^128 - 1.
        let diff = right.wrapping_sub(*left) as u128;
        diff.checked_add(1)
    }
}

impl<T> Count for FiniteInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = CountOverflowError;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        match self.view_raw() {
            Some((left, right)) => match T::count_inclusive(left.value(), right.value()) {
                Some(count) => Ok(Measurement::Finite(count)),
                None => Err(CountOverflowError),
            },
            None => Ok(Measurement::Finite(Self::Output::zero())),
        }
    }
}

impl<T> Count for HalfInterval<T> {
    type Output = ();
    type Error = CountOverflowError;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        Ok(Measurement::Infinite)
    }
}

impl<T> Count for EnumInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = CountOverflowError;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error> {
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
        assert_eq!(x.count().finite(), 11u128);
    }

    #[test]
    fn test_count_signed_full_range() {
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        assert_eq!(x.count().finite(), (u32::MAX as u128) + 1);
    }

    #[test]
    fn test_count_signed_negative_range() {
        // Previously panicked: 1 - i32::MIN overflowed i32 in count_inclusive.
        let x = EnumInterval::closed(i32::MIN, 0);
        assert_eq!(x.count().finite(), (i32::MAX as u128) + 2);
    }

    #[test]
    fn test_count_unsigned_full_range() {
        let x = EnumInterval::closed(0u64, u64::MAX);
        assert_eq!(x.count().finite(), (u64::MAX as u128) + 1);
    }

    #[test]
    fn test_count_i128_full_range_overflows_u128() {
        // [i128::MIN, i128::MAX] has 2^128 elements which doesn't fit in u128.
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        assert!(x.try_count().is_err());
    }

    #[test]
    fn test_count_u128_full_range_overflows_u128() {
        // [0, u128::MAX] has 2^128 elements which doesn't fit in u128.
        let x = EnumInterval::closed(0u128, u128::MAX);
        assert!(x.try_count().is_err());
    }

    #[test]
    #[should_panic]
    fn test_count_overflow_panics() {
        // count() is the panicking sibling of try_count() and is
        // documented to panic when the count overflows Self::Output.
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        let _ = x.count();
    }
}

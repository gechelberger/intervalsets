use super::Extent;
use crate::error::MathError;
use crate::numeric::{Element, Zero};
use crate::ops::math::TryAdd;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Defines the cardinality (counting measure) of a [`Countable`] Set.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::measure::Cardinality;
///
/// let x = EnumInterval::closed(0, 10);
/// assert_eq!(x.cardinality().finite(), 11u128);
/// ```
pub trait Cardinality {
    type Output;
    type Error: core::error::Error;

    /// Compute the cardinality of this set.
    ///
    /// # Panics
    ///
    /// Panics if the cardinality cannot be represented in `Self::Output`
    /// (e.g. counting `[i32::MIN, i32::MAX]` overflows `i32`). For
    /// panic-free counting, use [`try_cardinality`](Cardinality::try_cardinality).
    fn cardinality(&self) -> Extent<Self::Output> {
        self.try_cardinality().unwrap()
    }

    /// Compute the cardinality of this set, returning `Err` if
    /// it cannot be represented in `Self::Output`.
    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error>;
}

/// Defines whether a set of type T is countable.
///
/// [`Cardinality`] delegates to the underlying type that implements [`Countable`].
///
/// # Example
/// ```
/// use intervalsets_core::numeric::{CheckedSub, Element};
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::default_countable_impl;
/// use intervalsets_core::measure::{Cardinality, Countable};
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
/// assert_eq!(interval.cardinality().finite(), MyInt(11));
/// ```
pub trait Countable: Element {
    type Output;

    /// `true` if `Self` is a continuous element type (uncountably many values
    /// between any two distinct elements). Acts as a *semantic marker for what
    /// `None` from [`count_inclusive`](Self::count_inclusive) means*:
    ///
    /// - **Discrete** (`false`, default): `None` ⇒ representation overflow.
    ///   `Cardinality::try_cardinality` raises `Err(MathError::Range)`.
    /// - **Continuous** (`true`): `None` ⇒ uncountable (non-singleton range).
    ///   `Cardinality::try_cardinality` returns `Ok(Extent::Infinite)`.
    ///
    /// Continuous impls return `Some(1)` from `count_inclusive` when
    /// `left == right` (a singleton) and `None` otherwise.
    const IS_CONTINUOUS: bool = false;

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
    };
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

/// Implements [`Countable`] for a continuous element type. `Output` is
/// [`u128`]; `count_inclusive` returns `Some(1)` when `left == right`
/// (a singleton) and `None` otherwise. The `Cardinality` impl interprets
/// `None` as `Extent::Infinite` because `IS_CONTINUOUS = true`.
#[macro_export]
macro_rules! continuous_countable_impl {
    ($t:ty) => {
        impl $crate::measure::Countable for $t {
            type Output = u128;
            const IS_CONTINUOUS: bool = true;

            fn count_inclusive(left: &Self, right: &Self) -> Option<u128> {
                if left == right {
                    Some(1)
                } else {
                    None
                }
            }
        }
    };
}

continuous_countable_impl!(f32);
continuous_countable_impl!(f64);

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

impl<T> Cardinality for FiniteInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = MathError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self.view_raw() {
            None => Ok(Extent::Finite(Self::Output::zero())),
            Some((left, right)) => match T::count_inclusive(left.value(), right.value()) {
                Some(count) => Ok(Extent::Finite(count)),
                None if T::IS_CONTINUOUS => Ok(Extent::Infinite),
                None => Err(MathError::Range),
            },
        }
    }
}

impl<T> Cardinality for HalfInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = MathError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        Ok(Extent::Infinite)
    }
}

impl<T> Cardinality for EnumInterval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = MathError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Finite(inner) => inner.try_cardinality(),
            _ => Ok(Extent::Infinite),
        }
    }
}

/// Cardinality of a [`MaybeDisjoint`] is the sum of its pieces' cardinalities.
/// `Connected(iv)` delegates; `Disjoint(a, b)` sums per-piece cardinalities
/// via [`TryAdd`] so an overflowing total surfaces as [`MathError`]
/// rather than wrapping. Infinite from either piece propagates to an
/// infinite total.
impl<T, Out> Cardinality for MaybeDisjoint<T>
where
    T: Countable<Output = Out>,
    Out: Zero + TryAdd<Out, Output = Out>,
    <Out as TryAdd>::Error: Into<MathError>,
{
    type Output = Out;
    type Error = MathError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        match self {
            Self::Connected(iv) => iv.try_cardinality(),
            Self::Disjoint(a, b) => {
                let ac = a.try_cardinality()?;
                let bc = b.try_cardinality()?;
                ac.try_binop_map(bc, |x, y| x.try_add(y).map_err(Into::into))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};

    #[test]
    fn test_cardinality() {
        let x = EnumInterval::closed(0, 10);
        assert_eq!(x.cardinality().finite(), 11u128);
    }

    #[test]
    fn test_cardinality_signed_full_range() {
        let x = EnumInterval::closed(i32::MIN, i32::MAX);
        assert_eq!(x.cardinality().finite(), (u32::MAX as u128) + 1);
    }

    #[test]
    fn test_cardinality_signed_negative_range() {
        // Previously panicked: 1 - i32::MIN overflowed i32 in count_inclusive.
        let x = EnumInterval::closed(i32::MIN, 0);
        assert_eq!(x.cardinality().finite(), (i32::MAX as u128) + 2);
    }

    #[test]
    fn test_cardinality_unsigned_full_range() {
        let x = EnumInterval::closed(0u64, u64::MAX);
        assert_eq!(x.cardinality().finite(), (u64::MAX as u128) + 1);
    }

    #[test]
    fn test_cardinality_i128_full_range_overflows_u128() {
        // [i128::MIN, i128::MAX] has 2^128 elements which doesn't fit in u128.
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        assert!(x.try_cardinality().is_err());
    }

    #[test]
    fn test_cardinality_u128_full_range_overflows_u128() {
        // [0, u128::MAX] has 2^128 elements which doesn't fit in u128.
        let x = EnumInterval::closed(0u128, u128::MAX);
        assert!(x.try_cardinality().is_err());
    }

    #[test]
    #[should_panic]
    fn test_cardinality_overflow_panics() {
        // cardinality() is the panicking sibling of try_cardinality() and is
        // documented to panic when the cardinality overflows Self::Output.
        let x = EnumInterval::closed(i128::MIN, i128::MAX);
        let _ = x.cardinality();
    }

    // ===== MaybeDisjoint =====

    #[test]
    fn md_empty_cardinality_is_zero() {
        let x = MaybeDisjoint::<i32>::empty();
        assert_eq!(x.cardinality().finite(), 0_u128);
    }

    #[test]
    fn md_connected_delegates_to_inner_cardinality() {
        let x = MaybeDisjoint::from_interval(EnumInterval::closed(0, 10));
        assert_eq!(x.cardinality().finite(), 11_u128);
    }

    #[test]
    fn md_disjoint_cardinality_sums_pieces() {
        // [0, 5] (6 elements) ∪ [10, 20] (11 elements) → 17
        let x =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i32, 5), EnumInterval::closed(10, 20));
        assert_eq!(x.cardinality().finite(), 17_u128);
    }

    #[test]
    fn md_disjoint_with_half_interval_is_infinite() {
        // Disjoint(finite, half) — half-piece makes total cardinality infinite.
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i32, 5),
            EnumInterval::closed_unbound(10),
        );
        assert!(x.try_cardinality().unwrap().is_infinite());
    }

    #[test]
    fn md_per_piece_overflow_propagates() {
        // A single piece's count_inclusive can overflow (e.g.
        // [i128::MIN, i128::MAX] needs 2^128 which doesn't fit in u128).
        // That overflow surfaces from the inner try_cardinality and propagates
        // through MD's impl via `?`.
        let inner = EnumInterval::closed(i128::MIN, i128::MAX);
        let md = MaybeDisjoint::from_interval(inner);
        assert!(md.try_cardinality().is_err());
    }

    // ===== Continuous T =====

    #[test]
    fn singleton_f64_cardinality_is_one() {
        let x = EnumInterval::closed(5.0_f64, 5.0);
        assert_eq!(x.cardinality().finite(), 1_u128);
    }

    #[test]
    fn nondegenerate_f64_cardinality_is_infinite() {
        let x = EnumInterval::closed(5.0_f64, 5.0001);
        assert!(x.try_cardinality().unwrap().is_infinite());
    }

    #[test]
    fn empty_f64_cardinality_is_zero() {
        let x = FiniteInterval::<f64>::empty();
        assert_eq!(x.cardinality().finite(), 0_u128);
    }

    #[test]
    fn md_disjoint_f64_singletons_sum_to_two() {
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 0.0),
            EnumInterval::closed(1.0, 1.0),
        );
        assert_eq!(x.cardinality().finite(), 2_u128);
    }

    #[test]
    fn md_disjoint_f64_with_nondegenerate_is_infinite() {
        // One non-degenerate piece makes the total uncountable.
        let x = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 0.0),
            EnumInterval::closed(1.0, 2.0),
        );
        assert!(x.try_cardinality().unwrap().is_infinite());
    }
}

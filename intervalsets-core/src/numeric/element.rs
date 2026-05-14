use crate::bound::Side;
use crate::numeric::Zero;
use crate::ops::math::TryAdd;

mod sealed {
    pub trait Kind {}
}

/// Marker indicating a [discrete](Element::Kind) element type. Every
/// element has a unique adjacent neighbor under the type's order
/// ([`Element::try_adjacent`] returns `Some`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiscreteKind;
impl sealed::Kind for DiscreteKind {}

/// Marker indicating a [continuous](Element::Kind) element type. The
/// order admits no successor / predecessor relation
/// ([`Element::try_adjacent`] returns `None`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ContinuousKind;
impl sealed::Kind for ContinuousKind {}

/// Defines the data types whose elements make up a Set.
///
/// `Element` declares (1) whether the type is discrete or continuous
/// via [`Kind`](Element::Kind), (2) what its natural additive measure
/// is via [`Measure`](Element::Measure), and (3) the primitives
/// needed to compute that measure on a finite-bounded interval.
///
/// # Kind: discrete vs continuous
///
/// Discrete types ([`DiscreteKind`]) have adjacency — every element
/// has a unique next/prev under the type's order, and
/// [`try_adjacent`](Element::try_adjacent) returns `Some(±1-equivalent)`.
/// Continuous types ([`ContinuousKind`]) do not — `try_adjacent`
/// returns `None`. The crate uses adjacency to normalize discrete
/// intervals to a canonical bit-pattern (`[1, 9]` ≡ `(0, 10)` etc.)
/// and to merge connected pieces.
///
/// # Measure: the natural additive measure of a finite-bounded interval
///
/// [`try_measure_finite`](Element::try_measure_finite) computes the
/// natural measure of `[left, right]`:
///
/// - Discrete `T`: cardinality (e.g. `right - left + 1` for primitive
///   integers).
/// - Continuous `T`: Lebesgue width (`right - left`).
///
/// `None` means representation overflow; finite-bounded sets surface
/// this as `Err(MathError::Range)` through the
/// [`Measure`](crate::measure::Measure) trait.
///
/// ## `Measure` type convention
///
/// The associated [`Measure`](Element::Measure) is by convention a
/// non-negative magnitude (the type system can't easily bound on
/// `Unsigned` because we use signed types like `f64`/`Decimal`
/// non-negatively by invariant). Primitive integer impls widen one
/// bit-class to the next unsigned type — `u8`/`i8` → `u16`,
/// `u16`/`i16` → `u32`, `u32`/`i32` → `u64`, `u64`/`usize`/`i64`/`isize`
/// → `u128`. The 128-bit primitives use `u128` as their own measure
/// (width fits; cardinality of `[MIN, MAX]` = 2¹²⁸ overflows and
/// surfaces as `Err`). Floats keep their own type (`f32`/`f64`).
///
/// # Design: `PartialOrd`, not `Ord`
///
/// `Element` deliberately requires only `PartialOrd`, **not** `Ord`.
/// Tightening to `Ord` would exclude `f32`/`f64` (which are `!Ord`
/// because of NaN), and float support is a core value proposition.
/// PartialOrd-only discrete types (Gaussian integers, integer
/// lattices, power-set posets) are also legitimate; total order is an
/// orthogonal axis. Operations that genuinely need total order add
/// `+ Ord` at the call site.
pub trait Element: Sized + PartialEq + PartialOrd {
    /// The element category — [`DiscreteKind`] or [`ContinuousKind`].
    type Kind: sealed::Kind;

    /// The natural additive measure of an interval over `Self` (count
    /// for discrete, Lebesgue width for continuous). Bound on
    /// `Zero + TryAdd + Clone` to drive the per-piece result and the
    /// `IntervalSet::try_measure` fold.
    type Measure: Zero + TryAdd<Self::Measure, Output = Self::Measure> + Clone;

    /// Return the element adjacent to `self` on `side`, or `None` if
    /// none exists (continuous types, or extremes of representable
    /// range on discrete types).
    fn try_adjacent(&self, side: Side) -> Option<Self>;

    /// Compute the natural measure of `[left, right]` (inclusive on
    /// both sides). `None` means representation overflow.
    fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure>;

    /// Validate (and optionally normalize) `self` as a finite bound
    /// value. Returns `Some(v)` to accept (where `v` is the canonical
    /// form to store) or `None` to reject. The default rejects values
    /// that are incomparable to themselves (NaN). Library float types
    /// override to additionally reject `±INF`.
    fn validate(self) -> Option<Self> {
        self.partial_cmp(&self).map(|_| self)
    }
}

/// Pure-marker subtrait — `T: DiscreteElement` is `T: Element<Kind = DiscreteKind>`.
pub trait DiscreteElement: Element<Kind = DiscreteKind> {}
impl<T: Element<Kind = DiscreteKind>> DiscreteElement for T {}

/// Pure-marker subtrait — `T: ContinuousElement` is `T: Element<Kind = ContinuousKind>`.
pub trait ContinuousElement: Element<Kind = ContinuousKind> {}
impl<T: Element<Kind = ContinuousKind>> ContinuousElement for T {}

/// Default implementation of [`Element::try_measure_finite`] for
/// discrete user types whose [`Measure`](Element::Measure) is `Self`
/// and that implement [`CheckedSub`](crate::numeric::CheckedSub). Use
/// inside a manual `Element` impl when the type's adjacency primitive
/// is already wired up:
///
/// ```ignore
/// fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
///     intervalsets_core::numeric::default_discrete_count_inclusive(left, right)
/// }
/// ```
///
/// Computes `(right + 1) - left` (falling back to `right - (left - 1)`
/// when `right` is at the type's upper limit), via
/// [`CheckedSub`](crate::numeric::CheckedSub). Returns `None` if both
/// adjacents are unrepresentable (e.g. both endpoints at the type's
/// limits, which means `[MIN, MAX]` for a primitive integer).
pub fn default_discrete_count_inclusive<T>(left: &T, right: &T) -> Option<T>
where
    T: Element + crate::numeric::CheckedSub,
{
    if let Some(upper) = right.try_adjacent(Side::Right) {
        return crate::numeric::CheckedSub::checked_sub(&upper, left);
    }
    if let Some(lower) = left.try_adjacent(Side::Left) {
        return crate::numeric::CheckedSub::checked_sub(right, &lower);
    }
    None
}

/// Implement [`Element`] for a continuous user type whose
/// [`Measure`](Element::Measure) equals `Self` and whose subtraction
/// is infallible (typical for arbitrary-precision types like
/// `BigDecimal`).
///
/// Sets `Kind = ContinuousKind`, `try_adjacent → None`,
/// `try_measure_finite → Some(right - left)`. `validate` keeps the
/// default (`partial_cmp(&self)` rejection of NaN-style values).
///
/// For native floats (which need an `is_finite()` check on both
/// `validate` and the subtraction result), see this file's
/// `float_element_impl!` macro — those impls are baked in for `f32` /
/// `f64`.
///
/// # Example
///
/// ```
/// use intervalsets_core::default_continuous_element_impl;
/// use intervalsets_core::prelude::*;
///
/// #[derive(Clone, PartialEq, PartialOrd)]
/// struct MyFloat(f64);
///
/// impl core::ops::Sub for MyFloat {
///     type Output = Self;
///     fn sub(self, rhs: Self) -> Self { MyFloat(self.0 - rhs.0) }
/// }
///
/// impl core::ops::Add for MyFloat {
///     type Output = Self;
///     fn add(self, rhs: Self) -> Self { MyFloat(self.0 + rhs.0) }
/// }
///
/// impl intervalsets_core::ops::math::TryAdd for MyFloat {
///     type Output = Self;
///     type Error = intervalsets_core::error::MathError;
///     fn try_add(self, rhs: Self) -> Result<Self, Self::Error> {
///         let r = self.0 + rhs.0;
///         if r.is_finite() { Ok(MyFloat(r)) }
///         else { Err(intervalsets_core::error::MathError::Domain) }
///     }
/// }
///
/// impl num_traits::Zero for MyFloat {
///     fn zero() -> Self { MyFloat(0.0) }
///     fn is_zero(&self) -> bool { self.0 == 0.0 }
/// }
///
/// default_continuous_element_impl!(MyFloat);
/// ```
#[macro_export]
macro_rules! default_continuous_element_impl {
    ($($t:ty),+ $(,)?) => {
        $(
            impl $crate::numeric::Element for $t {
                type Kind = $crate::numeric::ContinuousKind;
                type Measure = $t;

                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }

                #[inline]
                fn try_measure_finite(
                    left: &Self,
                    right: &Self,
                ) -> Option<Self::Measure> {
                    Some(right.clone() - left.clone())
                }
            }
        )+
    };
}

// Native floats override `validate` to reject non-finite (NaN, ±INF).
// `default_continuous_element_impl!` is reserved for types whose
// default `validate` is already correct (e.g. `BigDecimal`).
macro_rules! float_element_impl {
    ($($t:ty),+ $(,)?) => {
        $(
            impl $crate::numeric::Element for $t {
                type Kind = $crate::numeric::ContinuousKind;
                type Measure = $t;

                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }

                #[inline]
                fn try_measure_finite(
                    left: &Self,
                    right: &Self,
                ) -> Option<Self::Measure> {
                    let d = right - left;
                    d.is_finite().then_some(d)
                }

                #[inline]
                fn validate(self) -> Option<Self> {
                    self.is_finite().then_some(self)
                }
            }
        )+
    };
}

float_element_impl!(f32, f64);

// Primitive integer impls — stepwise widening to the next unsigned
// bit-class. The diff and the +1 always fit in `i128`/`u128` for any
// sub-128-bit primitive, and the final `TryFrom<u128>` lets the
// macro share its body across all narrower targets (the conversion
// is infallible for these widenings; we use TryFrom for uniformity).
macro_rules! primitive_integer_element_impl {
    ($($t:ty => $u:ty),+ $(,)?) => {
        $(
            impl $crate::numeric::Element for $t {
                type Kind = $crate::numeric::DiscreteKind;
                type Measure = $u;

                #[inline]
                fn try_adjacent(&self, side: $crate::bound::Side) -> Option<Self> {
                    match side {
                        $crate::bound::Side::Right => <$t>::checked_add(*self, 1),
                        $crate::bound::Side::Left => <$t>::checked_sub(*self, 1),
                    }
                }

                #[inline]
                fn try_measure_finite(
                    left: &Self,
                    right: &Self,
                ) -> Option<Self::Measure> {
                    // Interval invariant: right >= left. Widening to
                    // i128 first keeps the subtraction overflow-free;
                    // reinterpreting the non-negative diff as u128
                    // and adding 1 gives the inclusive count.
                    let diff = (*right as i128) - (*left as i128);
                    let count = (diff as u128).checked_add(1)?;
                    <$u as ::core::convert::TryFrom<u128>>::try_from(count).ok()
                }
            }
        )+
    };
}

primitive_integer_element_impl!(
    u8 => u16, i8 => u16,
    u16 => u32, i16 => u32,
    u32 => u64, i32 => u64,
    u64 => u128, i64 => u128,
    usize => u128, isize => u128,
);

// 128-bit types — can't widen further. Width fits in u128 exactly;
// cardinality of `[MIN, MAX]` = 2^128 overflows and returns `None`.

impl Element for u128 {
    type Kind = DiscreteKind;
    type Measure = u128;

    #[inline]
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Right => self.checked_add(1),
            Side::Left => self.checked_sub(1),
        }
    }

    #[inline]
    fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
        right.checked_sub(*left).and_then(|d| d.checked_add(1))
    }
}

impl Element for i128 {
    type Kind = DiscreteKind;
    type Measure = u128;

    #[inline]
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        match side {
            Side::Right => self.checked_add(1),
            Side::Left => self.checked_sub(1),
        }
    }

    #[inline]
    fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
        // Interval invariant: right >= left. The wrapping i128
        // difference reinterpreted as u128 yields the unsigned
        // distance, up to 2^128 - 1. The +1 (inclusive count)
        // overflows u128 exactly at `[i128::MIN, i128::MAX]`.
        let diff = right.wrapping_sub(*left) as u128;
        diff.checked_add(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_adjacent_discrete() {
        assert_eq!(10_i32.try_adjacent(Side::Right).unwrap(), 11);
        assert_eq!(11_i32.try_adjacent(Side::Left).unwrap(), 10);
        assert_eq!(i32::MAX.try_adjacent(Side::Right), None);
        assert_eq!(i32::MIN.try_adjacent(Side::Left), None);
    }

    #[test]
    fn try_adjacent_continuous_is_none() {
        assert_eq!(1.0_f64.try_adjacent(Side::Right), None);
        assert_eq!(1.0_f64.try_adjacent(Side::Left), None);
    }

    #[test]
    fn measure_primitive_widens_one_step() {
        // i32 → u64 under stepwise widening.
        let m: <i32 as Element>::Measure = i32::try_measure_finite(&0, &10).unwrap();
        assert_eq!(m, 11_u64);

        // [i32::MIN, i32::MAX] cardinality = 2^32 fits in u64.
        let full = i32::try_measure_finite(&i32::MIN, &i32::MAX).unwrap();
        assert_eq!(full, 1u64 << 32);
    }

    #[test]
    fn measure_u8_widens_to_u16() {
        let m: <u8 as Element>::Measure = u8::try_measure_finite(&0, &u8::MAX).unwrap();
        assert_eq!(m, 256_u16);
    }

    #[test]
    fn measure_i128_min_max_overflows() {
        // Cardinality 2^128 doesn't fit in u128.
        assert_eq!(i128::try_measure_finite(&i128::MIN, &i128::MAX), None);
    }

    #[test]
    fn measure_u128_min_max_overflows() {
        assert_eq!(u128::try_measure_finite(&0, &u128::MAX), None);
    }

    #[test]
    fn measure_continuous_float() {
        let w: <f64 as Element>::Measure = f64::try_measure_finite(&0.0, &10.5).unwrap();
        assert_eq!(w, 10.5);
    }

    #[test]
    fn measure_continuous_singleton_is_zero() {
        let w: <f64 as Element>::Measure = f64::try_measure_finite(&5.0, &5.0).unwrap();
        assert_eq!(w, 0.0);
    }

    #[test]
    fn measure_float_overflow_returns_none() {
        // f64::MIN - f64::MAX would be -INF, surfaced as None.
        assert_eq!(f64::try_measure_finite(&f64::MIN, &f64::MAX), None);
    }

    #[test]
    fn validate_rejects_nan_via_default() {
        assert_eq!(f64::NAN.validate(), None);
    }
}

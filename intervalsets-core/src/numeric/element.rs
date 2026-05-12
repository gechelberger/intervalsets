use crate::bound::Side;

/// Defines the data types whose elements make up a Set.
///
/// `try_adjacent` determines whether the elements are
/// treated as continuous or discrete data.
///
/// # Design: `PartialOrd`, not `Ord`
///
/// `Element` deliberately requires only `PartialOrd`, **not** `Ord`.
/// Tightening this bound to `Ord` would exclude `f32` and `f64` (which
/// are `!Ord` because of NaN), and float support is one of the crate's
/// core value propositions — much of the crate's complexity exists to
/// keep floats in the domain. Don't tighten this.
///
/// The crate handles NaN at runtime via [`TryCmp`](crate::try_cmp::TryCmp)
/// (a blanket impl over `T: PartialOrd` that returns
/// [`TotalOrderError`](crate::error::TotalOrderError) when
/// `partial_cmp` returns `None`). Validating constructors (`new`,
/// `try_new`, `Deserialize`) call `try_cmp` and reject NaN. Operations
/// that benefit from a stronger guarantee (set-op traits like `Union`)
/// add `T: Ord` as a separate per-trait bound, so callers using
/// integer-only types pay no NaN-checking cost while float users still
/// get a working API. The verbose `T: Element + Ord + Clone + Zero`-style
/// bounds elsewhere are deliberate; that's the cost of the split.
pub trait Element: Sized + PartialEq + PartialOrd {
    fn try_adjacent(&self, side: Side) -> Option<Self>;

    /// Validate (and optionally normalize) `self` as a `FiniteBound` value.
    ///
    /// Returns `Some(v)` to accept `self` (where `v` is the canonical
    /// form to store — possibly the same value), or `None` to reject.
    /// A `None` return collapses to
    /// [`Error::InvalidBoundLimit`](crate::error::Error::InvalidBoundLimit)
    /// at construction sites that funnel through
    /// [`FiniteBound::try_new`](crate::bound::FiniteBound::try_new).
    ///
    /// # Default behavior
    ///
    /// The default impl delegates to `self.partial_cmp(&self)`, which
    /// rejects values that are incomparable to themselves — i.e. NaN.
    /// This preserves the historical NaN-rejection behavior the crate's
    /// `try_cmp`-based validators relied on.
    ///
    /// # When to override
    ///
    /// Override when the type carries values that are comparable but
    /// not valid finite bound limits — most commonly intrinsic
    /// infinities. Library floats (`f32`, `f64`, `OrderedFloat<f*>`,
    /// `NotNan<f*>`) override to reject non-finite values via
    /// `is_finite()`. Discrete types (integers, `BigInt`/`BigUint`,
    /// `Decimal`, `BigDecimal`, `Fixed*`) keep the default — none have
    /// intrinsic infinities, and NaN is either nonexistent or already
    /// covered by `partial_cmp`.
    fn validate(self) -> Option<Self> {
        self.partial_cmp(&self).map(|_| self)
    }
}

/// Automatically implements [`Element`] for a type.
///
/// Interval/Set types require generic storage types to implement
/// the [`Element`] trait. It's primary function is to normalize **discrete**
/// data types.
///
/// For **continuous** data types, normalization is a **noop**, but the trait
/// still needs to be implemented to meet the trait bounds for Set types.
///
/// This macro provides a default impl for **continuous** types;
///
/// # Example
///
/// ```
/// use intervalsets_core::continuous_domain_impl;
/// use intervalsets_core::prelude::*;
///
/// #[derive(Clone, PartialEq, PartialOrd)]
/// struct MyFloat(f64);
///
/// continuous_domain_impl!(MyFloat);
/// ```
#[macro_export]
macro_rules! continuous_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Element for $t {
                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }
            }
        )+
    }
}

// Native floats override `validate` to reject non-finite (NaN, ±INF).
// `continuous_domain_impl!` is reserved for types whose default
// `validate` is already correct (e.g. `BigDecimal`).
macro_rules! float_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Element for $t {
                #[inline]
                fn try_adjacent(&self, _: $crate::bound::Side) -> Option<Self> {
                    None
                }

                #[inline]
                fn validate(self) -> Option<Self> {
                    self.is_finite().then_some(self)
                }
            }
        )+
    }
}

float_domain_impl!(f32, f64);

macro_rules! integer_domain_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Element for $t {
                #[inline]
                fn try_adjacent(&self, side: $crate::bound::Side) -> Option<Self> {
                    match side {
                        Side::Right => <$t>::checked_add(*self, 1),
                        Side::Left => <$t>::checked_sub(*self, 1),
                    }
                }
            }
        )+
    }
}

integer_domain_impl!(u8, u16, u32, u64, u128, usize);
integer_domain_impl!(i8, i16, i32, i64, i128, isize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacent() {
        assert_eq!(10.try_adjacent(Side::Right).unwrap(), 11);
        assert_eq!(11.try_adjacent(Side::Left).unwrap(), 10);
    }
}

/// Computes the midpoint (average) of two values.
///
/// The midpoint is the value equidistant from both inputs. For numeric
/// types this is conceptually `(self + other) / 2`, computed in a way
/// that does not overflow at the bounds of the type.
///
/// # Contract
///
/// Implementations should uphold the following:
///
/// 1. **No overflow.** The computation must not overflow even when
///    `self + other` would. Conceptually, evaluate as if in a
///    sufficiently-large arithmetic domain, then map back into `Self`.
/// 2. **Commutativity.** `a.midpoint(b) == b.midpoint(a)`.
/// 3. **Boundedness.** When inputs are comparable, the result lies
///    between them: `min(a, b) <= a.midpoint(b) <= max(a, b)`.
///
/// # Rounding
///
/// For std primitives this trait delegates to the inherent
/// `<T>::midpoint` method, so rounding behavior matches std exactly:
///
/// - **Integers** — rounded toward zero.
/// - **Floats** (`f32`, `f64`) — computed as if in extended precision,
///   then rounded to the nearest representable value
///   (round-to-nearest-even).
///
/// Custom types choose the rounding semantics appropriate to their
/// domain; this trait does not prescribe one. Downstream impls are
/// expected to document their own rounding, error, and edge-case
/// behavior.
///
/// # Errors
///
/// The associated [`Error`](Midpointable::Error) type is a user-extension
/// hook. Every library-provided impl uses
/// [`core::convert::Infallible`] except [`Decimal`](rust_decimal::Decimal),
/// whose bounded precision means rounding the two halves can push their
/// sum out of range; that impl returns [`MathError::Range`](crate::error::MathError::Range).
///
/// In-tree contract: values stored in any in-tree set type
/// (`FiniteInterval`, `HalfInterval`, `EnumInterval`, `Interval`) are
/// validated finite at construction (see [`Element::validate`](super::Element::validate)). When
/// `T::midpoint` is reached through the set-level
/// [`Midpoint`](crate::ops::Midpoint) trait, the inputs are guaranteed
/// finite, so library impls succeed.
///
/// Direct callers passing arbitrary values (e.g. `f32::midpoint(NAN, 0.0)`)
/// inherit std's `f*::midpoint` semantics — typically a NaN-tainted
/// result rather than an error.
pub trait Midpointable: Sized {
    type Error;
    fn midpoint(self, other: Self) -> Result<Self, Self::Error>;
}

macro_rules! integer_midpoint_delegate_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Midpointable for $t {
                type Error = ::core::convert::Infallible;

                /// Infallible: std's inherent integer `midpoint` is
                /// defined for every value in the type's range and
                /// cannot overflow.
                #[inline]
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
                    Ok(<$t>::midpoint(self, other))
                }
            }
        )+
    }
}

integer_midpoint_delegate_impl!(u8, u16, u32, u64, u128, usize);
integer_midpoint_delegate_impl!(i8, i16, i32, i64, i128, isize);

macro_rules! float_midpoint_delegate_impl {
    ($($t:ty), +) => {
        $(
            impl $crate::numeric::Midpointable for $t {
                type Error = ::core::convert::Infallible;

                /// Infallible by contract: values stored in any in-tree
                /// set type are validated finite at construction
                /// ([`Element::validate`](crate::numeric::Element::validate)
                /// rejects `±INF`/`NaN`), so the set-level
                /// [`Midpoint`](crate::ops::Midpoint) trait only ever
                /// reaches this impl with finite inputs. Delegates to
                /// the inherent float `midpoint`, which avoids spurious
                /// overflow/underflow at extremes.
                ///
                /// Direct callers passing non-finite values bypass the
                /// in-tree contract and inherit std's `f*::midpoint`
                /// semantics (typically a NaN-tainted result).
                #[inline]
                fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
                    Ok(<$t>::midpoint(self, other))
                }
            }
        )+
    }
}

float_midpoint_delegate_impl!(f32, f64);

#[cfg(test)]
mod tests {
    use super::*;

    // force resolution through trait
    fn get_midpoint<T: Midpointable>(a: T, b: T) -> Result<T, T::Error> {
        a.midpoint(b)
    }

    #[quickcheck]
    fn quickcheck_midpoint_i32(a: i32, b: i32) {
        let expected = (((a as i64) + (b as i64)) / 2) as i32;
        assert_eq!(get_midpoint(a, b).unwrap(), expected);
    }

    #[quickcheck]
    fn quickcheck_midpoint_f32(a: f32, b: f32) {
        if !a.is_finite() || !b.is_finite() {
            return;
        }
        // widen so the reference sum can't overflow f32 range
        let expected = (((a as f64) + (b as f64)) / 2.0) as f32;
        assert_eq!(get_midpoint(a, b).unwrap(), expected);
    }
}

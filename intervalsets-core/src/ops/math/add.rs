use core::ops::Add;

use super::TryAdd;
use crate::error::Error;
use crate::factory::traits::*;
use crate::numeric::Element;
use crate::EnumInterval::{self, Finite, Half, Unbounded};
use crate::{FiniteInterval, HalfInterval, MaybeEmpty};

// The infix Add operators below all require T: Ord (and the arithmetic
// output type to also be Ord). For Ord types, partial_cmp on bounds is
// total, so try_add is provably infallible and the .unwrap() can never
// panic. Float users without an Ord wrapper (e.g. OrderedFloat) must
// use TryAdd::try_add directly.

macro_rules! add_via_try {
    ($lhs:ty, $rhs:ty, $out:ty) => {
        impl<T> Add<$rhs> for $lhs
        where
            T: Add + Ord,
            <T as Add>::Output: Element + Ord,
        {
            type Output = $out;
            #[inline]
            fn add(self, rhs: $rhs) -> Self::Output {
                self.try_add(rhs).unwrap()
            }
        }
    };
}

add_via_try!(
    FiniteInterval<T>,
    FiniteInterval<T>,
    FiniteInterval<<T as Add>::Output>
);
add_via_try!(
    HalfInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    HalfInterval<T>,
    FiniteInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    EnumInterval<T>,
    FiniteInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    EnumInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    EnumInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    FiniteInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    FiniteInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Add>::Output>
);
add_via_try!(
    HalfInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Add>::Output>
);

impl<T> TryAdd for FiniteInterval<T>
where
    T: Add,
    <T as Add>::Output: Element,
{
    type Output = FiniteInterval<<T as Add>::Output>;
    type Error = Error;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        FiniteInterval::try_new(lhs_min + rhs_min, lhs_max + rhs_max)
    }
}

impl<T> TryAdd for HalfInterval<T>
where
    T: Add,
    <T as Add>::Output: Element,
{
    type Output = EnumInterval<<T as Add>::Output>;
    type Error = Error;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let (l_side, l_bound) = self.into_raw();
        let (r_side, r_bound) = rhs.into_raw();
        if l_side == r_side {
            EnumInterval::try_half_bounded(l_side, l_bound + r_bound)
        } else {
            Ok(EnumInterval::unbounded())
        }
    }
}

impl<T> TryAdd<FiniteInterval<T>> for HalfInterval<T>
where
    T: Add,
    <T as Add>::Output: Element,
{
    type Output = EnumInterval<<T as Add>::Output>;
    type Error = Error;

    #[inline]
    fn try_add(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        let Some((min, max)) = rhs.into_raw() else {
            return Ok(EnumInterval::empty());
        };

        let offset = self.side().select(min, max);
        let (side, bound) = self.into_raw();
        EnumInterval::try_half_bounded(side, bound + offset)
    }
}

macro_rules! dispatch_try_add_impl {
    ($t_rhs:ty) => {
        impl<T> TryAdd<$t_rhs> for EnumInterval<T>
        where
            T: Add,
            <T as Add>::Output: Element,
        {
            type Output = EnumInterval<<T as Add>::Output>;
            type Error = Error;

            #[inline]
            fn try_add(self, rhs: $t_rhs) -> Result<Self::Output, Self::Error> {
                match self {
                    Finite(inner) => inner.try_add(rhs).map(EnumInterval::from),
                    Half(inner) => inner.try_add(rhs).map(EnumInterval::from),
                    Unbounded => {
                        if rhs.is_empty() {
                            Ok(EnumInterval::empty())
                        } else {
                            Ok(Unbounded)
                        }
                    }
                }
            }
        }
    };
}

dispatch_try_add_impl!(FiniteInterval<T>);
dispatch_try_add_impl!(HalfInterval<T>);
dispatch_try_add_impl!(EnumInterval<T>);

macro_rules! commutative_try_add_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T> TryAdd<$t_rhs> for $t_lhs
        where
            T: Add,
            <T as Add>::Output: $crate::numeric::Element,
        {
            type Output = EnumInterval<<T as Add>::Output>;
            type Error = Error;

            #[inline]
            fn try_add(self, rhs: $t_rhs) -> Result<Self::Output, Self::Error> {
                rhs.try_add(self)
            }
        }
    };
}

commutative_try_add_impl!(FiniteInterval<T>, HalfInterval<T>);
commutative_try_add_impl!(FiniteInterval<T>, EnumInterval<T>);
commutative_try_add_impl!(HalfInterval<T>, EnumInterval<T>);

// === Value-level primitive impls (E2) ===
//
// These are net-new — set-level math above still binds on `core::ops::Add`,
// not `TryAdd`. E6 rebinds set-level math onto the value-level TryOp impls.

use super::macros::{impl_try_add_checked, impl_try_add_float_finite};

impl_try_add_checked!(i8);
impl_try_add_checked!(i16);
impl_try_add_checked!(i32);
impl_try_add_checked!(i64);
impl_try_add_checked!(i128);
impl_try_add_checked!(isize);
impl_try_add_checked!(u8);
impl_try_add_checked!(u16);
impl_try_add_checked!(u32);
impl_try_add_checked!(u64);
impl_try_add_checked!(u128);
impl_try_add_checked!(usize);

impl_try_add_float_finite!(f32);
impl_try_add_float_finite!(f64);

/// `Option<T>` delegates to the inner `T` impl. Any `None` operand
/// short-circuits to `Ok(None)`; `Some/Some` runs `T::try_add` and
/// re-wraps. The error type is `T::Error` (no widening) so users
/// wrapping a custom `T` keep their precise error.
impl<T: TryAdd> TryAdd for Option<T> {
    type Output = Option<<T as TryAdd>::Output>;
    type Error = <T as TryAdd>::Error;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (self, rhs) {
            (Some(a), Some(b)) => a.try_add(b).map(Some),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_add() {
        let x = EnumInterval::closed(100, 200);
        let y = EnumInterval::closed(100, 200);

        assert_eq!(x + y, EnumInterval::closed(200, 400));

        let y = EnumInterval::open(100, 200);
        assert_eq!(x + y, EnumInterval::open(200, 400));

        let y = EnumInterval::open_closed(100, 200);
        assert_eq!(x + y, EnumInterval::open_closed(200, 400));

        let e = EnumInterval::empty();
        assert_eq!(x + e, e);
        assert_eq!(e + x, e);
    }

    #[test]
    fn test_half_finite_add() {
        let x = EnumInterval::closed_unbound(100);
        let y = EnumInterval::closed(100, 200);
        let expected = EnumInterval::closed_unbound(200);
        assert_eq!(x + y, expected);
        assert_eq!(y + x, expected);

        let x = EnumInterval::unbound_closed(100);
        let y = EnumInterval::closed(100, 200);
        let expected = EnumInterval::unbound_closed(300);
        assert_eq!(x + y, expected);
        assert_eq!(y + x, expected);

        let e = EnumInterval::empty();
        assert_eq!(x + e, e);
        assert_eq!(e + x, e);
    }

    #[test]
    fn test_half_bounded_add() {
        let a = EnumInterval::closed_unbound(-10);
        let b = EnumInterval::closed_unbound(10);
        let expected = EnumInterval::closed_unbound(0);
        assert_eq!(a + b, expected);

        let c = EnumInterval::unbound_closed(10);
        assert_eq!(a + c, EnumInterval::unbounded());
    }

    #[test]
    fn test_unbounded_add() {
        let u = EnumInterval::<i32>::unbounded();
        assert_eq!(u + u, u);

        let x = EnumInterval::closed(100, 200);
        assert_eq!(x + u, u);
        assert_eq!(u + x, u);

        let x = EnumInterval::closed_unbound(100);
        assert_eq!(x + u, u);
        assert_eq!(u + x, u);

        let x = EnumInterval::empty();
        assert_eq!(u + x, x);
        assert_eq!(x + u, x);
    }

    /// Verify that OrderedFloat<f64> satisfies the infix Add operator
    /// bounds. Confirms wrapping floats with OrderedFloat restores
    /// access to the infix arithmetic operators.
    #[cfg(feature = "ordered-float")]
    #[test]
    fn test_ord_float_add() {
        use ordered_float::OrderedFloat as O;

        // finite + finite
        let x = EnumInterval::closed(O(100.0), O(200.0));
        let y = EnumInterval::closed(O(100.0), O(200.0));
        assert_eq!(x + y, EnumInterval::closed(O(200.0), O(400.0)));

        let y = EnumInterval::open(O(100.0), O(200.0));
        assert_eq!(x + y, EnumInterval::open(O(200.0), O(400.0)));

        // half + finite
        let h = EnumInterval::closed_unbound(O(100.0));
        let f = EnumInterval::closed(O(100.0), O(200.0));
        assert_eq!(h + f, EnumInterval::closed_unbound(O(200.0)));
        assert_eq!(f + h, EnumInterval::closed_unbound(O(200.0)));

        // half + half: same side = half-bounded, opposite = unbounded
        let a = EnumInterval::closed_unbound(O(-10.0));
        let b = EnumInterval::closed_unbound(O(10.0));
        assert_eq!(a + b, EnumInterval::closed_unbound(O(0.0)));

        let c = EnumInterval::unbound_closed(O(10.0));
        assert_eq!(a + c, EnumInterval::unbounded());

        // empty propagation
        let e = EnumInterval::empty();
        assert_eq!(x + e, e);
        assert_eq!(e + x, e);
    }

    // -- value-level primitive smoke tests (E2) --

    use crate::error::MathError;

    #[test]
    fn primitive_signed_add() {
        assert_eq!(<i32 as TryAdd>::try_add(1, 2), Ok(3));
        assert_eq!(<i32 as TryAdd>::try_add(i32::MAX, 1), Err(MathError::Range));
        assert_eq!(
            <i32 as TryAdd>::try_add(i32::MIN, -1),
            Err(MathError::Range)
        );
    }

    #[test]
    fn primitive_unsigned_add() {
        assert_eq!(<u32 as TryAdd>::try_add(1, 2), Ok(3));
        assert_eq!(<u32 as TryAdd>::try_add(u32::MAX, 1), Err(MathError::Range));
    }

    #[test]
    fn primitive_float_add() {
        assert_eq!(<f64 as TryAdd>::try_add(1.0, 2.0), Ok(3.0));
        assert_eq!(
            <f64 as TryAdd>::try_add(f64::MAX, f64::MAX),
            Err(MathError::Domain)
        );
        assert_eq!(
            <f64 as TryAdd>::try_add(f64::INFINITY, f64::NEG_INFINITY),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn option_add_some_some() {
        assert_eq!(Some(1_i32).try_add(Some(2)), Ok(Some(3)));
    }

    #[test]
    fn option_add_some_none() {
        assert_eq!(Some(1_i32).try_add(None), Ok(None));
    }

    #[test]
    fn option_add_none_some() {
        assert_eq!(None::<i32>.try_add(Some(2)), Ok(None));
    }

    #[test]
    fn option_add_none_none() {
        assert_eq!(None::<i32>.try_add(None), Ok(None));
    }

    #[test]
    fn option_add_propagates_inner_error() {
        // Inner overflow surfaces as the *inner* error type — Option<T>
        // does not widen `T::Error`.
        let r: Result<Option<i32>, MathError> = Some(i32::MAX).try_add(Some(1));
        assert_eq!(r, Err(MathError::Range));
    }
}

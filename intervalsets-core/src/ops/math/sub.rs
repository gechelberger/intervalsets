use core::ops::Sub;

use super::TrySub;
use crate::bound::FiniteBound;
use crate::error::Error;
use crate::factory::traits::*;
use crate::numeric::Element;
use crate::EnumInterval::{self, Finite, Half, Unbounded};
use crate::{FiniteInterval, HalfInterval, MaybeEmpty};

impl<T: Sub> Sub for FiniteBound<T> {
    type Output = FiniteBound<<T as Sub>::Output>;
    fn sub(self, rhs: Self) -> Self::Output {
        let (l_kind, l_val) = self.into_raw();
        let (r_kind, r_val) = rhs.into_raw();
        FiniteBound::new(l_kind.combine(r_kind), l_val - r_val)
    }
}

// The infix Sub operators below all require T: Ord (and the arithmetic
// output type to also be Ord). For Ord types, partial_cmp on bounds is
// total, so try_sub is provably infallible and the .unwrap() can never
// panic. Float users without an Ord wrapper (e.g. OrderedFloat) must
// use TrySub::try_sub directly.

macro_rules! sub_via_try {
    ($lhs:ty, $rhs:ty, $out:ty) => {
        impl<T> Sub<$rhs> for $lhs
        where
            T: Sub + Ord,
            <T as Sub>::Output: Element + Ord,
        {
            type Output = $out;
            #[inline]
            fn sub(self, rhs: $rhs) -> Self::Output {
                self.try_sub(rhs).unwrap()
            }
        }
    };
}

sub_via_try!(
    FiniteInterval<T>,
    FiniteInterval<T>,
    FiniteInterval<<T as Sub>::Output>
);
sub_via_try!(
    HalfInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    FiniteInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    HalfInterval<T>,
    FiniteInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    EnumInterval<T>,
    FiniteInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    EnumInterval<T>,
    HalfInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    EnumInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    FiniteInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Sub>::Output>
);
sub_via_try!(
    HalfInterval<T>,
    EnumInterval<T>,
    EnumInterval<<T as Sub>::Output>
);

impl<T> TrySub for FiniteInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = FiniteInterval<<T as Sub>::Output>;
    type Error = Error;

    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        FiniteInterval::try_new(lhs_min - rhs_max, lhs_max - rhs_min)
    }
}

impl<T> TrySub for HalfInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = EnumInterval<<T as Sub>::Output>;
    type Error = Error;

    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let (l_side, l_bound) = self.into_raw();
        let (r_side, r_bound) = rhs.into_raw();
        if l_side == r_side {
            Ok(EnumInterval::unbounded())
        } else {
            EnumInterval::try_half_bounded(l_side, l_bound - r_bound)
        }
    }
}

impl<T> TrySub<HalfInterval<T>> for FiniteInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = EnumInterval<<T as Sub>::Output>;
    type Error = Error;

    fn try_sub(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        // (a, b) - (c, ->) => (<-, b - c)
        // (a, b) - (<-, c) => (a - c, ->)
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Ok(EnumInterval::empty());
        };

        let (side, bound) = rhs.into_raw();
        let side = side.flip();
        let anchor = side.select(lhs_min, lhs_max);
        EnumInterval::try_half_bounded(side, anchor - bound)
    }
}

impl<T> TrySub<FiniteInterval<T>> for HalfInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = EnumInterval<<T as Sub>::Output>;
    type Error = Error;

    fn try_sub(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        // (<-, c) - (a, b) => (<-, c - a)
        // (c, ->) - (a, b) => (c - b, ->)
        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return Ok(EnumInterval::empty());
        };

        let (side, bound) = self.into_raw();
        let offset = side.select(rhs_max, rhs_min);
        EnumInterval::try_half_bounded(side, bound - offset)
    }
}

macro_rules! dispatch_lhs_try_sub_impl {
    ($t_rhs:ty) => {
        impl<T> TrySub<$t_rhs> for EnumInterval<T>
        where
            T: Sub,
            <T as Sub>::Output: Element,
        {
            type Output = EnumInterval<<T as Sub>::Output>;
            type Error = Error;

            #[inline]
            fn try_sub(self, rhs: $t_rhs) -> Result<Self::Output, Self::Error> {
                match self {
                    Finite(inner) => inner.try_sub(rhs).map(EnumInterval::from),
                    Half(inner) => inner.try_sub(rhs).map(EnumInterval::from),
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

dispatch_lhs_try_sub_impl!(FiniteInterval<T>);
dispatch_lhs_try_sub_impl!(HalfInterval<T>);
dispatch_lhs_try_sub_impl!(EnumInterval<T>);

macro_rules! dispatch_rhs_try_sub_impl {
    ($t_lhs:ty) => {
        impl<T> TrySub<EnumInterval<T>> for $t_lhs
        where
            T: Sub,
            <T as Sub>::Output: Element,
        {
            type Output = EnumInterval<<T as Sub>::Output>;
            type Error = Error;

            #[inline]
            fn try_sub(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
                match rhs {
                    Finite(rhs) => self.try_sub(rhs).map(EnumInterval::from),
                    Half(rhs) => self.try_sub(rhs).map(EnumInterval::from),
                    Unbounded => {
                        if self.is_empty() {
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

dispatch_rhs_try_sub_impl!(FiniteInterval<T>);
dispatch_rhs_try_sub_impl!(HalfInterval<T>);

// === Value-level primitive impls (E2) ===

use super::macros::{impl_try_sub_checked, impl_try_sub_float_finite};

impl_try_sub_checked!(i8);
impl_try_sub_checked!(i16);
impl_try_sub_checked!(i32);
impl_try_sub_checked!(i64);
impl_try_sub_checked!(i128);
impl_try_sub_checked!(isize);
impl_try_sub_checked!(u8);
impl_try_sub_checked!(u16);
impl_try_sub_checked!(u32);
impl_try_sub_checked!(u64);
impl_try_sub_checked!(u128);
impl_try_sub_checked!(usize);

impl_try_sub_float_finite!(f32);
impl_try_sub_float_finite!(f64);

/// `Option<T>` delegates to the inner `T` impl. See [`TryAdd`](super::TryAdd)'s
/// `Option` impl for the convention.
impl<T: TrySub> TrySub for Option<T> {
    type Output = Option<<T as TrySub>::Output>;
    type Error = <T as TrySub>::Error;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (self, rhs) {
            (Some(a), Some(b)) => a.try_sub(b).map(Some),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_sub() {
        let x = EnumInterval::closed(100, 200);
        let y = EnumInterval::closed(100, 200);
        assert_eq!(x - y, EnumInterval::closed(-100, 100));

        let y = EnumInterval::open_closed(100, 200);
        assert_eq!(x - y, EnumInterval::closed_open(-100, 100));

        let e = EnumInterval::empty();
        assert_eq!(x - e, e);
        assert_eq!(e - x, e);
    }

    #[test]
    fn test_half_sub() {
        let x = EnumInterval::closed_unbound(100);
        let y = EnumInterval::closed_unbound(100);
        assert_eq!(x - y, EnumInterval::unbounded());
        assert_eq!(y - x, EnumInterval::unbounded());

        let y = EnumInterval::unbound_open(100);
        assert_eq!(x - y, EnumInterval::open_unbound(0));

        let e = EnumInterval::empty();
        assert_eq!(x - e, e);
        assert_eq!(e - x, e);
    }

    #[test]
    fn test_finite_half_sub() {
        let x = EnumInterval::closed(0, 10);
        let y = EnumInterval::closed_unbound(7);
        assert_eq!(x - y, EnumInterval::unbound_closed(3));
        assert_eq!(y - x, EnumInterval::closed_unbound(-3));

        let x = EnumInterval::closed(-5, 5);
        let y = EnumInterval::unbound_open(-7);
        assert_eq!(x - y, EnumInterval::open_unbound(2));
        assert_eq!(y - x, EnumInterval::unbound_open(-2));
    }

    #[test]
    fn test_unbounded_sub() {
        let u = EnumInterval::<i32>::unbounded();
        assert_eq!(u - u, u);

        let x = EnumInterval::closed(100, 200);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::closed_unbound(100);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::unbound_closed(100);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::empty();
        assert_eq!(x - u, x);
        assert_eq!(u - x, x);
    }

    /// Verify that OrderedFloat<f64> satisfies the infix Sub operator
    /// bounds. Confirms wrapping floats with OrderedFloat restores
    /// access to the infix arithmetic operators.
    #[cfg(feature = "ordered-float")]
    #[test]
    fn test_ord_float_sub() {
        use ordered_float::OrderedFloat as O;

        // finite - finite
        let x = EnumInterval::closed(O(100.0), O(200.0));
        let y = EnumInterval::closed(O(100.0), O(200.0));
        assert_eq!(x - y, EnumInterval::closed(O(-100.0), O(100.0)));

        let y = EnumInterval::open_closed(O(100.0), O(200.0));
        assert_eq!(x - y, EnumInterval::closed_open(O(-100.0), O(100.0)));

        // half - half: same side = unbounded, opposite = half-bounded
        let cu = EnumInterval::closed_unbound(O(100.0));
        assert_eq!(cu - cu, EnumInterval::unbounded());

        let uo = EnumInterval::unbound_open(O(100.0));
        assert_eq!(cu - uo, EnumInterval::open_unbound(O(0.0)));

        // finite - half (and reverse)
        let x = EnumInterval::closed(O(0.0), O(10.0));
        let y = EnumInterval::closed_unbound(O(7.0));
        assert_eq!(x - y, EnumInterval::unbound_closed(O(3.0)));
        assert_eq!(y - x, EnumInterval::closed_unbound(O(-3.0)));

        // empty propagation
        let e = EnumInterval::empty();
        assert_eq!(x - e, e);
        assert_eq!(e - x, e);
    }

    // -- value-level primitive smoke tests (E2) --

    use crate::error::MathError;

    #[test]
    fn primitive_signed_sub() {
        assert_eq!(<i32 as TrySub>::try_sub(5, 3), Ok(2));
        assert_eq!(<i32 as TrySub>::try_sub(i32::MIN, 1), Err(MathError::Range));
    }

    #[test]
    fn primitive_unsigned_sub() {
        assert_eq!(<u32 as TrySub>::try_sub(5, 3), Ok(2));
        assert_eq!(<u32 as TrySub>::try_sub(0, 1), Err(MathError::Range));
    }

    #[test]
    fn primitive_float_sub() {
        assert_eq!(<f64 as TrySub>::try_sub(5.0, 3.0), Ok(2.0));
        assert_eq!(
            <f64 as TrySub>::try_sub(f64::INFINITY, f64::INFINITY),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn option_sub_matrix() {
        assert_eq!(Some(5_i32).try_sub(Some(3)), Ok(Some(2)));
        assert_eq!(Some(5_i32).try_sub(None), Ok(None));
        assert_eq!(None::<i32>.try_sub(Some(3)), Ok(None));
        assert_eq!(None::<i32>.try_sub(None), Ok(None));

        let r: Result<Option<u32>, MathError> = Some(0_u32).try_sub(Some(1));
        assert_eq!(r, Err(MathError::Range));
    }
}

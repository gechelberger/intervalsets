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

impl<T> Sub for FiniteInterval<T>
where
    T: Sub + Ord,
    <T as Sub>::Output: Element + Ord,
{
    type Output = FiniteInterval<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> Sub for HalfInterval<T>
where
    T: Sub + Ord,
    <T as Sub>::Output: Element + Ord,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> Sub<HalfInterval<T>> for FiniteInterval<T>
where
    T: Sub + Ord,
    <T as Sub>::Output: Element + Ord,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: HalfInterval<T>) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> Sub<FiniteInterval<T>> for HalfInterval<T>
where
    T: Sub + Ord,
    <T as Sub>::Output: Element + Ord,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: FiniteInterval<T>) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

macro_rules! dispatch_lhs_sub_impl {
    ($t_rhs:ty) => {
        impl<T> Sub<$t_rhs> for EnumInterval<T>
        where
            T: Sub + Ord,
            <T as Sub>::Output: Element + Ord,
        {
            type Output = EnumInterval<<T as Sub>::Output>;

            #[inline]
            fn sub(self, rhs: $t_rhs) -> Self::Output {
                self.try_sub(rhs).unwrap()
            }
        }
    };
}

dispatch_lhs_sub_impl!(FiniteInterval<T>);
dispatch_lhs_sub_impl!(HalfInterval<T>);
dispatch_lhs_sub_impl!(EnumInterval<T>);

macro_rules! dispatch_rhs_sub_impl {
    ($t_lhs:ty) => {
        impl<T> Sub<EnumInterval<T>> for $t_lhs
        where
            T: Sub + Ord,
            <T as Sub>::Output: Element + Ord,
        {
            type Output = EnumInterval<<T as Sub>::Output>;

            #[inline]
            fn sub(self, rhs: EnumInterval<T>) -> Self::Output {
                self.try_sub(rhs).unwrap()
            }
        }
    };
}

dispatch_rhs_sub_impl!(FiniteInterval<T>);
dispatch_rhs_sub_impl!(HalfInterval<T>);

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
}

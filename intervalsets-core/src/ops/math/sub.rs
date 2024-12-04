use core::ops::Sub;

use crate::bound::FiniteBound;
use crate::factory::traits::*;
use crate::numeric::{Element, Zero};
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

impl<T> Sub for FiniteInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = FiniteInterval<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return FiniteInterval::empty();
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return FiniteInterval::empty();
        };

        let min = lhs_min - rhs_max;
        let max = lhs_max - rhs_min;

        FiniteInterval::new(min, max)
    }
}

impl<T> Sub for HalfInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        //(a, ->) - (b, ->) => (a - inf, inf - b) => (<-, ->)
        //(<-, a) - (<-, b) => (-inf - b, a -- inf) => (<-, ->)
        //(<-, a) - (b, ->) => (-inf - inf, a - b) => (<-, a - b)
        //(a, ->) - (<-, b) => (a - b, inf - -inf) => (a - b, ->)
        let (l_side, l_bound) = self.into_raw();
        let (r_side, r_bound) = rhs.into_raw();
        if l_side == r_side {
            EnumInterval::unbounded()
        } else {
            EnumInterval::half_bounded(l_side, l_bound - r_bound)
        }
    }
}

impl<T> Sub<HalfInterval<T>> for FiniteInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: HalfInterval<T>) -> Self::Output {
        // (a, b) - (c, ->) => (a - inf, b - c) => (<-, b - c)
        // (a, b) - (<-, c) => (a - c, b -- inf) => (a - c, ->)
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return EnumInterval::empty();
        };

        let (side, bound) = rhs.into_raw();
        let side = side.flip();
        let anchor = side.select(lhs_min, lhs_max);
        EnumInterval::half_bounded(side, anchor - bound)
    }
}

impl<T> Sub<FiniteInterval<T>> for HalfInterval<T>
where
    T: Sub,
    <T as Sub>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Sub>::Output>;

    fn sub(self, rhs: FiniteInterval<T>) -> Self::Output {
        // (<-, c) - (a, b) => (-inf - b, c - a) => (<-, c - a)
        // (c, ->) - (a, b) => (c - b, inf - a) => (c - b, ->)
        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return EnumInterval::empty();
        };

        let (side, bound) = self.into_raw();
        let offset = side.select(rhs_max, rhs_min);
        EnumInterval::half_bounded(side, bound - offset)
    }
}

macro_rules! dispatch_lhs_sub_impl {
    ($t_rhs:ty) => {
        impl<T> Sub<$t_rhs> for EnumInterval<T>
        where
            T: Sub,
            <T as Sub>::Output: Element + Zero,
        {
            type Output = EnumInterval<<T as Sub>::Output>;

            #[inline]
            fn sub(self, rhs: $t_rhs) -> Self::Output {
                match self {
                    Finite(inner) => (inner - rhs).into(),
                    Half(inner) => (inner - rhs).into(),
                    Unbounded => {
                        if rhs.is_empty() {
                            EnumInterval::empty()
                        } else {
                            Unbounded
                        }
                    }
                }
            }
        }

        // by ref?
    };
}

dispatch_lhs_sub_impl!(FiniteInterval<T>);
dispatch_lhs_sub_impl!(HalfInterval<T>);
dispatch_lhs_sub_impl!(EnumInterval<T>);

macro_rules! dispatch_rhs_sub_impl {
    ($t_lhs:ty) => {
        impl<T> Sub<EnumInterval<T>> for $t_lhs
        where
            T: Sub,
            <T as Sub>::Output: Element + Zero,
        {
            type Output = EnumInterval<<T as Sub>::Output>;

            #[inline]
            fn sub(self, rhs: EnumInterval<T>) -> Self::Output {
                match rhs {
                    Finite(rhs) => (self - rhs).into(),
                    Half(rhs) => (self - rhs).into(),
                    Unbounded => {
                        if self.is_empty() {
                            EnumInterval::empty()
                        } else {
                            Unbounded
                        }
                    }
                }
            }
        }

        // by ref?
    };
}

dispatch_rhs_sub_impl!(FiniteInterval<T>);
dispatch_rhs_sub_impl!(HalfInterval<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_sub() {
        let x = EnumInterval::closed(100.0, 200.0);
        let y = EnumInterval::closed(100.0, 200.0);
        assert_eq!(x - y, EnumInterval::closed(-100.0, 100.0));

        let y = EnumInterval::open_closed(100.0, 200.0);
        assert_eq!(x - y, EnumInterval::closed_open(-100.0, 100.0));

        let e = EnumInterval::empty();
        assert_eq!(x - e, e);
        assert_eq!(e - x, e);
    }

    #[test]
    fn test_half_sub() {
        let x = EnumInterval::closed_unbound(100.0);
        let y = EnumInterval::closed_unbound(100.0);
        assert_eq!(x - y, EnumInterval::unbounded());
        assert_eq!(y - x, EnumInterval::unbounded());

        let y = EnumInterval::unbound_open(100.0);
        assert_eq!(x - y, EnumInterval::open_unbound(0.0));

        let e = EnumInterval::empty();
        assert_eq!(x - e, e);
        assert_eq!(e - x, e);
    }

    #[test]
    fn test_finite_half_sub() {
        let x = EnumInterval::closed(0.0, 10.0);
        let y = EnumInterval::closed_unbound(7.0);
        assert_eq!(x - y, EnumInterval::unbound_closed(3.0));
        assert_eq!(y - x, EnumInterval::closed_unbound(-3.0));

        let x = EnumInterval::closed(-5.0, 5.0);
        let y = EnumInterval::unbound_open(-7.0);
        assert_eq!(x - y, EnumInterval::open_unbound(2.0));
        assert_eq!(y - x, EnumInterval::unbound_open(-2.0));
    }

    #[test]
    fn test_unbounded_sub() {
        let u = EnumInterval::<f32>::unbounded();
        assert_eq!(u - u, u);

        let x = EnumInterval::closed(100.0, 200.0);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::closed_unbound(100.0);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::unbound_closed(100.0);
        assert_eq!(x - u, u);
        assert_eq!(u - x, u);

        let x = EnumInterval::empty();
        assert_eq!(x - u, x);
        assert_eq!(u - x, x);
    }
}

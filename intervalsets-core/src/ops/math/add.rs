use core::ops::Add;

use crate::factory::traits::*;
use crate::numeric::{Element, Zero};
use crate::EnumInterval::{self, Finite, Half, Unbounded};
use crate::{FiniteInterval, HalfInterval, MaybeEmpty};

impl<T> Add for FiniteInterval<T>
where
    T: Add,
    <T as Add>::Output: Element,
{
    type Output = FiniteInterval<<T as Add>::Output>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return FiniteInterval::empty();
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return FiniteInterval::empty();
        };

        let min = lhs_min + rhs_min;
        let max = lhs_max + rhs_max;

        FiniteInterval::new(min, max)
    }
}

impl<T> Add for HalfInterval<T>
where
    T: Add,
    <T as Add>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Add>::Output>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let (l_side, l_bound) = self.into_raw();
        let (r_side, r_bound) = rhs.into_raw();
        if l_side == r_side {
            EnumInterval::half_bounded(l_side, l_bound + r_bound)
        } else {
            EnumInterval::unbounded()
        }
    }
}

impl<T> Add<FiniteInterval<T>> for HalfInterval<T>
where
    T: Add,
    <T as Add>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Add>::Output>;

    #[inline]
    fn add(self, rhs: FiniteInterval<T>) -> Self::Output {
        let Some((min, max)) = rhs.into_raw() else {
            return EnumInterval::empty();
        };

        let offset = self.side().select(min, max);
        let (side, bound) = self.into_raw();
        EnumInterval::half_bounded(side, bound + offset)
    }
}

macro_rules! dispatch_add_impl {
    ($t_rhs:ty) => {
        impl<T> Add<$t_rhs> for EnumInterval<T>
        where
            T: Add,
            <T as Add>::Output: Element + Zero,
        {
            type Output = EnumInterval<<T as Add>::Output>;

            #[inline]
            fn add(self, rhs: $t_rhs) -> Self::Output {
                match self {
                    Finite(inner) => (inner + rhs).into(),
                    Half(inner) => (inner + rhs).into(),
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

dispatch_add_impl!(FiniteInterval<T>);
dispatch_add_impl!(HalfInterval<T>);
dispatch_add_impl!(EnumInterval<T>);

macro_rules! commutative_add_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T> Add<$t_rhs> for $t_lhs
        where
            T: Add,
            <T as Add>::Output: $crate::numeric::Element,
            <T as Add>::Output: $crate::numeric::Zero,
        {
            type Output = EnumInterval<<T as Add>::Output>;

            #[inline]
            fn add(self, rhs: $t_rhs) -> Self::Output {
                rhs + self
            }
        }
    };
}

commutative_add_impl!(FiniteInterval<T>, HalfInterval<T>);
commutative_add_impl!(FiniteInterval<T>, EnumInterval<T>);
commutative_add_impl!(HalfInterval<T>, EnumInterval<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_add() {
        let x = EnumInterval::closed(100.0, 200.0);
        let y = EnumInterval::closed(100.0, 200.0);

        assert_eq!(x + y, EnumInterval::closed(200.0, 400.0));

        let y = EnumInterval::open(100.0, 200.0);
        assert_eq!(x + y, EnumInterval::open(200.0, 400.0));

        let y = EnumInterval::open_closed(100.0, 200.0);
        assert_eq!(x + y, EnumInterval::open_closed(200.0, 400.0));
    }

    #[test]
    fn test_half_finite_add() {
        let x = EnumInterval::closed_unbound(100.0);
        let y = EnumInterval::closed(100.0, 200.0);
        let expected = EnumInterval::closed_unbound(200.0);
        assert_eq!(x + y, expected);
        assert_eq!(y + x, expected);

        let x = EnumInterval::unbound_closed(100.0);
        let y = EnumInterval::closed(100.0, 200.0);
        let expected = EnumInterval::unbound_closed(300.0);
        assert_eq!(x + y, expected);
        assert_eq!(y + x, expected);
    }

    #[test]
    fn test_unbounded_add() {
        let u = EnumInterval::unbounded();
        assert_eq!(u + u, u);

        let x = EnumInterval::closed(100.0, 200.0);
        assert_eq!(x + u, u);
        assert_eq!(u + x, u);

        let x = EnumInterval::closed_unbound(100.0);
        assert_eq!(x + u, u);
        assert_eq!(u + x, u);

        let x = EnumInterval::empty();
        assert_eq!(u + x, x);
        assert_eq!(x + u, x);
    }
}

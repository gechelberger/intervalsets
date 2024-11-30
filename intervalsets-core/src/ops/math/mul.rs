use core::ops::Mul;

use crate::bound::{FiniteBound, Side};
use crate::factory::{HalfBoundedFactory, UnboundedFactory};
use crate::numeric::{Element, Zero};
use crate::ops::Connects;
use crate::{EnumInterval, FiniteInterval, HalfInterval};


impl<T> Mul for FiniteInterval<T>
where
    T: Mul + Clone + PartialOrd + Zero,
    <T as Mul>::Output: Element + Zero,
{
    type Output = FiniteInterval<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return FiniteInterval::empty();
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return FiniteInterval::empty();
        };

        let t_zero = T::zero();
        if lhs_min.value() >= &t_zero && rhs_min.value() >= &t_zero {
            let min = lhs_min * rhs_min;
            let max = lhs_max * rhs_max;
            return FiniteInterval::new(min, max);
        } else if lhs_max.value() <= &t_zero && rhs_max.value() <= &t_zero {
            let min = lhs_max * rhs_max;
            let max = lhs_min * rhs_min;
            return FiniteInterval::new(min, max);
        }

        let a = lhs_min.clone() * rhs_min.clone();
        let b = lhs_min * rhs_max.clone();
        let c = lhs_max.clone() * rhs_min;
        let d = lhs_max * rhs_max;

        unsafe {
            let (min, ab) = FiniteBound::min_max_unchecked(Side::Left, a, b);
            let (min, abc) = FiniteBound::min_max_unchecked(Side::Left, min, c);
            let (min, abcd) = FiniteBound::min_max_unchecked(Side::Left, min, d);
            let max = FiniteBound::take_max_unchecked(Side::Right, ab, abc);
            let max = FiniteBound::take_max_unchecked(Side::Right, max, abcd);
            FiniteInterval::new_unchecked(min, max)
        }
    }
}

fn half_x_half<T>(a: HalfInterval<T>, b: HalfInterval<T>)
where 
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element
{
    todo!()
}

impl<T> Mul for HalfInterval<T>
where
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        let zero = T::zero();

        if self.side() == rhs.side() {
            let (non_neg, non_pos) = {
                let l = self.finite_bound().value();
                let r = rhs.finite_bound().value();
                (l >= &zero && r >= &zero, l <= &zero && r <= &zero)
            };

            if (self.side() == Side::Left && non_neg) || (self.side() == Side::Right && non_pos) {
                let (_, l_bound) = self.into_raw();
                let (_, r_bound) = rhs.into_raw();
                EnumInterval::half_bounded(Side::Left, l_bound * r_bound)
            } else {
                EnumInterval::unbounded()
            }
        } else {
            let (pos, neg) = {
                let l = self.finite_bound().value();
                let r = rhs.finite_bound().value();
                (l > &zero && r > &zero, l < &zero && r < &zero)
            };

            if pos || neg || self.connects(&rhs) {
                EnumInterval::unbounded()
            } else {
                let (_, l_bound) = self.into_raw();
                let (_, r_bound) = rhs.into_raw();
                EnumInterval::half_bounded(Side::Right, l_bound * r_bound)
            }
        }
    }
}

impl<T> Mul<HalfInterval<T>> for FiniteInterval<T> 
where 
    T: Mul,
    <T as Mul>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: HalfInterval<T>) -> Self::Output {
        

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_finite_x_finite() {
        let x = FiniteInterval::closed(0.0, 10.0);
        assert_eq!(x * x, FiniteInterval::closed(0.0, 100.0));

        let x = FiniteInterval::closed(5.0, 10.0);
        assert_eq!(x * x, FiniteInterval::closed(25.0, 100.0));

        let y = FiniteInterval::closed(-10.0, -5.0);
        assert_eq!(x * y, FiniteInterval::closed(-100.0, -25.0));
        assert_eq!(y * y, FiniteInterval::closed(25.0, 100.0));

        let a = FiniteInterval::open(-10.0, 0.0);
        let b = FiniteInterval::open(0.0, 10.0);
        assert_eq!(a * b, FiniteInterval::open(-100.0, 0.0));

        let a = FiniteInterval::closed(-10.0, 10.0);
        assert_eq!(a * a, FiniteInterval::closed(-100.0, 100.0));
    }

    #[test]
    fn test_half_x_half() {
        let u = EnumInterval::unbounded();

        let x = HalfInterval::closed_unbound(-1.0);
        assert_eq!(x * x, u);

        let x = HalfInterval::unbound_closed(1.0);
        assert_eq!(x * x, u);

        let x = HalfInterval::open_unbound(10.0);
        let expected = EnumInterval::open_unbound(100.0);
        assert_eq!(x * x, expected);

        let x = HalfInterval::unbound_open(-10.0);
        assert_eq!(x * x, expected);

        let x = HalfInterval::unbound_closed(0.0);
        let expected = EnumInterval::closed_unbound(0.0);
        assert_eq!(x * x, expected);

        let x = HalfInterval::closed_unbound(0.0);
        assert_eq!(x * x, expected);

        assert_eq!(
            HalfInterval::unbound_closed(-5.0) * HalfInterval::closed_unbound(10.0),
            EnumInterval::unbound_closed(-50.0)
        );

        let a = HalfInterval::unbound_open(0.0);
        let b = HalfInterval::open_unbound(0.0);
        let expected = EnumInterval::unbound_open(0.0);
        assert_eq!(a * b, expected);
        assert_eq!(b * a, expected);

        let b = HalfInterval::closed_unbound(0.0);
        assert_eq!(a * b, u);
        assert_eq!(b * a, u);
    }
}

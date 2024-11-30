use core::cmp::Ordering::{Equal, Greater, Less};
use core::ops::Mul;

use crate::bound::ord::{FiniteOrdBound, FiniteOrdBoundKind};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};
use crate::numeric::{Element, Zero};
use crate::ops::Connects;
use crate::{EnumInterval, FiniteInterval, HalfInterval};

enum MaybeZero {
    Zero,
    NonZero,
}

enum MCat {
    Empty,
    Zero,
    NegPos,
    Pos(MaybeZero),
    Neg(MaybeZero),
}

impl<T: Zero + PartialOrd> FiniteInterval<T> {
    #[inline]
    fn mul_category(&self) -> MCat {
        let Some((lhs, rhs)) = self.view_raw() else {
            return MCat::Empty;
        };

        let zero = T::zero();
        match lhs.value().partial_cmp(&zero).unwrap() {
            Greater => MCat::Pos(MaybeZero::NonZero),
            Equal => match rhs.value().partial_cmp(&zero).unwrap() {
                Greater => MCat::Pos(MaybeZero::Zero),
                Equal => MCat::Zero,
                Less => unreachable!(),
            },
            Less => match rhs.value().partial_cmp(&zero).unwrap() {
                Greater => MCat::NegPos,
                Equal => MCat::Neg(MaybeZero::Zero),
                Less => MCat::Neg(MaybeZero::NonZero),
            },
        }
    }
}

impl<T: Zero + PartialOrd> HalfInterval<T> {
    #[inline]
    fn mul_category(&self) -> MCat {
        let t_zero = T::zero();
        let zero = FiniteOrdBound::closed(&t_zero);
        match self.side() {
            Left => match self.finite_ord_bound().partial_cmp(&zero).unwrap() {
                Less => MCat::NegPos,
                Equal => MCat::Pos(MaybeZero::Zero),
                Greater => MCat::Pos(MaybeZero::NonZero),
            },
            Right => match self.finite_ord_bound().partial_cmp(&zero).unwrap() {
                Less => MCat::Neg(MaybeZero::NonZero),
                Equal => MCat::Neg(MaybeZero::Zero),
                Greater => MCat::NegPos,
            },
        }
    }
}

impl<T: Zero + PartialOrd> EnumInterval<T> {
    fn mul_category(&self) -> MCat {
        match self {
            Self::Finite(inner) => inner.mul_category(),
            Self::Half(inner) => inner.mul_category(),
            Self::Unbounded => MCat::NegPos,
        }
    }
}

impl<T> Mul for FiniteOrdBound<T>
where
    T: Mul + Zero + PartialOrd,
{
    type Output = FiniteOrdBound<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        let zero = T::zero();

        // lv*rv + lv*re + rv*le

        fn sign_assign<T: PartialOrd>(pivot: &T, val: &T, pos: i32, neg: i32) -> i32 {
            match val.partial_cmp(pivot).unwrap() {
                Greater => pos,
                Equal => 0,
                Less => neg,
            }
        }

        // LeftOpen => +1, Equal => 0, RightOpen => -1
        let e_lhs = self.1 as i32;
        let e_rhs = rhs.1 as i32;

        let e1 =
            sign_assign(&zero, &rhs.0, e_lhs, -e_lhs) + sign_assign(&zero, &self.0, e_rhs, -e_rhs);

        let epsilons = (e1, e_lhs * e_rhs);

        let value = self.0 * rhs.0;
        match epsilons.cmp(&(0, 0)) {
            Greater => FiniteOrdBound::new(value, FiniteOrdBoundKind::LeftOpen),
            Less => FiniteOrdBound::new(value, FiniteOrdBoundKind::RightOpen),
            Equal => FiniteOrdBound::closed(value),
        }
    }
}

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
    <T as Mul>::Output: Element,
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
    T: Mul + PartialOrd + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    #[inline]
    fn mul(self, rhs: HalfInterval<T>) -> Self::Output {
        let fcat = self.mul_category();
        let Some((fmin, fmax)) = self.into_raw() else {
            return EnumInterval::empty();
        };

        let hcat = rhs.mul_category();
        let (side, hbound) = rhs.into_raw();

        match (fcat, hcat) {
            (MCat::Pos(_), MCat::Pos(_)) => EnumInterval::half_bounded(side, fmin * hbound),
            (MCat::Pos(_), MCat::NegPos) => EnumInterval::half_bounded(side, fmax * hbound),
            (MCat::Pos(_), MCat::Neg(_)) => EnumInterval::half_bounded(side, fmax * hbound),
            (MCat::Neg(_), MCat::Pos(_)) => EnumInterval::half_bounded(Right, fmax * hbound),
            (MCat::Neg(_), MCat::NegPos) => EnumInterval::half_bounded(side.flip(), fmin * hbound),
            (MCat::NegPos, _) => EnumInterval::unbounded(),
            (MCat::Zero, _) => EnumInterval::singleton(<T as Mul>::Output::zero()),
            _ => unreachable!(),
        }
    }
}

impl<T> Mul<FiniteInterval<T>> for HalfInterval<T>
where
    T: Mul + PartialOrd + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: FiniteInterval<T>) -> Self::Output {
        rhs * self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::Side::{Left, Right};
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

    #[test]
    fn test_finite_ord_bound() {
        //
        let a = FiniteOrdBound::open(Side::Left, 5.0);
        let b = FiniteOrdBound::open(Side::Left, 10.0);
        assert_eq!(a * b, FiniteOrdBound::open(Side::Left, 50.0));

        let b = FiniteOrdBound::open(Side::Right, 5.0);
        assert_eq!(a * b, FiniteOrdBound::open(Side::Right, 25.0));

        let a = FiniteOrdBound::open(Left, -5.0);
        // -5+e * 5-e => -25 +5e + 5e - e^2
        assert_eq!(a * b, FiniteOrdBound::open(Side::Left, -25.0));

        // LO(+) * LO(+) => 3+e * 5+e => 15 + 8e + e2 => 15 + 8e => LO(15)
        let a = FiniteOrdBound::open(Left, 3.0);
        let b = FiniteOrdBound::open(Left, 5.0);
        assert_eq!(a * b, FiniteOrdBound::open(Left, 15.0));

        // LO(+) * LO(-) => 3+e * -5+e => -15 -2e + e2 => -15 -2e => RO(-15)
        let a = FiniteOrdBound::open(Left, 3.0);
        let b = FiniteOrdBound::open(Left, -5.0);
        assert_eq!(a * b, FiniteOrdBound::open(Right, -15.0));
        // LO(-) * LO(-)

        // RO(+) * RO(+)
        // RO(+) * RO(-) == RO(-) * RO(-)
        // RO(-) * RO(-)

        // LO(+) * RO(+)
        // LO(-) * RO(+)
        // LO(+) * RO(-)
        // LO(-) * RO(-)
    }
}

use core::cmp::Ordering::{Equal, Greater, Less};
use core::ops::Mul;

use crate::bound::ord::FiniteOrdBound;
use crate::bound::FiniteBound;
use crate::bound::Side::{Left, Right};
use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};
use crate::numeric::{Element, Zero};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl<T> Mul for FiniteInterval<T>
where
    T: Mul + Clone + PartialOrd + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = FiniteInterval<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        impls::finite_x_finite_by_cat(self, rhs)
    }
}

impl<T> Mul for HalfInterval<T>
where
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element + Zero,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        impls::half_x_half_by_cat(self, rhs)
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
        impls::finite_x_half(self, rhs)
    }
}

impl<T> Mul<FiniteInterval<T>> for HalfInterval<T>
where
    T: Mul + PartialOrd + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: FiniteInterval<T>) -> Self::Output {
        impls::finite_x_half(rhs, self)
    }
}

impl<T> Mul<FiniteInterval<T>> for EnumInterval<T>
where
    T: Mul + PartialOrd + Zero + Clone,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: FiniteInterval<T>) -> Self::Output {
        impls::enum_x_finite(self, rhs)
    }
}

impl<T> Mul<HalfInterval<T>> for EnumInterval<T>
where
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: HalfInterval<T>) -> Self::Output {
        impls::enum_x_half(self, rhs)
    }
}

impl<T> Mul<EnumInterval<T>> for EnumInterval<T>
where
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: EnumInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs * rhs,
            Self::Half(lhs) => lhs * rhs,
            Self::Unbounded => match rhs {
                Self::Finite(rhs) => self * rhs,
                Self::Half(rhs) => self * rhs,
                Self::Unbounded => EnumInterval::Unbounded,
            },
        }
    }
}

impl<T> Mul<EnumInterval<T>> for FiniteInterval<T>
where
    T: Mul + PartialOrd + Zero + Clone,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: EnumInterval<T>) -> Self::Output {
        impls::enum_x_finite(rhs, self)
    }
}

impl<T> Mul<EnumInterval<T>> for HalfInterval<T>
where
    T: Mul + Element + Zero + Clone,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = EnumInterval<<T as Mul>::Output>;

    fn mul(self, rhs: EnumInterval<T>) -> Self::Output {
        impls::enum_x_half(rhs, self)
    }
}

// This is only public for benchmarking access
#[doc(hidden)]
pub mod impls {
    use super::*;
    use crate::bound::FiniteBound as FB;
    use crate::{EnumInterval, FiniteInterval};

    /// Multiply two non-zero bounds together.
    ///
    /// # Safety
    ///
    /// The user must ensure that a Closed(Zero) bound is not passed.
    /// Closed(0) * Open(5) will return Open(5) which is wrong.
    #[inline(always)]
    unsafe fn non_zero_mul_unchecked<T: Mul>(a: FB<T>, b: FB<T>) -> FB<<T as Mul>::Output> {
        let (akind, aval) = a.into_raw();
        let (bkind, bval) = b.into_raw();
        FiniteBound::new(akind.combine(bkind), aval * bval)
    }

    pub fn finite_x_finite_by_cat<T>(
        a: FiniteInterval<T>,
        b: FiniteInterval<T>,
    ) -> FiniteInterval<<T as Mul>::Output>
    where
        T: Mul + PartialOrd + Clone + Zero,
        <T as Mul>::Output: Element + Zero + Clone,
    {
        let acat = a.mul_category();
        let bcat = b.mul_category();

        let Some((amin, amax)) = a.into_raw() else {
            return FiniteInterval::empty();
        };

        let Some((bmin, bmax)) = b.into_raw() else {
            return FiniteInterval::empty();
        };

        match (acat, bcat) {
            (MCat::Pos(az), MCat::Pos(bz)) => unsafe {
                // [a=0?, b>0] x [c=0?, d>0]
                let max = non_zero_mul_unchecked(amax, bmax);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(FiniteBound::zero(), max)
                } else {
                    let min = non_zero_mul_unchecked(amin, bmin);
                    FiniteInterval::new(min, max)
                }
            },
            (MCat::Pos(_), MCat::NegPos) => unsafe {
                // [a=0?, b>0] x [c<0, d>0] => a produces intermediate values
                let min = non_zero_mul_unchecked(amax.clone(), bmin);
                let max = non_zero_mul_unchecked(amax, bmax);
                FiniteInterval::new(min, max)
            },
            (MCat::Pos(az), MCat::Neg(bz)) => unsafe {
                // [a=0?, b>0] x [c<0, d=0?]
                let min = non_zero_mul_unchecked(amax, bmin);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(min, FiniteBound::zero())
                } else {
                    let max = non_zero_mul_unchecked(amin, bmax);
                    FiniteInterval::new(min, max)
                }
            },
            (MCat::Neg(az), MCat::Pos(bz)) => unsafe {
                // [a<0, b=0?] x [c=0?, d>0]
                let min = non_zero_mul_unchecked(amin, bmax);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(min, FiniteBound::zero())
                } else {
                    let max = non_zero_mul_unchecked(amax, bmin);
                    FiniteInterval::new(min, max)
                }
            },
            (MCat::Neg(_), MCat::NegPos) => unsafe {
                // [a<0, b=0?] x [c<0, d>0] => b produces intermediate values
                let min = non_zero_mul_unchecked(amin.clone(), bmax);
                let max = non_zero_mul_unchecked(amin, bmin);
                FiniteInterval::new(min, max)
            },
            (MCat::Neg(az), MCat::Neg(bz)) => unsafe {
                // [a<0, b=0?] x [c<0, d=0?]
                let max = non_zero_mul_unchecked(amin, bmin);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(FiniteBound::zero(), max)
                } else {
                    let min = non_zero_mul_unchecked(amax, bmax);
                    FiniteInterval::new(min, max)
                }
            },
            (MCat::NegPos, MCat::Pos(_)) => unsafe {
                // [a<0, b>0] x [c=0?, d>0] => c produces intermediate values
                let min = non_zero_mul_unchecked(amin, bmax.clone());
                let max = non_zero_mul_unchecked(amax, bmax);
                FiniteInterval::new(min, max)
            },
            (MCat::NegPos, MCat::Neg(_)) => unsafe {
                // [a<0, b>0] x [c<0, d=0?] => d produces intermediate values
                let min = non_zero_mul_unchecked(amax, bmin.clone());
                let max = non_zero_mul_unchecked(amin, bmin);
                FiniteInterval::new(min, max)
            },
            (MCat::NegPos, MCat::NegPos) => unsafe {
                // SAFETY: NegPos category can not have an end bound of Closed(0)
                let c1_min = non_zero_mul_unchecked(amin.clone(), bmax.clone());
                let c2_min = non_zero_mul_unchecked(amax.clone(), bmin.clone());
                let c1_max = non_zero_mul_unchecked(amin, bmin);
                let c2_max = non_zero_mul_unchecked(amax, bmax);
                let min = FiniteBound::take_min_unchecked(Left, c1_min, c2_min);
                let max = FiniteBound::take_max_unchecked(Right, c1_max, c2_max);
                FiniteInterval::new(min, max)
            },
            (MCat::Zero, _) | (_, MCat::Zero) => {
                FiniteInterval::singleton(<T as Mul>::Output::zero())
            }
            _ => unreachable!(),
        }
    }

    pub fn half_x_half_by_cat<T>(
        a: HalfInterval<T>,
        b: HalfInterval<T>,
    ) -> EnumInterval<<T as Mul>::Output>
    where
        T: Mul + PartialOrd + Zero,
        <T as Mul>::Output: Element + Zero,
    {
        let acat = a.mul_category();
        let bcat = b.mul_category();

        let (_, abound) = a.into_raw();
        let (_, bbound) = b.into_raw();

        match (acat, bcat) {
            (MCat::NegPos, _) | (_, MCat::NegPos) => EnumInterval::unbounded(),
            (MCat::Pos(az), MCat::Pos(bz)) | (MCat::Neg(az), MCat::Neg(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // SAFETY: (a > 0 && b > 0) || (a < 0 && b < 0)
                    let min = unsafe { non_zero_mul_unchecked(abound, bbound) };
                    EnumInterval::half_bounded(Left, min)
                }
            }
            (MCat::Pos(az), MCat::Neg(bz)) | (MCat::Neg(az), MCat::Pos(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::unbound_closed(<T as Mul>::Output::zero())
                } else {
                    // SAFETY: (a > 0 && b < 0) or (a < 0 && b > 0)
                    let max = unsafe { non_zero_mul_unchecked(abound, bbound) };
                    EnumInterval::half_bounded(Right, max)
                }
            }
            _ => unreachable!(), // zero, empty should be unreachable
        }
    }

    pub fn finite_x_half<T>(
        a: FiniteInterval<T>,
        b: HalfInterval<T>,
    ) -> EnumInterval<<T as Mul>::Output>
    where
        T: Mul + PartialOrd + Zero,
        <T as Mul>::Output: Element + Zero + Clone,
    {
        let fcat = a.mul_category();
        let Some((fmin, fmax)) = a.into_raw() else {
            return EnumInterval::empty();
        };

        let hcat = b.mul_category();
        let (side, hbound) = b.into_raw();

        match (fcat, hcat) {
            (MCat::Pos(az), MCat::Pos(bz)) => {
                // [a=0?, b>0] x [c=0? d=inf]
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // SAFETY: zeros handled ^^^
                    EnumInterval::half_bounded(Left, unsafe {
                        non_zero_mul_unchecked(fmin, hbound)
                    })
                }
            }
            (MCat::Pos(_), MCat::NegPos) => {
                // [a=0?, b>0] x [c<0, d>0]
                // Case 1: [a=0?, b>0] x [c<0, d=+inf] => |ac<=0, ad>=0, bc<0, bd=+inf| => (bc, ->)
                // Case 2: [a=0?, b>0] x [c=-inf, d>0] => |ac<=0, ad>=0, bc=-inf, bd>0| -> (<-, bd)
                // SAFETY: a always produces an intermediate value
                EnumInterval::half_bounded(side, unsafe { non_zero_mul_unchecked(fmax, hbound) })
            }
            (MCat::Pos(az), MCat::Neg(bz)) | (MCat::Neg(az), MCat::Pos(bz)) => {
                // Case 1: [a=0?, b>0] x [c=-inf, d=0?]
                // Case 2: [a<0, b=0?] x [c=0?, d=+inf]
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::unbound_closed(<T as Mul>::Output::zero())
                } else {
                    // SAFETY: zeros handled ^^^
                    EnumInterval::half_bounded(Right, unsafe {
                        non_zero_mul_unchecked(fmax, hbound)
                    })
                }
            }
            (MCat::Neg(_), MCat::NegPos) => {
                // [a<0, b=0?] x [c<0, d>0] => b produces intermediate values
                // Case 1: [a<0, b=0?] x [c<0, d=+inf] => |ac>0, ad=-inf, bc>=0, bd<=0| => (<-, ac>0)
                // Case 2: [a<0, b=0?] x [c=-inf, d>0] => |ac=+inf, ad<0, bc>=0, bd<=0| => (ad<0, ->)
                EnumInterval::half_bounded(side.flip(), unsafe {
                    non_zero_mul_unchecked(fmin, hbound)
                })
            }
            (MCat::Neg(az), MCat::Neg(bz)) => {
                // [a<0, b<=0?] x [c=-inf, d<=0?] => |ac=+inf, ad>=0, bc>=0, bd>=0
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // SAFETY: checked zeros ^^^
                    EnumInterval::half_bounded(Left, unsafe {
                        non_zero_mul_unchecked(fmax, hbound)
                    })
                }
            }
            (MCat::NegPos, _) => EnumInterval::unbounded(),
            (MCat::Zero, _) => EnumInterval::singleton(<T as Mul>::Output::zero()),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn enum_x_finite<T>(
        a: EnumInterval<T>,
        b: FiniteInterval<T>,
    ) -> EnumInterval<<T as Mul>::Output>
    where
        T: Mul + PartialOrd + Clone + Zero,
        <T as Mul>::Output: Element + Zero + Clone,
    {
        match a {
            EnumInterval::Finite(inner) => (inner * b).into(),
            EnumInterval::Half(inner) => inner * b,
            EnumInterval::Unbounded => match b.mul_category() {
                MCat::Empty => EnumInterval::empty(),
                MCat::Zero => EnumInterval::singleton(<T as Mul>::Output::zero()),
                _ => EnumInterval::Unbounded,
            },
        }
    }

    #[inline]
    pub fn enum_x_half<T>(
        a: EnumInterval<T>,
        b: HalfInterval<T>,
    ) -> EnumInterval<<T as Mul>::Output>
    where
        T: Mul + Element + Clone + Zero,
        <T as Mul>::Output: Element + Zero + Clone,
    {
        match a {
            EnumInterval::Finite(inner) => inner * b,
            EnumInterval::Half(inner) => half_x_half_by_cat(inner, b),
            EnumInterval::Unbounded => EnumInterval::Unbounded,
        }
    }

    #[derive(Debug, PartialEq)]
    enum MaybeZero {
        Zero,
        NonZero,
    }

    #[derive(Debug, PartialEq)]
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

            let t_zero = T::zero();
            let zero = FiniteOrdBound::closed(&t_zero);
            match lhs.finite_ord(Left).partial_cmp(&zero).unwrap() {
                Greater => MCat::Pos(MaybeZero::NonZero),
                Equal => match rhs.finite_ord(Right).partial_cmp(&zero).unwrap() {
                    Greater => MCat::Pos(MaybeZero::Zero),
                    Equal => MCat::Zero,
                    Less => unreachable!(),
                },
                Less => match rhs.finite_ord(Right).partial_cmp(&zero).unwrap() {
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

        let xno = HalfInterval::unbound_open(-10.0);
        let xpo = HalfInterval::open_unbound(10.0);
        let expected = EnumInterval::open_unbound(100.0);
        assert_eq!(xno * xno, expected);
        assert_eq!(xpo * xpo, expected);

        let xnc = HalfInterval::unbound_closed(0.0);
        let xpc = HalfInterval::closed_unbound(0.0);
        assert_eq!(xnc * xnc, xpc.into());
        assert_eq!(xpc * xpc, xpc.into());
        assert_eq!(xnc * xpc, xnc.into());
        assert_eq!(xpc * xnc, xnc.into());

        assert_eq!(xno * xnc, xpc.into());
        assert_eq!(xnc * xno, xpc.into());
        assert_eq!(xpo * xpc, xpc.into());
        assert_eq!(xpc * xpo, xpc.into());

        assert_eq!(xpc * xno, xnc.into());
        assert_eq!(xno * xpc, xnc.into());
        assert_eq!(xpo * xnc, xnc.into());
        assert_eq!(xnc * xpo, xnc.into());

        assert_eq!(
            HalfInterval::unbound_closed(-5.0) * HalfInterval::closed_unbound(10.0),
            EnumInterval::unbound_closed(-50.0)
        );

        let a = HalfInterval::unbound_open(0.0);
        let b = HalfInterval::open_unbound(0.0);
        let expected = EnumInterval::unbound_open(0.0);
        assert_eq!(a * b, expected);
        assert_eq!(b * a, expected);

        let a = HalfInterval::unbound_open(0.0);
        let b = HalfInterval::closed_unbound(0.0);
        let expected = EnumInterval::unbound_closed(0.0);
        assert_eq!(a * b, expected);
        assert_eq!(b * a, expected);

        let a = HalfInterval::unbound_closed(0.0);
        let b = HalfInterval::open_unbound(0.0);
        let expected = EnumInterval::unbound_closed(0.0);
        assert_eq!(a * b, expected);
        assert_eq!(b * a, expected);
    }

    #[test]
    fn test_enum_x_finite() {
        assert_eq!(
            EnumInterval::unbounded() * FiniteInterval::singleton(0.0),
            EnumInterval::singleton(0.0)
        );

        assert_eq!(
            EnumInterval::closed(0, 5) * EnumInterval::closed(0, 5),
            EnumInterval::closed(0, 25)
        );

        assert_eq!(
            EnumInterval::open(-10.0, -5.0) * EnumInterval::open(-10.0, -5.0),
            EnumInterval::open(25.0, 100.0)
        );
    }
}

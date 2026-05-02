use core::ops::Mul;

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
    use crate::category::{ECat, MaybeZero};
    use crate::{EnumInterval, FiniteInterval};

    /// Multiply two non-zero bounds together.
    ///
    /// # Preconditions
    ///
    /// A Closed(Zero) bound must not be passed. Closed(0) * Open(5)
    /// returns Open(5) which is wrong. Violating this yields incorrect
    /// results but no undefined behavior.
    #[inline(always)]
    fn mul_assume_nonzero<T: Mul>(a: FB<T>, b: FB<T>) -> FB<<T as Mul>::Output> {
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
        let acat = a.category();
        let bcat = b.category();

        let Some((amin, amax)) = a.into_raw() else {
            return FiniteInterval::empty();
        };

        let Some((bmin, bmax)) = b.into_raw() else {
            return FiniteInterval::empty();
        };

        match (acat, bcat) {
            (ECat::Pos(az), ECat::Pos(bz)) => {
                // [a=0?, b>0] x [c=0?, d>0]
                let max = mul_assume_nonzero(amax, bmax);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(FiniteBound::zero(), max)
                } else {
                    let min = mul_assume_nonzero(amin, bmin);
                    FiniteInterval::new(min, max)
                }
            }
            (ECat::Pos(_), ECat::NegPos) => {
                // [a=0?, b>0] x [c<0, d>0] => a produces intermediate values
                let min = mul_assume_nonzero(amax.clone(), bmin);
                let max = mul_assume_nonzero(amax, bmax);
                FiniteInterval::new(min, max)
            }
            (ECat::Pos(az), ECat::Neg(bz)) => {
                // [a=0?, b>0] x [c<0, d=0?]
                let min = mul_assume_nonzero(amax, bmin);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(min, FiniteBound::zero())
                } else {
                    let max = mul_assume_nonzero(amin, bmax);
                    FiniteInterval::new(min, max)
                }
            }
            (ECat::Neg(az), ECat::Pos(bz)) => {
                // [a<0, b=0?] x [c=0?, d>0]
                let min = mul_assume_nonzero(amin, bmax);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(min, FiniteBound::zero())
                } else {
                    let max = mul_assume_nonzero(amax, bmin);
                    FiniteInterval::new(min, max)
                }
            }
            (ECat::Neg(_), ECat::NegPos) => {
                // [a<0, b=0?] x [c<0, d>0] => b produces intermediate values
                let min = mul_assume_nonzero(amin.clone(), bmax);
                let max = mul_assume_nonzero(amin, bmin);
                FiniteInterval::new(min, max)
            }
            (ECat::Neg(az), ECat::Neg(bz)) => {
                // [a<0, b=0?] x [c<0, d=0?]
                let max = mul_assume_nonzero(amin, bmin);
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::new(FiniteBound::zero(), max)
                } else {
                    let min = mul_assume_nonzero(amax, bmax);
                    FiniteInterval::new(min, max)
                }
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // [a<0, b>0] x [c=0?, d>0] => c produces intermediate values
                let min = mul_assume_nonzero(amin, bmax.clone());
                let max = mul_assume_nonzero(amax, bmax);
                FiniteInterval::new(min, max)
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // [a<0, b>0] x [c<0, d=0?] => d produces intermediate values
                let min = mul_assume_nonzero(amax, bmin.clone());
                let max = mul_assume_nonzero(amin, bmin);
                FiniteInterval::new(min, max)
            }
            (ECat::NegPos, ECat::NegPos) => {
                // NegPos category can not have an end bound of Closed(0), so
                // every product below avoids the Closed(0) precondition.
                let c1_min = mul_assume_nonzero(amin.clone(), bmax.clone());
                let c2_min = mul_assume_nonzero(amax.clone(), bmin.clone());
                let c1_max = mul_assume_nonzero(amin, bmin);
                let c2_max = mul_assume_nonzero(amax, bmax);
                let min = FiniteBound::take_min_assume_valid(Left, c1_min, c2_min);
                let max = FiniteBound::take_max_assume_valid(Right, c1_max, c2_max);
                FiniteInterval::new(min, max)
            }
            (ECat::Zero, _) | (_, ECat::Zero) => {
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
        let acat = a.category();
        let bcat = b.category();

        let (_, abound) = a.into_raw();
        let (_, bbound) = b.into_raw();

        match (acat, bcat) {
            (ECat::NegPos, _) | (_, ECat::NegPos) => EnumInterval::unbounded(),
            (ECat::Pos(az), ECat::Pos(bz)) | (ECat::Neg(az), ECat::Neg(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // (a > 0 && b > 0) || (a < 0 && b < 0) -> neither is Closed(0)
                    let min = mul_assume_nonzero(abound, bbound);
                    EnumInterval::half_bounded(Left, min)
                }
            }
            (ECat::Pos(az), ECat::Neg(bz)) | (ECat::Neg(az), ECat::Pos(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::unbound_closed(<T as Mul>::Output::zero())
                } else {
                    // (a > 0 && b < 0) || (a < 0 && b > 0) -> neither is Closed(0)
                    let max = mul_assume_nonzero(abound, bbound);
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
        let fcat = a.category();
        let Some((fmin, fmax)) = a.into_raw() else {
            return EnumInterval::empty();
        };

        let hcat = b.category();
        let (side, hbound) = b.into_raw();

        match (fcat, hcat) {
            (ECat::Pos(az), ECat::Pos(bz)) => {
                // [a=0?, b>0] x [c=0? d=inf]
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::half_bounded(Left, mul_assume_nonzero(fmin, hbound))
                }
            }
            (ECat::Pos(_), ECat::NegPos) => {
                // [a=0?, b>0] x [c<0, d>0]
                // Case 1: [a=0?, b>0] x [c<0, d=+inf] => |ac<=0, ad>=0, bc<0, bd=+inf| => (bc, ->)
                // Case 2: [a=0?, b>0] x [c=-inf, d>0] => |ac<=0, ad>=0, bc=-inf, bd>0| -> (<-, bd)
                // b > 0 always produces an intermediate value
                EnumInterval::half_bounded(side, mul_assume_nonzero(fmax, hbound))
            }
            (ECat::Pos(az), ECat::Neg(bz)) | (ECat::Neg(az), ECat::Pos(bz)) => {
                // Case 1: [a=0?, b>0] x [c=-inf, d=0?]
                // Case 2: [a<0, b=0?] x [c=0?, d=+inf]
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::unbound_closed(<T as Mul>::Output::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::half_bounded(Right, mul_assume_nonzero(fmax, hbound))
                }
            }
            (ECat::Neg(_), ECat::NegPos) => {
                // [a<0, b=0?] x [c<0, d>0] => b produces intermediate values
                // Case 1: [a<0, b=0?] x [c<0, d=+inf] => |ac>0, ad=-inf, bc>=0, bd<=0| => (<-, ac>0)
                // Case 2: [a<0, b=0?] x [c=-inf, d>0] => |ac=+inf, ad<0, bc>=0, bd<=0| => (ad<0, ->)
                // a < 0 always produces an intermediate value
                EnumInterval::half_bounded(side.flip(), mul_assume_nonzero(fmin, hbound))
            }
            (ECat::Neg(az), ECat::Neg(bz)) => {
                // [a<0, b<=0?] x [c=-inf, d<=0?] => |ac=+inf, ad>=0, bc>=0, bd>=0
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::closed_unbound(<T as Mul>::Output::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::half_bounded(Left, mul_assume_nonzero(fmax, hbound))
                }
            }
            (ECat::NegPos, _) => EnumInterval::unbounded(),
            (ECat::Zero, _) => EnumInterval::singleton(<T as Mul>::Output::zero()),
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
            EnumInterval::Unbounded => match b.category() {
                ECat::Empty => EnumInterval::empty(),
                ECat::Zero => EnumInterval::singleton(<T as Mul>::Output::zero()),
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

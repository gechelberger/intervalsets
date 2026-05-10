use core::ops::Mul;

use super::TryMul;
use crate::bound::FiniteBound;
use crate::bound::Side::{Left, Right};
use crate::error::Error;
use crate::factory::traits::*;
use crate::numeric::{Element, Zero};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

// Set-level `TryMul` binds on `T: TryMul<Output = T>` and propagates
// `T::TryMul::Error` (e.g. `MathError`) into `crate::error::Error`. The
// infix `*` is panicking sugar over `try_mul().unwrap()` (Tier 3b).

macro_rules! mul_via_try {
    ($lhs:ty, $rhs:ty, $out:ty) => {
        impl<T> Mul<$rhs> for $lhs
        where
            $lhs: TryMul<$rhs, Output = $out>,
            <$lhs as TryMul<$rhs>>::Error: core::fmt::Debug,
        {
            type Output = $out;
            #[inline]
            fn mul(self, rhs: $rhs) -> Self::Output {
                self.try_mul(rhs).unwrap()
            }
        }
    };
}

mul_via_try!(FiniteInterval<T>, FiniteInterval<T>, FiniteInterval<T>);
mul_via_try!(HalfInterval<T>, HalfInterval<T>, EnumInterval<T>);
mul_via_try!(FiniteInterval<T>, HalfInterval<T>, EnumInterval<T>);
mul_via_try!(HalfInterval<T>, FiniteInterval<T>, EnumInterval<T>);
mul_via_try!(EnumInterval<T>, FiniteInterval<T>, EnumInterval<T>);
mul_via_try!(EnumInterval<T>, HalfInterval<T>, EnumInterval<T>);
mul_via_try!(EnumInterval<T>, EnumInterval<T>, EnumInterval<T>);
mul_via_try!(FiniteInterval<T>, EnumInterval<T>, EnumInterval<T>);
mul_via_try!(HalfInterval<T>, EnumInterval<T>, EnumInterval<T>);

impl<T> TryMul for FiniteInterval<T>
where
    T: Element + Clone + PartialOrd + Zero + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = FiniteInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        impls::finite_x_finite_by_cat(self, rhs)
    }
}

impl<T> TryMul for HalfInterval<T>
where
    T: Element + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        impls::half_x_half_by_cat(self, rhs)
    }
}

impl<T> TryMul<HalfInterval<T>> for FiniteInterval<T>
where
    T: Element + PartialOrd + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::finite_x_half(self, rhs)
    }
}

impl<T> TryMul<FiniteInterval<T>> for HalfInterval<T>
where
    T: Element + PartialOrd + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::finite_x_half(rhs, self)
    }
}

impl<T> TryMul<FiniteInterval<T>> for EnumInterval<T>
where
    T: Element + PartialOrd + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::enum_x_finite(self, rhs)
    }
}

impl<T> TryMul<HalfInterval<T>> for EnumInterval<T>
where
    T: Element + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::enum_x_half(self, rhs)
    }
}

impl<T> TryMul<EnumInterval<T>> for EnumInterval<T>
where
    T: Element + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(lhs) => lhs.try_mul(rhs),
            Self::Half(lhs) => lhs.try_mul(rhs),
            Self::Unbounded => match rhs {
                Self::Finite(rhs) => self.try_mul(rhs),
                Self::Half(rhs) => self.try_mul(rhs),
                Self::Unbounded => Ok(EnumInterval::Unbounded),
            },
        }
    }
}

impl<T> TryMul<EnumInterval<T>> for FiniteInterval<T>
where
    T: Element + PartialOrd + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::enum_x_finite(rhs, self)
    }
}

impl<T> TryMul<EnumInterval<T>> for HalfInterval<T>
where
    T: Element + Zero + Clone + TryMul<Output = T>,
    <T as TryMul>::Error: Into<Error>,
{
    type Output = EnumInterval<T>;
    type Error = Error;

    fn try_mul(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::enum_x_half(rhs, self)
    }
}

mod impls {
    use super::*;
    use crate::bound::FiniteBound as FB;
    use crate::category::{ECat, MaybeZero};
    use crate::{EnumInterval, FiniteInterval};

    /// Multiply two non-zero bounds together, propagating overflow as Err.
    ///
    /// # Preconditions
    ///
    /// A Closed(Zero) bound must not be passed. Closed(0) * Open(5)
    /// returns Open(5) which is wrong. Violating this yields incorrect
    /// results but no undefined behavior.
    #[inline(always)]
    fn mul_assume_nonzero<T>(a: FB<T>, b: FB<T>) -> Result<FB<T>, Error>
    where
        T: Element + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        let (akind, aval) = a.into_raw();
        let (bkind, bval) = b.into_raw();
        let val = aval.try_mul(bval).map_err(Into::into)?;
        FiniteBound::try_new(akind.combine(bkind), val)
    }

    pub(super) fn finite_x_finite_by_cat<T>(
        a: FiniteInterval<T>,
        b: FiniteInterval<T>,
    ) -> Result<FiniteInterval<T>, Error>
    where
        T: Element + PartialOrd + Clone + Zero + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        let acat = a.category();
        let bcat = b.category();

        let Some((amin, amax)) = a.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        let Some((bmin, bmax)) = b.into_raw() else {
            return Ok(FiniteInterval::empty());
        };

        match (acat, bcat) {
            (ECat::Pos(az), ECat::Pos(bz)) => {
                // [a=0?, b>0] x [c=0?, d>0]
                let max = mul_assume_nonzero(amax, bmax)?;
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::try_new(FiniteBound::closed(T::zero()), max)
                } else {
                    let min = mul_assume_nonzero(amin, bmin)?;
                    FiniteInterval::try_new(min, max)
                }
            }
            (ECat::Pos(_), ECat::NegPos) => {
                // [a=0?, b>0] x [c<0, d>0] => a produces intermediate values
                let min = mul_assume_nonzero(amax.clone(), bmin)?;
                let max = mul_assume_nonzero(amax, bmax)?;
                FiniteInterval::try_new(min, max)
            }
            (ECat::Pos(az), ECat::Neg(bz)) => {
                // [a=0?, b>0] x [c<0, d=0?]
                let min = mul_assume_nonzero(amax, bmin)?;
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::try_new(min, FiniteBound::closed(T::zero()))
                } else {
                    let max = mul_assume_nonzero(amin, bmax)?;
                    FiniteInterval::try_new(min, max)
                }
            }
            (ECat::Neg(az), ECat::Pos(bz)) => {
                // [a<0, b=0?] x [c=0?, d>0]
                let min = mul_assume_nonzero(amin, bmax)?;
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::try_new(min, FiniteBound::closed(T::zero()))
                } else {
                    let max = mul_assume_nonzero(amax, bmin)?;
                    FiniteInterval::try_new(min, max)
                }
            }
            (ECat::Neg(_), ECat::NegPos) => {
                // [a<0, b=0?] x [c<0, d>0] => b produces intermediate values
                let min = mul_assume_nonzero(amin.clone(), bmax)?;
                let max = mul_assume_nonzero(amin, bmin)?;
                FiniteInterval::try_new(min, max)
            }
            (ECat::Neg(az), ECat::Neg(bz)) => {
                // [a<0, b=0?] x [c<0, d=0?]
                let max = mul_assume_nonzero(amin, bmin)?;
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    FiniteInterval::try_new(FiniteBound::closed(T::zero()), max)
                } else {
                    let min = mul_assume_nonzero(amax, bmax)?;
                    FiniteInterval::try_new(min, max)
                }
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // [a<0, b>0] x [c=0?, d>0] => c produces intermediate values
                let min = mul_assume_nonzero(amin, bmax.clone())?;
                let max = mul_assume_nonzero(amax, bmax)?;
                FiniteInterval::try_new(min, max)
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // [a<0, b>0] x [c<0, d=0?] => d produces intermediate values
                let min = mul_assume_nonzero(amax, bmin.clone())?;
                let max = mul_assume_nonzero(amin, bmin)?;
                FiniteInterval::try_new(min, max)
            }
            (ECat::NegPos, ECat::NegPos) => {
                // NegPos category can not have an end bound of Closed(0), so
                // every product below avoids the Closed(0) precondition.
                let c1_min = mul_assume_nonzero(amin.clone(), bmax.clone())?;
                let c2_min = mul_assume_nonzero(amax.clone(), bmin.clone())?;
                let c1_max = mul_assume_nonzero(amin, bmin)?;
                let c2_max = mul_assume_nonzero(amax, bmax)?;
                let min = FiniteBound::take_min_assume_valid(Left, c1_min, c2_min);
                let max = FiniteBound::take_max_assume_valid(Right, c1_max, c2_max);
                FiniteInterval::try_new(min, max)
            }
            (ECat::Zero, _) | (_, ECat::Zero) => FiniteInterval::try_singleton(T::zero()),
            _ => unreachable!(),
        }
    }

    pub(super) fn half_x_half_by_cat<T>(
        a: HalfInterval<T>,
        b: HalfInterval<T>,
    ) -> Result<EnumInterval<T>, Error>
    where
        T: Element + PartialOrd + Zero + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        let acat = a.category();
        let bcat = b.category();

        let (_, abound) = a.into_raw();
        let (_, bbound) = b.into_raw();

        match (acat, bcat) {
            (ECat::NegPos, _) | (_, ECat::NegPos) => Ok(EnumInterval::unbounded()),
            (ECat::Pos(az), ECat::Pos(bz)) | (ECat::Neg(az), ECat::Neg(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::try_closed_unbound(T::zero())
                } else {
                    // (a > 0 && b > 0) || (a < 0 && b < 0) -> neither is Closed(0)
                    let min = mul_assume_nonzero(abound, bbound)?;
                    EnumInterval::try_half_bounded(Left, min)
                }
            }
            (ECat::Pos(az), ECat::Neg(bz)) | (ECat::Neg(az), ECat::Pos(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::try_unbound_closed(T::zero())
                } else {
                    // (a > 0 && b < 0) || (a < 0 && b > 0) -> neither is Closed(0)
                    let max = mul_assume_nonzero(abound, bbound)?;
                    EnumInterval::try_half_bounded(Right, max)
                }
            }
            _ => unreachable!(), // zero, empty should be unreachable
        }
    }

    pub(super) fn finite_x_half<T>(
        a: FiniteInterval<T>,
        b: HalfInterval<T>,
    ) -> Result<EnumInterval<T>, Error>
    where
        T: Element + PartialOrd + Zero + Clone + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        let fcat = a.category();
        let Some((fmin, fmax)) = a.into_raw() else {
            return Ok(EnumInterval::empty());
        };

        let hcat = b.category();
        let (side, hbound) = b.into_raw();

        match (fcat, hcat) {
            (ECat::Pos(az), ECat::Pos(bz)) => {
                // [a=0?, b>0] x [c=0? d=inf]
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::try_closed_unbound(T::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::try_half_bounded(Left, mul_assume_nonzero(fmin, hbound)?)
                }
            }
            (ECat::Pos(_), ECat::NegPos) => {
                // [a=0?, b>0] x [c<0, d>0]
                // b > 0 always produces an intermediate value
                EnumInterval::try_half_bounded(side, mul_assume_nonzero(fmax, hbound)?)
            }
            (ECat::Pos(az), ECat::Neg(bz)) | (ECat::Neg(az), ECat::Pos(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::try_unbound_closed(T::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::try_half_bounded(Right, mul_assume_nonzero(fmax, hbound)?)
                }
            }
            (ECat::Neg(_), ECat::NegPos) => {
                // [a<0, b=0?] x [c<0, d>0] => a produces intermediate values
                EnumInterval::try_half_bounded(side.flip(), mul_assume_nonzero(fmin, hbound)?)
            }
            (ECat::Neg(az), ECat::Neg(bz)) => {
                if az == MaybeZero::Zero || bz == MaybeZero::Zero {
                    EnumInterval::try_closed_unbound(T::zero())
                } else {
                    // zeros handled above -> neither operand is Closed(0)
                    EnumInterval::try_half_bounded(Left, mul_assume_nonzero(fmax, hbound)?)
                }
            }
            (ECat::NegPos, _) => Ok(EnumInterval::unbounded()),
            (ECat::Zero, _) => EnumInterval::try_singleton(T::zero()),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub(super) fn enum_x_finite<T>(
        a: EnumInterval<T>,
        b: FiniteInterval<T>,
    ) -> Result<EnumInterval<T>, Error>
    where
        T: Element + PartialOrd + Clone + Zero + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        match a {
            EnumInterval::Finite(inner) => finite_x_finite_by_cat(inner, b).map(EnumInterval::from),
            EnumInterval::Half(inner) => finite_x_half(b, inner),
            EnumInterval::Unbounded => match b.category() {
                ECat::Empty => Ok(EnumInterval::empty()),
                ECat::Zero => EnumInterval::try_singleton(T::zero()),
                _ => Ok(EnumInterval::Unbounded),
            },
        }
    }

    #[inline]
    pub(super) fn enum_x_half<T>(
        a: EnumInterval<T>,
        b: HalfInterval<T>,
    ) -> Result<EnumInterval<T>, Error>
    where
        T: Element + Clone + Zero + TryMul<Output = T>,
        <T as TryMul>::Error: Into<Error>,
    {
        match a {
            EnumInterval::Finite(inner) => finite_x_half(inner, b),
            EnumInterval::Half(inner) => half_x_half_by_cat(inner, b),
            EnumInterval::Unbounded => Ok(EnumInterval::Unbounded),
        }
    }
}

// === Value-level primitive impls (E2) ===

use super::macros::{impl_try_mul_checked, impl_try_mul_float_finite};

impl_try_mul_checked!(i8);
impl_try_mul_checked!(i16);
impl_try_mul_checked!(i32);
impl_try_mul_checked!(i64);
impl_try_mul_checked!(i128);
impl_try_mul_checked!(isize);
impl_try_mul_checked!(u8);
impl_try_mul_checked!(u16);
impl_try_mul_checked!(u32);
impl_try_mul_checked!(u64);
impl_try_mul_checked!(u128);
impl_try_mul_checked!(usize);

impl_try_mul_float_finite!(f32);
impl_try_mul_float_finite!(f64);

/// `Option<T>` delegates to the inner `T` impl. See [`TryAdd`](super::TryAdd)'s
/// `Option` impl for the convention.
impl<T: TryMul> TryMul for Option<T> {
    type Output = Option<<T as TryMul>::Output>;
    type Error = <T as TryMul>::Error;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        match (self, rhs) {
            (Some(a), Some(b)) => a.try_mul(b).map(Some),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MathError;

    #[test]
    fn test_finite_x_finite() {
        let x = FiniteInterval::closed(0.0, 10.0);
        assert_eq!(x.try_mul(x).unwrap(), FiniteInterval::closed(0.0, 100.0));

        let x = FiniteInterval::closed(5.0, 10.0);
        assert_eq!(x.try_mul(x).unwrap(), FiniteInterval::closed(25.0, 100.0));

        let y = FiniteInterval::closed(-10.0, -5.0);
        assert_eq!(x.try_mul(y).unwrap(), FiniteInterval::closed(-100.0, -25.0));
        assert_eq!(y.try_mul(y).unwrap(), FiniteInterval::closed(25.0, 100.0));

        let a = FiniteInterval::open(-10.0, 0.0);
        let b = FiniteInterval::open(0.0, 10.0);
        assert_eq!(a.try_mul(b).unwrap(), FiniteInterval::open(-100.0, 0.0));

        let a = FiniteInterval::closed(-10.0, 10.0);
        assert_eq!(a.try_mul(a).unwrap(), FiniteInterval::closed(-100.0, 100.0));
    }

    #[test]
    fn test_half_x_half() {
        let u: EnumInterval<f64> = EnumInterval::unbounded();

        let x = HalfInterval::closed_unbound(-1.0);
        assert_eq!(x.try_mul(x).unwrap(), u);

        let x = HalfInterval::unbound_closed(1.0);
        assert_eq!(x.try_mul(x).unwrap(), u);

        let xno = HalfInterval::unbound_open(-10.0);
        let xpo = HalfInterval::open_unbound(10.0);
        let expected = EnumInterval::open_unbound(100.0);
        assert_eq!(xno.try_mul(xno).unwrap(), expected);
        assert_eq!(xpo.try_mul(xpo).unwrap(), expected);

        let xnc = HalfInterval::unbound_closed(0.0);
        let xpc = HalfInterval::closed_unbound(0.0);
        assert_eq!(xnc.try_mul(xnc).unwrap(), xpc.into());
        assert_eq!(xpc.try_mul(xpc).unwrap(), xpc.into());
        assert_eq!(xnc.try_mul(xpc).unwrap(), xnc.into());
        assert_eq!(xpc.try_mul(xnc).unwrap(), xnc.into());

        assert_eq!(xno.try_mul(xnc).unwrap(), xpc.into());
        assert_eq!(xnc.try_mul(xno).unwrap(), xpc.into());
        assert_eq!(xpo.try_mul(xpc).unwrap(), xpc.into());
        assert_eq!(xpc.try_mul(xpo).unwrap(), xpc.into());

        assert_eq!(xpc.try_mul(xno).unwrap(), xnc.into());
        assert_eq!(xno.try_mul(xpc).unwrap(), xnc.into());
        assert_eq!(xpo.try_mul(xnc).unwrap(), xnc.into());
        assert_eq!(xnc.try_mul(xpo).unwrap(), xnc.into());

        assert_eq!(
            HalfInterval::unbound_closed(-5.0)
                .try_mul(HalfInterval::closed_unbound(10.0))
                .unwrap(),
            EnumInterval::unbound_closed(-50.0)
        );

        let a = HalfInterval::unbound_open(0.0);
        let b = HalfInterval::open_unbound(0.0);
        let expected = EnumInterval::unbound_open(0.0);
        assert_eq!(a.try_mul(b).unwrap(), expected);
        assert_eq!(b.try_mul(a).unwrap(), expected);

        let a = HalfInterval::unbound_open(0.0);
        let b = HalfInterval::closed_unbound(0.0);
        let expected = EnumInterval::unbound_closed(0.0);
        assert_eq!(a.try_mul(b).unwrap(), expected);
        assert_eq!(b.try_mul(a).unwrap(), expected);

        let a = HalfInterval::unbound_closed(0.0);
        let b = HalfInterval::open_unbound(0.0);
        let expected = EnumInterval::unbound_closed(0.0);
        assert_eq!(a.try_mul(b).unwrap(), expected);
        assert_eq!(b.try_mul(a).unwrap(), expected);
    }

    /// Closed(0) interacting with Open bounds is the longstanding bug
    /// fixed in the FiniteBound::Mul impl. Verify the fix holds at
    /// every layer:
    /// - direct FiniteBound::Mul (the path that was buggy)
    /// - interval-level via the categorical analysis (which never
    ///   passes Closed(0) to mul_assume_nonzero, so the bug couldn't
    ///   reach this path even before the fix - but verify anyway).
    #[test]
    fn test_closed_zero_propagation() {
        // direct FiniteBound: was Closed(0) * Open(5) -> Open(0) (wrong)
        // now: Closed(0) * Open(5) -> Closed(0)
        let cl_0: crate::bound::FiniteBound<i32> = crate::bound::FiniteBound::closed(0);
        let op_5: crate::bound::FiniteBound<i32> = crate::bound::FiniteBound::open(5);
        assert_eq!(cl_0 * op_5, crate::bound::FiniteBound::closed(0));

        // interval-level: singleton {0} * any non-empty positive set = {0}
        assert_eq!(
            FiniteInterval::singleton(0)
                .try_mul(FiniteInterval::open(0, 5))
                .unwrap(),
            FiniteInterval::singleton(0)
        );
    }

    #[cfg(feature = "ordered-float")]
    #[test]
    fn test_closed_zero_propagation_ordered_float() {
        // interval-level: [0, 1] * (0.0, 5.0) for OrderedFloat -> [0.0, 5.0)
        // closed lower bound at 0 is preserved (0 is reachable via x=0)
        // open upper bound at 5 is preserved (5 is not reachable, since rhs upper is open)
        use ordered_float::OrderedFloat as O;
        assert_eq!(
            FiniteInterval::closed(O(0.0_f64), O(1.0))
                .try_mul(FiniteInterval::open(O(0.0_f64), O(5.0)))
                .unwrap(),
            FiniteInterval::closed_open(O(0.0), O(5.0))
        );
    }

    #[test]
    fn test_enum_x_finite() {
        assert_eq!(
            EnumInterval::<f64>::unbounded()
                .try_mul(FiniteInterval::singleton(0.0))
                .unwrap(),
            EnumInterval::singleton(0.0)
        );

        // i32 is Ord; the infix `*` operator is available.
        assert_eq!(
            EnumInterval::closed(0, 5) * EnumInterval::closed(0, 5),
            EnumInterval::closed(0, 25)
        );

        assert_eq!(
            EnumInterval::open(-10.0, -5.0)
                .try_mul(EnumInterval::open(-10.0, -5.0))
                .unwrap(),
            EnumInterval::open(25.0, 100.0)
        );
    }

    /// OrderedFloat<f64> satisfies the infix Mul operator bounds.
    #[cfg(feature = "ordered-float")]
    #[test]
    fn test_ord_float_mul() {
        use ordered_float::OrderedFloat as O;

        // strict-positive bounds
        let x = FiniteInterval::closed(O(0.0), O(10.0));
        assert_eq!(x * x, FiniteInterval::closed(O(0.0), O(100.0)));

        let y = FiniteInterval::closed(O(5.0), O(10.0));
        assert_eq!(y * y, FiniteInterval::closed(O(25.0), O(100.0)));

        // negative × negative
        let z = FiniteInterval::closed(O(-10.0), O(-5.0));
        assert_eq!(y * z, FiniteInterval::closed(O(-100.0), O(-25.0)));
        assert_eq!(z * z, FiniteInterval::closed(O(25.0), O(100.0)));

        // open zero-crossing
        let a = FiniteInterval::open(O(-10.0), O(0.0));
        let b = FiniteInterval::open(O(0.0), O(10.0));
        assert_eq!(a * b, FiniteInterval::open(O(-100.0), O(0.0)));

        // half × half
        let h = HalfInterval::closed_unbound(O(-1.0));
        let u: EnumInterval<O<f64>> = EnumInterval::unbounded();
        assert_eq!(h * h, u);
    }

    // -- value-level primitive smoke tests (E2) --

    #[test]
    fn primitive_signed_mul() {
        assert_eq!(<i32 as TryMul>::try_mul(6, 7), Ok(42));
        assert_eq!(<i32 as TryMul>::try_mul(i32::MAX, 2), Err(MathError::Range));
    }

    #[test]
    fn primitive_unsigned_mul() {
        assert_eq!(<u32 as TryMul>::try_mul(6, 7), Ok(42));
        assert_eq!(<u32 as TryMul>::try_mul(u32::MAX, 2), Err(MathError::Range));
    }

    #[test]
    fn primitive_float_mul() {
        assert_eq!(<f64 as TryMul>::try_mul(6.0, 7.0), Ok(42.0));
        assert_eq!(
            <f64 as TryMul>::try_mul(f64::MAX, 2.0),
            Err(MathError::Domain)
        );
    }

    #[test]
    fn option_mul_matrix() {
        assert_eq!(Some(6_i32).try_mul(Some(7)), Ok(Some(42)));
        assert_eq!(Some(6_i32).try_mul(None), Ok(None));
        assert_eq!(None::<i32>.try_mul(Some(7)), Ok(None));
        assert_eq!(None::<i32>.try_mul(None), Ok(None));
    }

    // -- E6: set-level overflow surfaces as Err on `try_mul`, panics on `*` --

    #[test]
    fn set_level_int_overflow_returns_err() {
        let a = FiniteInterval::<i32>::closed(i32::MAX, i32::MAX);
        let b = FiniteInterval::<i32>::closed(2, 2);
        let r = a.try_mul(b);
        assert!(matches!(r, Err(Error::Math(MathError::Range))));
    }

    #[test]
    #[should_panic]
    fn set_level_int_overflow_infix_panics() {
        let a = FiniteInterval::<i32>::closed(i32::MAX, i32::MAX);
        let b = FiniteInterval::<i32>::closed(2, 2);
        let _ = a * b;
    }
}

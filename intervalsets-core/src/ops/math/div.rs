#![allow(unused)]

use core::ops::Div;

use crate::category::ECat;
use crate::disjoint::MaybeDisjoint;
use crate::error::Error;
use crate::factory::traits::*;
use crate::numeric::{Element, Zero};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

/// Div that returns Result instead of panicking on logical violations.
///
/// See [`super::TryAdd`] for the convention.
pub trait TryDiv<Rhs = Self> {
    /// The type produced by a successful division.
    type Output;
    /// The error returned when the operation cannot produce a valid result.
    type Error;
    /// Divide `self` by `rhs`, returning `Err` instead of panicking.
    fn try_div(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

// The infix Div operators below all require T: Ord. For Ord types,
// partial_cmp on bounds is total, so try_div is provably infallible
// and the .unwrap() can never panic. Float users without an Ord
// wrapper (e.g. OrderedFloat) must use TryDiv::try_div directly.

macro_rules! div_via_try {
    ($lhs:ty, $rhs:ty) => {
        impl<T> Div<$rhs> for $lhs
        where
            T: Div<Output = T> + Element + Ord + Zero + Clone,
        {
            type Output = MaybeDisjoint<T>;
            #[inline(always)]
            fn div(self, rhs: $rhs) -> Self::Output {
                self.try_div(rhs).unwrap()
            }
        }
    };
}

div_via_try!(FiniteInterval<T>, FiniteInterval<T>);
div_via_try!(HalfInterval<T>, HalfInterval<T>);
div_via_try!(FiniteInterval<T>, HalfInterval<T>);
div_via_try!(HalfInterval<T>, FiniteInterval<T>);
div_via_try!(EnumInterval<T>, FiniteInterval<T>);
div_via_try!(EnumInterval<T>, HalfInterval<T>);
div_via_try!(EnumInterval<T>, EnumInterval<T>);
div_via_try!(FiniteInterval<T>, EnumInterval<T>);
div_via_try!(HalfInterval<T>, EnumInterval<T>);

impl<T> TryDiv for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        impls::finite_by_finite(self, rhs)
    }
}

impl<T> TryDiv for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        impls::half_by_half(self, rhs)
    }
}

impl<T> TryDiv<HalfInterval<T>> for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::finite_by_half(self, rhs)
    }
}

impl<T> TryDiv<FiniteInterval<T>> for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        impls::half_by_finite(self, rhs)
    }
}

impl<T> TryDiv<FiniteInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: FiniteInterval<T>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(lhs) => lhs.try_div(rhs),
            Self::Half(lhs) => lhs.try_div(rhs),
            Self::Unbounded => impls::unbounded_by_cat(rhs.category()),
        }
    }
}

impl<T> TryDiv<HalfInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(lhs) => lhs.try_div(rhs),
            Self::Half(lhs) => lhs.try_div(rhs),
            Self::Unbounded => Ok(Self::Unbounded.into()),
        }
    }
}

impl<T> TryDiv<EnumInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(lhs) => lhs.try_div(rhs),
            Self::Half(lhs) => lhs.try_div(rhs),
            Self::Unbounded => impls::unbounded_by_cat(rhs.category()),
        }
    }
}

impl<T> TryDiv<EnumInterval<T>> for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    #[inline(always)]
    fn try_div(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        match rhs {
            EnumInterval::Finite(rhs) => self.try_div(rhs),
            EnumInterval::Half(rhs) => self.try_div(rhs),
            EnumInterval::Unbounded => Ok(match self.category() {
                ECat::Empty => EnumInterval::empty(),
                ECat::Zero => EnumInterval::try_singleton(T::zero())?,
                _ => EnumInterval::Unbounded,
            }
            .into()),
        }
    }
}

impl<T> TryDiv<EnumInterval<T>> for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    fn try_div(self, rhs: EnumInterval<T>) -> Result<Self::Output, Self::Error> {
        match rhs {
            EnumInterval::Finite(rhs) => self.try_div(rhs),
            EnumInterval::Half(rhs) => self.try_div(rhs),
            EnumInterval::Unbounded => Ok(EnumInterval::Unbounded.into()),
        }
    }
}

mod impls {
    use EnumInterval as EI;

    use super::*;
    use crate::bound::Side::{self, Left, Right};
    use crate::bound::{FiniteBound as FB, SetBounds};
    use crate::category::{ECat, MaybeZero};
    use crate::factory::traits::*;

    /// Divide two bounds that have a non-branching finite output.
    ///
    /// # Preconditions
    ///
    /// The caller is responsible for making sure that:
    /// 1. the numerator is not Closed(0) unless the denom is also closed.
    ///    the numerator is allowed to be Open(0). ie. +/- epsilon.
    /// 2. the denominator is not allowed to be Open or Closed 0. ie. (-e, 0, +e)
    ///
    /// Violating these yields incorrect results but no undefined behavior.
    #[inline(always)]
    fn div_assume_nonzero<T>(numer: FB<T>, denom: FB<T>) -> FB<T>
    where
        T: Div<Output = T>,
    {
        let (nkind, nval) = numer.into_raw();
        let (dkind, dval) = denom.into_raw();
        FB::new(nkind.combine(dkind), nval / dval)
    }

    /// anything divided by the zero singleton set.
    /// [a, b] / [0, 0]
    #[inline(always)]
    fn any_by_zero<T>() -> MaybeDisjoint<T> {
        MaybeDisjoint::empty()
    }

    #[inline(always)]
    fn zero_by_non_zero<T: Zero + Element + Clone>() -> Result<MaybeDisjoint<T>, Error> {
        EI::try_singleton(T::zero()).map(MaybeDisjoint::from)
    }

    #[inline(always)]
    fn all_except_zero<T: Zero + Element>() -> Result<MaybeDisjoint<T>, Error> {
        let neg = EI::try_right_bounded(FB::open(T::zero()))?;
        let pos = EI::try_left_bounded(FB::open(T::zero()))?;
        Ok((neg, pos).into())
    }

    #[inline(always)]
    pub fn unbounded_by_cat<T>(denom_cat: ECat) -> Result<MaybeDisjoint<T>, Error> {
        Ok(match denom_cat {
            ECat::Empty => FiniteInterval::empty().into(),
            ECat::Zero => any_by_zero(),
            _ => EI::Unbounded.into(),
        })
    }

    pub fn finite_by_finite<T>(
        ab: FiniteInterval<T>,
        cd: FiniteInterval<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let Some((a, b)) = ab.into_raw() else {
            return Ok(MaybeDisjoint::empty());
        };

        let Some((c, d)) = cd.into_raw() else {
            return Ok(MaybeDisjoint::empty());
        };

        match (ab_cat, cd_cat) {
            (_, ECat::Zero) => Ok(any_by_zero()),
            (ECat::Zero, _) => zero_by_non_zero(),
            (ECat::Pos(lz), ECat::Pos(_)) => {
                // [a>=0, +e<b<+inf] / [c>=0, +e<d<+inf] => {a/d, b/c}
                // cd Pos(_) => d > +e (never 0 or epsilon)
                let min = div_assume_denom_nonzero(lz, a, d);
                // ab Pos => b > +e (never Closed(0))
                div_same_sign_max(min, b, c)
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // CASE 0: [a<0, b>0] / [0>=c>=+e, d>+e] = {-inf, +inf}
                // CASE 1: [a<0, b>0] / [c>+e, d>+e]     = {a/c, b/c}
                div_non_zero_bounds_by_bound(a, b, c)
            }
            (ECat::Neg(lz), ECat::Pos(_)) => {
                // cd Pos(_) => d > +e
                let max = div_assume_denom_nonzero(lz, b, d);
                // ab Neg => a < -e (never Closed(0))
                div_opp_sign_min(max, a, c)
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // c < -e && d > +e && a != Closed(0)
                let left = EI::try_right_bounded(div_assume_nonzero(a.clone(), c))?;
                let right = EI::try_left_bounded(div_assume_nonzero(a, d))?;
                Ok((left, right).into())
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // c < -e && d > +e && b != Closed(0)
                let left = EI::try_right_bounded(div_assume_nonzero(b.clone(), d))?;
                let right = EI::try_left_bounded(div_assume_nonzero(b, c))?;
                Ok((left, right).into())
            }
            (_, ECat::NegPos) => Ok(EI::unbounded().into()),
            (ECat::Pos(lz), ECat::Neg(_)) => {
                // cd Neg(_) => c < -e
                let max = div_assume_denom_nonzero(lz, a, c);
                // ab Pos(_) => b > +e (never Closed(0))
                div_opp_sign_min(max, b, d)
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // CASE 0: [a<0, b>0] / [c<-e, -e<=d<=0] = {-inf, inf}
                // CASE 1: [a<0, b>0] / [c<-e, d<-e]     = {b/d, a/d}
                div_non_zero_bounds_by_bound(b, a, d)
            }
            (ECat::Neg(lz), ECat::Neg(_)) => {
                // cd Neg(_) => c < -e
                let min = div_assume_denom_nonzero(lz, b, c);
                // ab Neg [a<0, b<=0] => a is never Closed(0)
                div_same_sign_max(min, a, d)
            }
            _ => unreachable!(),
        }
    }

    pub fn half_by_half<T>(
        ab: HalfInterval<T>,
        cd: HalfInterval<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let (ab_side, ab_bound) = ab.into_raw();
        let (cd_side, cd_bound) = cd.into_raw();

        match (ab_cat, cd_cat) {
            (ECat::Pos(nz), ECat::Pos(_)) | (ECat::Neg(nz), ECat::Neg(_)) => {
                // CASE 0: [a>=0, b>+e] / [c>=0, d>+e] = {0, +inf}
                // CASE 1: [a<-e, b<=0] / [c<-e, d<=0] = {0, +inf}
                EI::try_left_bounded(div_inf_bound(nz)).map(MaybeDisjoint::from)
            }
            (ECat::Neg(nz), ECat::Pos(_)) | (ECat::Pos(nz), ECat::Neg(_)) => {
                // CASE 0: [a<-e, b<=0] / [c>=0, d>+e] = {-inf, 0}
                // CASE 1: [a>=0, b>+e] / [c<-e, d<=0] = {-inf, 0}
                EI::try_right_bounded(div_inf_bound(nz)).map(MaybeDisjoint::from)
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // CASE 0: [a<0, b>0] / [0<=c<+e, d=inf] = {-inf, +inf}
                // CASE 1: [a<0, b=+inf] / [c>+e, d=inf] = {a/c, +inf},
                // CASE 2: [a=-inf, b>0] / [c>+e, d=inf] = {-inf, b/c}
                if cd_bound.value() == &T::zero() {
                    Ok(EnumInterval::unbounded().into())
                } else {
                    // numer != Closed(0) because NegPos; denom > +e checked above
                    EnumInterval::try_half_bounded(ab_side, div_assume_nonzero(ab_bound, cd_bound))
                        .map(MaybeDisjoint::from)
                }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // CASE 0: [a<0, b>0] / [c=-inf, -e<d<=0] => {-inf, inf}
                // CASE 1: [a<0, b=+inf] / [c=-inf, d<-e] => {-inf, a/d}
                // CASE 2: [a=-inf, b>0] / [c=-inf, d<-e] => {b/d, +inf}
                if cd_bound.value() == &T::zero() {
                    Ok(EnumInterval::unbounded().into())
                } else {
                    // numer != Closed(0) because NegPos; denom < -e checked above
                    EnumInterval::try_half_bounded(
                        ab_side.flip(),
                        div_assume_nonzero(ab_bound, cd_bound),
                    )
                    .map(MaybeDisjoint::from)
                }
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b=+inf] / [c<0, d>0] = {-inf, a/c} U {a/d, +inf}
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                let zero = FB::open(T::zero());
                let non_zero = div_assume_nonzero(ab_bound, cd_bound);

                let pair = match cd_side {
                    // ab / [c<0, d=+inf] = {-inf, a/c} U {0, +inf}
                    Left => (
                        EI::try_right_bounded(non_zero)?,
                        EI::try_left_bounded(zero)?,
                    ),
                    // ab / [c=-inf, d>0] = {-inf, 0} U {a/d, +inf}
                    Right => (
                        EI::try_right_bounded(zero)?,
                        EI::try_left_bounded(non_zero)?,
                    ),
                };
                Ok(pair.into())
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a=-inf, b<0] / [c<0, d>0] = {-inf, b/d} U {b/c, +inf}
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                let zero = FB::open(T::zero());
                let non_zero = div_assume_nonzero(ab_bound, cd_bound);

                let pair = match cd_side {
                    // ab / [c<0, d=+inf] = {-inf, 0} U {b/c, +inf}
                    Left => (
                        EI::try_right_bounded(zero)?,
                        EI::try_left_bounded(non_zero)?,
                    ),
                    // ab / [c=-inf, d>0] = {-inf, b/d} U {0, +inf}
                    Right => (
                        EI::try_right_bounded(non_zero)?,
                        EI::try_left_bounded(zero)?,
                    ),
                };

                Ok(pair.into())
            }
            (_, ECat::NegPos) => Ok(EI::unbounded().into()),

            // half intervals can not be empty or zero
            _ => unreachable!(),
        }
    }

    pub fn finite_by_half<T>(
        ab: FiniteInterval<T>,
        cd: HalfInterval<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let Some((a, b)) = ab.into_raw() else {
            return Ok(EnumInterval::empty().into());
        };

        let (cd_side, cd_bound) = cd.into_raw();

        match (ab_cat, cd_cat) {
            (ECat::Zero, _) => zero_by_non_zero(),
            (ECat::Pos(nz), ECat::Pos(_)) => {
                let min = div_inf_bound(nz);
                // ab Pos => [a>=0, b>0] => b is not Closed(0)
                div_same_sign_max(min, b, cd_bound)
            }
            (ECat::Neg(nz), ECat::Neg(_)) => {
                let min = div_inf_bound(nz);
                // ab Neg => [a<0, b<=0] => a is not Closed(0)
                div_same_sign_max(min, a, cd_bound)
            }
            (ECat::Pos(nz), ECat::Neg(_)) => {
                // [a>=0, b>0] / [c=-inf, d<=0] => {b/d, a/c} => {b/d, 0}
                let max = div_inf_bound(nz);
                // ab Pos => b is never Closed(0)
                div_opp_sign_min(max, b, cd_bound)
            }
            (ECat::Neg(nz), ECat::Pos(_)) => {
                // [a<0, b<=0] / [c>=0, d=+inf] => {a/c, b/d} => {a/c, 0}
                let max = div_inf_bound(nz);
                // ab Neg => a is never Closed(0)
                div_opp_sign_min(max, a, cd_bound)
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // CASE 0: [a<0, b>0] / [0>=c>=+e, d=+inf] = {-inf, +inf}
                // CASE 1: [a<0, b>0] / [c>+e, d=+inf]     = {a/c, b/c}
                div_non_zero_bounds_by_bound(a, b, cd_bound)
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // CASE 0: [a<0, b>0] / [c=-inf, -e<=d<=0] = {-inf, inf}
                // CASE 1: [a<0, b>0] / [c=-inf, d<-e] = {b/d, a/d}
                div_non_zero_bounds_by_bound(b, a, cd_bound)
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b>0] / [c<0, d>0] => (<-, a/c) U (a/d, ->)
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                let zero = FB::open(T::zero());
                let non_zero = div_assume_nonzero(a, cd_bound);

                let pair = match cd_side {
                    // ab / [c<0, d=+inf] = {-inf, a/c} U {0, +inf}
                    Left => (
                        EI::try_right_bounded(non_zero)?,
                        EI::try_left_bounded(zero)?,
                    ),
                    // ab / [c=-inf, d>0] = {-inf, 0} U {a/d, +inf}
                    Right => (
                        EI::try_right_bounded(zero)?,
                        EI::try_left_bounded(non_zero)?,
                    ),
                };

                Ok(pair.into())
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a<0, b<0] / [c<0, d>0] => (<-, b/d) U (b/c, ->)
                // c = -inf OR d = +inf
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                let zero = FB::open(T::zero());
                let non_zero = div_assume_nonzero(b, cd_bound);

                let pair = match cd_side {
                    // ab / [c<0, d=+inf] = {-inf, 0} U {b/c, +inf}
                    Left => (
                        EI::try_right_bounded(zero)?,
                        EI::try_left_bounded(non_zero)?,
                    ),
                    // ab / [c=-inf, d>0] = {-inf, b/d} U {0, +inf}
                    Right => (
                        EI::try_right_bounded(non_zero)?,
                        EI::try_left_bounded(zero)?,
                    ),
                };

                Ok(pair.into())
            }
            (_, ECat::NegPos) => Ok(EI::unbounded().into()),
            _ => unreachable!(),
        }
    }

    pub fn half_by_finite<T>(
        ab: HalfInterval<T>,
        cd: FiniteInterval<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let (ab_side, ab_bound) = ab.into_raw();
        let Some((c, d)) = cd.into_raw() else {
            return Ok(MaybeDisjoint::Consumed);
        };

        match (ab_cat, cd_cat) {
            (_, ECat::Zero) => Ok(any_by_zero()),
            (ECat::Pos(nz), ECat::Pos(_)) => {
                //[a>=0, b=inf] / [c>=0, +e<d<inf] => {a/d, inf}
                let a = ab_bound;
                // cd Pos(_) => d > +e
                let min = div_assume_denom_nonzero(nz, a, d);
                EI::try_left_bounded(min).map(MaybeDisjoint::from)
            }
            (ECat::Neg(nz), ECat::Neg(_)) => {
                // [a=-inf, b<=0] / [-inf<c<-e, d<=0] => {b/c, inf}
                let b = ab_bound;
                // cd Neg(_) => c < -e
                let min = div_assume_denom_nonzero(nz, b, c);
                EI::try_left_bounded(min).map(MaybeDisjoint::from)
            }
            (ECat::Pos(nz), ECat::Neg(_)) => {
                // [a>=0, b=inf] / [-inf<c<-e, d<=0] => {-inf, a/c}
                let a = ab_bound;
                // cd Neg(_) => c < -e
                let max = div_assume_denom_nonzero(nz, a, c);
                EI::try_right_bounded(max).map(MaybeDisjoint::from)
            }
            (ECat::Neg(nz), ECat::Pos(_)) => {
                // [a=-inf, b<=0] / [c>=0, +e<d<+inf] = {-inf, b/d}
                let b = ab_bound;
                // cd Pos(_) => d > +e
                let max = div_assume_denom_nonzero(nz, b, d);
                EI::try_right_bounded(max).map(MaybeDisjoint::from)
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b=inf] / [c<0, d>0] => {-inf, a/c} U {a/d, +inf}
                let a = ab_bound;
                let neg = EI::try_right_bounded(div_assume_nonzero(a.clone(), c))?;
                let pos = EI::try_left_bounded(div_assume_nonzero(a, d))?;
                Ok((neg, pos).into())
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a=-inf, b<0] / [c<0, d>0] => {-inf, b/d} U {b/c, +inf}
                let b = ab_bound;
                let neg = EI::try_right_bounded(div_assume_nonzero(b.clone(), d))?;
                let pos = EI::try_right_bounded(div_assume_nonzero(b, c))?;
                Ok((neg, pos).into())
            }
            (_, ECat::NegPos) => Ok(EI::unbounded().into()),
            (ECat::NegPos, ECat::Pos(_)) => {
                // CASE 1: [a<0, b=+inf] / [c>=0, d>e] = {a/c, +inf}
                // CASE 2: [a=-inf, b>0] / [c>=0, d>e] = {-inf, b/c}
                if c.value() == &T::zero() {
                    Ok(EI::unbounded().into())
                } else {
                    EI::try_half_bounded(ab_side, div_assume_nonzero(ab_bound, c))
                        .map(MaybeDisjoint::from)
                }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // CASE 1: [a<0, b=+inf] / [c<-e, d<=0] = {-inf, a/d}
                // CASE 2: [a=-inf, b>0] / [c<-e, d<=0] = {b/d, +inf}
                if d.value() == &T::zero() {
                    Ok(EI::unbounded().into())
                } else {
                    EI::try_half_bounded(ab_side.flip(), div_assume_nonzero(ab_bound, d))
                        .map(MaybeDisjoint::from)
                }
            }
            _ => unreachable!(),
        }
    }

    /// Return a new bound div by inf.
    ///
    /// The new bound is always zero, but open/closed depending
    /// on whether the original interval contained zero.
    #[inline(always)]
    fn div_inf_bound<T: Zero>(numer: MaybeZero) -> FB<T> {
        match numer {
            MaybeZero::Zero => FB::closed(T::zero()),
            MaybeZero::NonZero => FB::open(T::zero()),
        }
    }

    /// Create interval with max from num/denom handling denom -> 0.
    ///
    /// ignore MaybeZero for denominator: only detects Closed(0).
    /// +/- epsilon is LeftOpen(0) & RightOpen(0) and also needs to be checked.
    ///
    /// # Preconditions
    /// - Numerator is not checked and may not be Closed(0).
    /// - Denominator **is** checked for 0 and epsilon internally.
    ///
    /// Violating these yields incorrect results but no undefined behavior.
    #[inline(always)]
    fn div_same_sign_max<T>(
        min: FB<T>,
        numer: FB<T>,
        denom: FB<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Zero,
    {
        if denom.value() == &T::zero() {
            // denom = (0 or +e) | (-e or 0)
            EI::try_left_bounded(min).map(MaybeDisjoint::from)
        } else {
            let max = div_assume_nonzero(numer, denom);
            EI::try_finite(min, max).map(MaybeDisjoint::from)
        }
    }

    /// Create interval with min from numer/denom handling denom -> 0.
    ///
    /// ignore MaybeZero for denominator: only detects Closed(0).
    /// +/- epsilon is LeftOpen(0) & RightOpen(0) and also needs to be checked.
    ///
    ///  # Preconditions
    /// - Numerator is not checked and may not be Closed(0).
    /// - Denominator **is** checked for 0 and epsilon internally.
    ///
    /// Violating these yields incorrect results but no undefined behavior.
    #[inline(always)]
    fn div_opp_sign_min<T>(
        max: FB<T>,
        numer: FB<T>,
        denom: FB<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Zero,
    {
        if denom.value() == &T::zero() {
            // denom = (0 or +e) | (0 or -e)
            EI::try_right_bounded(max).map(MaybeDisjoint::from)
        } else {
            let min = div_assume_nonzero(numer, denom);
            EI::try_finite(min, max).map(MaybeDisjoint::from)
        }
    }

    /// Divide two bounds, branch to handle numer = closed(0). Denom unchecked.
    ///
    ///  # Preconditions
    /// - Numerator is checked for closed(0)
    /// - Caller must ensure denominator is not 0 or epsilon
    ///
    /// Violating these yields incorrect results but no undefined behavior.
    #[inline(always)]
    fn div_assume_denom_nonzero<T>(nz: MaybeZero, numer: FB<T>, denom: FB<T>) -> FB<T>
    where
        T: Div<Output = T> + Element + Zero,
    {
        match nz {
            MaybeZero::Zero => FB::closed(T::zero()),
            MaybeZero::NonZero => div_assume_nonzero(numer, denom),
        }
    }

    /// Divide non-zero numer bounds by denom. Denom is checked for 0/epsilon.
    ///
    /// # Preconditions
    /// - Caller is responsible for ordering num bounds according to denom sign.
    /// - Caller ensures that numerators are not closed(0).
    ///
    /// Violating these yields incorrect results but no undefined behavior.
    #[inline(always)]
    fn div_non_zero_bounds_by_bound<T>(
        num_to_min: FB<T>,
        num_to_max: FB<T>,
        denom: FB<T>,
    ) -> Result<MaybeDisjoint<T>, Error>
    where
        T: Div<Output = T> + Element + Zero + Clone,
    {
        if denom.value() == &T::zero() {
            Ok(EI::unbounded().into())
        } else {
            EI::try_finite(
                div_assume_nonzero(num_to_min, denom.clone()),
                div_assume_nonzero(num_to_max, denom),
            )
            .map(MaybeDisjoint::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    // f64 is not Ord, so the infix `/` operator is not available on
    // float intervals. Tests use try_div(rhs).unwrap() to exercise
    // the same arithmetic semantics through the panic-free entry point.
    fn d<L, R>(lhs: L, rhs: R) -> MaybeDisjoint<f64>
    where
        L: TryDiv<R, Output = MaybeDisjoint<f64>, Error = Error>,
    {
        lhs.try_div(rhs).unwrap()
    }

    #[test]
    fn test_finite_by_finite_non_neg() {
        let fc = FiniteInterval::closed;
        let fo = FiniteInterval::open;
        let fco = FiniteInterval::closed_open;
        //let foc = FiniteInterval::open_closed;

        let ecu = EnumInterval::closed_unbound;
        let eou = EnumInterval::open_unbound;
        //let euc = EnumInterval::unbound_closed;
        let euo = EnumInterval::unbound_open;

        // open/closed non-zero, strict pos / strict pos
        assert_eq!(d(fc(10.0, 100.0), fc(1.0, 2.0)), fc(5.0, 100.0).into());
        assert_eq!(d(fo(10.0, 100.0), fc(1.0, 2.0)), fo(5.0, 100.0).into());
        assert_eq!(d(fc(10.0, 100.0), fo(1.0, 2.0)), fo(5.0, 100.0).into());
        assert_eq!(d(fo(10.0, 100.0), fo(1.0, 2.0)), fo(5.0, 100.0).into());

        // closed/open pos numer, strict-pos denom
        assert_eq!(d(fc(0.0, 10.0), fc(1.0, 5.0)), fc(0.0, 10.0).into());
        assert_eq!(d(fo(0.0, 10.0), fc(1.0, 5.0)), fo(0.0, 10.0).into());

        assert_eq!(d(fc(0.0, 10.0), fo(1.0, 5.0)), fco(0.0, 10.0).into());
        assert_eq!(d(fo(0.0, 10.0), fo(1.0, 5.0)), fo(0.0, 10.0).into());

        // strict pos numer, zero pos denom
        assert_eq!(d(fc(0.5, 10.0), fc(0.0, 2.0)), ecu(0.5 / 2.0).into());

        // closed-zero pos numer, closed-zero pos denom
        assert_eq!(d(fc(0.0, 10.0), fc(0.0, 5.0)), ecu(0.0).into());

        // (+e, 1.0) / [-1.0, 1.0] => (<-, 0.0) U (0.0, ->)
        assert_eq!(d(fo(0.0, 1.0), fc(-1.0, 1.0)), (euo(0.0), eou(0.0)).into());
    }

    #[test]
    fn test_finite_by_finite_closed() {
        let fc = FiniteInterval::closed;
        let uc = EnumInterval::unbound_closed;
        let cu = EnumInterval::closed_unbound;

        assert_eq!(d(fc(-50.0, 5.0), fc(10.0, 20.0)), fc(-5.0, 0.5).into());
        assert_eq!(
            d(fc(-10.0, -5.0), fc(-20.0, -15.0)),
            fc(0.25, 2.0 / 3.0).into()
        );

        assert_eq!(d(fc(-10.0, -5.0), fc(2.0, 3.0)), fc(-5.0, -5.0 / 3.0).into());
        assert_eq!(d(fc(5.0, 10.0), fc(-3.0, -2.0)), fc(-5.0, -5.0 / 3.0).into());

        assert_eq!(d(fc(-10.0, 0.0), fc(1.0, 2.0)), fc(-10.0, 0.0).into());
        assert_eq!(d(fc(-10.0, 0.0), fc(0.0, 2.0)), uc(0.0).into());
        assert_eq!(d(fc(5.0, 10.0), fc(0.0, 2.0)), cu(2.5).into());
        assert_eq!(d(fc(0.0, 10.0), fc(0.0, 2.0)), cu(0.0).into());

        assert_eq!(
            d(fc(0.0, 5.0), fc(-1.0, 1.0)),
            EnumInterval::unbounded().into()
        );
        assert_eq!(d(fc(2.0, 5.0), fc(-1.0, 1.0)), (uc(-2.0), cu(2.0)).into());
    }

    #[test]
    fn test_half_by_half() {
        let cu = EnumInterval::closed_unbound;
        let ou = EnumInterval::open_unbound;
        let uc = EnumInterval::unbound_closed;
        //let uo = EnumInterval::unbound_open;
        //let u = EnumInterval::unbounded();

        assert_eq!(d(cu(10.0), cu(10.0)), ou(0.0).into());
        assert_eq!(d(cu(0.0), cu(10.0)), cu(0.0).into());
        assert_eq!(d(cu(-10.0), cu(10.0)), cu(-1.0).into());
        assert_eq!(d(cu(-100.0), cu(10.0)), cu(-10.0).into());

        assert_eq!(d(uc(0.0), cu(10.0)), uc(0.0).into());
        assert_eq!(d(cu(0.0), uc(-10.0)), uc(0.0).into());

        assert_eq!(d(uc(-10.0), uc(-10.0)), ou(0.0).into());
    }

    /// Verify that OrderedFloat<f64> satisfies the infix Div operator
    /// bounds: Div<Output = T> + Element + Ord + Zero + Clone. Confirms
    /// the user-facing claim that wrapping floats with OrderedFloat
    /// restores access to the infix arithmetic operators.
    #[cfg(feature = "ordered-float")]
    #[test]
    fn test_ord_float_div() {
        use ordered_float::OrderedFloat as O;

        let fc = |a, b| FiniteInterval::closed(O(a), O(b));
        let fo = |a, b| FiniteInterval::open(O(a), O(b));
        let cu = |a| EnumInterval::closed_unbound(O(a));
        let uc = |a| EnumInterval::unbound_closed(O(a));

        // strict pos / strict pos
        assert_eq!(fc(10.0, 100.0) / fc(1.0, 2.0), fc(5.0, 100.0).into());
        assert_eq!(fo(10.0, 100.0) / fo(1.0, 2.0), fo(5.0, 100.0).into());

        // closed-zero pos numer, closed-zero pos denom -> [0, +inf)
        assert_eq!(fc(0.0, 10.0) / fc(0.0, 5.0), cu(0.0).into());

        // mixed-sign denominator -> disjoint result
        assert_eq!(fc(2.0, 5.0) / fc(-1.0, 1.0), (uc(-2.0), cu(2.0)).into());

        // half / half
        let cu_pos = EnumInterval::closed_unbound(O(10.0));
        assert_eq!(cu_pos / cu_pos, EnumInterval::open_unbound(O(0.0)).into());
    }
}

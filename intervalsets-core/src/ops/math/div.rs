#![allow(unused)]

use core::ops::Div;

use crate::category::ECat;
use crate::disjoint::MaybeDisjoint;
use crate::factory::traits::*;
use crate::numeric::{Element, Zero};
use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl<T> Div for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: Self) -> Self::Output {
        impls::finite_by_finite(self, rhs)
    }
}

impl<T> Div for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: Self) -> Self::Output {
        impls::half_by_half(self, rhs)
    }
}

impl<T> Div<HalfInterval<T>> for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: HalfInterval<T>) -> Self::Output {
        impls::finite_by_half(self, rhs)
    }
}

impl<T> Div<FiniteInterval<T>> for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: FiniteInterval<T>) -> Self::Output {
        impls::half_by_finite(self, rhs)
    }
}

impl<T> Div<FiniteInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => impls::unbounded_by_cat(rhs.category()),
        }
    }
}

impl<T> Div<HalfInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: HalfInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => Self::Unbounded.into(),
        }
    }
}

impl<T> Div<EnumInterval<T>> for EnumInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs / rhs,
            Self::Half(lhs) => lhs / rhs,
            Self::Unbounded => impls::unbounded_by_cat(rhs.category()),
        }
    }
}

impl<T> Div<EnumInterval<T>> for FiniteInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match rhs {
            EnumInterval::Finite(rhs) => self / rhs,
            EnumInterval::Half(rhs) => self / rhs,
            EnumInterval::Unbounded => match self.category() {
                ECat::Empty => EnumInterval::empty(),
                ECat::Zero => EnumInterval::singleton(T::zero()),
                _ => EnumInterval::Unbounded,
            }
            .into(),
        }
    }
}

impl<T> Div<EnumInterval<T>> for HalfInterval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = MaybeDisjoint<T>;

    fn div(self, rhs: EnumInterval<T>) -> Self::Output {
        match rhs {
            EnumInterval::Finite(rhs) => self / rhs,
            EnumInterval::Half(rhs) => self / rhs,
            EnumInterval::Unbounded => EnumInterval::Unbounded.into(),
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
    /// # SAFETY
    ///
    /// The user is responsible for making sure that:
    /// 1. the numerator is not Closed(0) unless the denom is also closed.
    ///    the numerator is allowed to be Open(0). ie. +/- epsilon.
    /// 2. the denominator is not allowed to be Open or Closed 0. ie. (-e, 0, +e)
    unsafe fn non_zero_div_unchecked<T>(numer: FB<T>, denom: FB<T>) -> FB<T>
    where
        T: Div<Output = T>,
    {
        let (nkind, nval) = numer.into_raw();
        let (dkind, dval) = denom.into_raw();
        FB::new(nkind.combine(dkind), nval / dval)
    }

    /// anything divided by the zero singleton set.
    /// [a, b] / [0, 0]
    fn any_by_zero<T>() -> MaybeDisjoint<T> {
        MaybeDisjoint::empty()
    }

    fn zero_by_non_zero<T: Zero + Element + Clone>() -> MaybeDisjoint<T> {
        EI::singleton(T::zero()).into()
    }

    fn all_except_zero<T: Zero + Element>() -> MaybeDisjoint<T> {
        let neg = EI::right_bounded(FB::open(T::zero()));
        let pos = EI::left_bounded(FB::open(T::zero()));
        (neg, pos).into()
    }

    pub fn unbounded_by_cat<T>(denom_cat: ECat) -> MaybeDisjoint<T> {
        match denom_cat {
            ECat::Empty => FiniteInterval::empty().into(),
            ECat::Zero => any_by_zero(),
            _ => EI::Unbounded.into(),
        }
    }

    pub fn finite_by_finite<T>(ab: FiniteInterval<T>, cd: FiniteInterval<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let Some((a, b)) = ab.into_raw() else {
            return MaybeDisjoint::empty();
        };

        let Some((c, d)) = cd.into_raw() else {
            return MaybeDisjoint::empty();
        };

        match (ab_cat, cd_cat) {
            (_, ECat::Zero) => any_by_zero(),
            (ECat::Zero, _) => zero_by_non_zero(),
            (ECat::Pos(lz), ECat::Pos(_)) => {
                // [a>=0, +e<b<+inf] / [c>=0, +e<d<+inf] => {a/d, b/c}
                // SAFETY: cd Pos(_): d > +e (never 0 or epsilon)
                let min = unsafe { div_denom_unchecked(lz, a, d) };
                // SAFETY:
                // 1) ab Pos => b > +e (never closed(0))
                unsafe { div_same_sign_max(min, b, c) }
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                if c.value() == &T::zero() {
                    // c = 0 or c = +e
                    EI::unbounded().into()
                } else {
                    // SAFETY: c > +e && a < -e && b > +e
                    unsafe {
                        EI::finite(
                            non_zero_div_unchecked(a, c.clone()),
                            non_zero_div_unchecked(b, c),
                        )
                        .into()
                    }
                }
            }
            (ECat::Neg(lz), ECat::Pos(_)) => {
                // SAFETY: cd Pos(_) => d > +e
                let max = unsafe { div_denom_unchecked(lz, b, d) };
                // SAFETY: numer Neg: a < -e.
                unsafe { div_opp_sign_min(max, a, c) }
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // SAFETY: c < -e && d > +e && a != Closed(0)
                let left = EI::right_bounded(unsafe { non_zero_div_unchecked(a.clone(), c) });
                // SAFETY: c < -e && d > +e && a != Closed(0)
                let right = EI::left_bounded(unsafe { non_zero_div_unchecked(a, d) });
                (left, right).into()
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // SAFETY: c < -e, && d > +e && b != Closed(0)
                let left = EI::right_bounded(unsafe { non_zero_div_unchecked(b.clone(), d) });
                // SAFETY: c < -e, && d > +e && b != Closed(0)
                let right = EI::left_bounded(unsafe { non_zero_div_unchecked(b, c) });
                (left, right).into()
            }
            (_, ECat::NegPos) => EI::unbounded().into(),
            (ECat::Pos(lz), ECat::Neg(_)) => {
                // SAFETY: cd Neg(_) => c < -e
                let max = unsafe { div_denom_unchecked(lz, a, c) };
                // SAFETY: numer Pos(_) => b > +e => not Closed(0)
                unsafe { div_opp_sign_min(max, b, d) }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                if d.value() == &T::zero() {
                    // d = 0 or d = -e
                    EI::unbounded().into()
                } else {
                    // SAFETY: numer NegPos => a < 0, b > 0 && d checked ^^
                    unsafe {
                        EI::finite(
                            non_zero_div_unchecked(b, d.clone()),
                            non_zero_div_unchecked(a, d),
                        )
                        .into()
                    }
                }
            }
            (ECat::Neg(lz), ECat::Neg(_)) => {
                // SAFETY: `cd` is Neg(_) => `c` < -e
                let min = unsafe { div_denom_unchecked(lz, b, c) };
                // SAFETY: Neg [a<0, b<=0] => a is never Closed(0)
                unsafe { div_same_sign_max(min, a, d) }
            }
            _ => unreachable!(),
        }
    }

    pub fn half_by_half<T>(ab: HalfInterval<T>, cd: HalfInterval<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let (ab_side, ab_bound) = ab.into_raw();
        let (cd_side, cd_bound) = cd.into_raw();

        match (ab_cat, cd_cat) {
            (ECat::Pos(nz), ECat::Pos(_)) | (ECat::Neg(nz), ECat::Neg(_)) => {
                EI::left_bounded(div_inf_bound(nz)).into()
            }
            (ECat::Neg(nz), ECat::Pos(_)) | (ECat::Pos(nz), ECat::Neg(_)) => {
                EI::right_bounded(div_inf_bound(nz)).into()
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                if cd_bound.value() == &T::zero() {
                    EnumInterval::unbounded().into()
                } else {
                    // SAFETY: ab < 0 or ab > 0 && checked cd != 0 or +e
                    EnumInterval::half_bounded(ab_side, unsafe {
                        non_zero_div_unchecked(ab_bound, cd_bound)
                    })
                    .into()
                }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                if cd_bound.value() == &T::zero() {
                    EnumInterval::unbounded().into()
                } else {
                    EnumInterval::half_bounded(ab_side.flip(), unsafe {
                        non_zero_div_unchecked(ab_bound, cd_bound)
                    })
                    .into()
                }
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b=+inf] / [c<0, d>0] = {-inf, a/c} U {a/d, +inf}

                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                match cd_side {
                    Left => {
                        // c < 0, d = +inf = {-inf, a/c} U {0, +inf}
                        let left = EI::right_bounded(unsafe {
                            non_zero_div_unchecked(ab_bound, cd_bound)
                        });
                        let right = EI::open_unbound(T::zero());
                        (left, right).into()
                    }
                    Right => {
                        // c = -inf, d > 0 = {-inf, 0} U {a/d, +inf}
                        let left = EI::unbound_open(T::zero());
                        let right =
                            EI::left_bounded(unsafe { non_zero_div_unchecked(ab_bound, cd_bound) });
                        (left, right).into()
                    }
                }
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a=-inf, b<0] / [c<0, d>0] = {-inf, b/d} U {b/c, +inf}
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                match cd_side {
                    Left => {
                        // c < 0, d=+inf
                        let left = EI::unbound_open(T::zero());
                        let right =
                            EI::left_bounded(unsafe { non_zero_div_unchecked(ab_bound, cd_bound) });
                        (left, right).into()
                    }
                    Right => {
                        // c = -inf, d > 0
                        let left = EI::right_bounded(unsafe {
                            non_zero_div_unchecked(ab_bound, cd_bound)
                        });
                        let right = EI::open_unbound(T::zero());
                        (left, right).into()
                    }
                }
            }
            (_, ECat::NegPos) => EI::unbounded().into(),

            // half intervals can not be empty or zero
            _ => unreachable!(),
        }
    }

    pub fn finite_by_half<T>(ab: FiniteInterval<T>, cd: HalfInterval<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let Some((a, b)) = ab.into_raw() else {
            return EnumInterval::empty().into();
        };

        let (cd_side, cd_bound) = cd.into_raw();

        match (ab_cat, cd_cat) {
            (ECat::Zero, _) => zero_by_non_zero(),
            (ECat::Pos(nz), ECat::Pos(_)) => {
                let min = div_inf_bound(nz);
                // SAFETY: Pos => [a>=0, b>0] => b is not Closed(0)
                unsafe { div_same_sign_max(min, b, cd_bound) }
            }
            (ECat::Neg(nz), ECat::Neg(_)) => {
                let min = div_inf_bound(nz);
                // SAFETY: Neg => [a<0, b<=0] => a is not Closed(0)
                unsafe { div_same_sign_max(min, a, cd_bound) }
            }
            (ECat::Pos(nz), ECat::Neg(_)) => {
                // [a>=0, b>0] / [c=-inf, d<=0] => {b/d, a/c} => {b/d, 0}
                let max = div_inf_bound(nz);
                // SAFETY: Numer Pos => b is never Closed(0).
                unsafe { div_opp_sign_min(max, b, cd_bound) }
            }
            (ECat::Neg(nz), ECat::Pos(_)) => {
                // [a<0, b<=0] / [c>=0, d=+inf] => {a/c, b/d} => {a/c, 0}
                let max = div_inf_bound(nz);
                // SAFETY: Numer Neg => a is never Closed(0).
                unsafe { div_opp_sign_min(max, a, cd_bound) }
            }
            (ECat::NegPos, ECat::Pos(_)) => {
                // [a<0, b>0] / [c>=0, d=+inf] => {a/c, b/c}
                if cd_bound.value() == &T::zero() {
                    EI::unbounded().into()
                } else {
                    let (min, max) = unsafe {
                        (
                            non_zero_div_unchecked(a, cd_bound.clone()),
                            non_zero_div_unchecked(b, cd_bound),
                        )
                    };
                    EI::finite(min, max).into()
                }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // [a<0, b>0] / [c=-inf, d<=0] => {b/d, a/d}
                if cd_bound.value() == &T::zero() {
                    EI::unbounded().into()
                } else {
                    let (min, max) = unsafe {
                        (
                            non_zero_div_unchecked(b, cd_bound.clone()),
                            non_zero_div_unchecked(a, cd_bound),
                        )
                    };
                    EI::finite(min, max).into()
                }
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b>0] / [c<0, d>0] => (<-, a/c) U (a/d, ->)
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                match cd_side {
                    Left => {
                        // c < 0, d = +inf
                        //(<-, a/c) U (0, ->)
                        let left =
                            EI::right_bounded(unsafe { non_zero_div_unchecked(a, cd_bound) });
                        let right = EI::open_unbound(T::zero());
                        (left, right).into()
                    }
                    Right => {
                        // c = -inf, d > 0
                        //(<-, 0) U (a/d, ->)
                        let left = EI::unbound_open(T::zero());
                        let right =
                            EI::left_bounded(unsafe { non_zero_div_unchecked(a, cd_bound) });
                        (left, right).into()
                    }
                }
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a<0, b<0] / [c<0, d>0] => (<-, b/d) U (b/c, ->)
                // c = -inf OR d = +inf
                if cd_bound.value() == &T::zero() {
                    return all_except_zero();
                }

                match cd_side {
                    Left => {
                        // c < 0, d = +inf
                        // (<-, 0) U (b/c, ->)
                        let left = EI::unbound_open(T::zero());
                        let right =
                            EI::left_bounded(unsafe { non_zero_div_unchecked(b, cd_bound) });
                        (left, right).into()
                    }
                    Right => {
                        // c = -inf, d > 0
                        // (<-, b/d) U (0, ->)
                        let left =
                            EI::right_bounded(unsafe { non_zero_div_unchecked(b, cd_bound) });
                        let right = EI::open_unbound(T::zero());
                        (left, right).into()
                    }
                }
            }
            (_, ECat::NegPos) => EI::unbounded().into(),
            _ => unreachable!(),
        }
    }

    pub fn half_by_finite<T>(ab: HalfInterval<T>, cd: FiniteInterval<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Clone + Zero,
    {
        let ab_cat = ab.category();
        let cd_cat = cd.category();

        let (ab_side, ab_bound) = ab.into_raw();
        let Some((c, d)) = cd.into_raw() else {
            return MaybeDisjoint::Consumed;
        };

        match (ab_cat, cd_cat) {
            (_, ECat::Zero) => any_by_zero(),
            (ECat::Pos(nz), ECat::Pos(_)) => {
                //[a>=0, b=inf] / [c>=0, +e<d<inf] => {a/d, inf}
                let a = ab_bound;
                // SAFETY: cd Pos(_) => d > +e
                let min = unsafe { div_denom_unchecked(nz, a, d) };
                EI::left_bounded(min).into()
            }
            (ECat::Neg(nz), ECat::Neg(_)) => {
                // [a=-inf, b<=0] / [-inf<c<-e, d<=0] => {b/c, inf}
                let b = ab_bound;
                // SAFETY: cd Neg(_) => c < -e
                let min = unsafe { div_denom_unchecked(nz, b, c) };
                EI::left_bounded(min).into()
            }
            (ECat::Pos(nz), ECat::Neg(_)) => {
                // [a>=0, b=inf] / [-inf<c<-e, d<=0] => {-inf, a/c}
                let a = ab_bound;
                // SAFETY: cd Neg(_) => c < -e
                let max = unsafe { div_denom_unchecked(nz, a, c) };
                EI::right_bounded(max).into()
            }
            (ECat::Neg(nz), ECat::Pos(_)) => {
                // [a=-inf, b<=0] / [c>=0, +e<d<+inf] = {-inf, b/d}
                let b = ab_bound;
                // SAFETY: cd Pos(_) => d > +e
                let max = unsafe { div_denom_unchecked(nz, b, d) };
                EI::right_bounded(max).into()
            }
            (ECat::Pos(MaybeZero::NonZero), ECat::NegPos) => {
                // [a>0, b=inf] / [c<0, d>0] => {-inf, a/c} U {a/d, +inf}
                let a = ab_bound;
                let neg = EI::right_bounded(unsafe { non_zero_div_unchecked(a.clone(), c) });
                let pos = EI::left_bounded(unsafe { non_zero_div_unchecked(a, d) });
                (neg, pos).into()
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                // [a=-inf, b<0] / [c<0, d>0] => {-inf, b/d} U {b/c, +inf}
                let b = ab_bound;
                let neg = EI::right_bounded(unsafe { non_zero_div_unchecked(b.clone(), d) });
                let pos = EI::right_bounded(unsafe { non_zero_div_unchecked(b, c) });
                (neg, pos).into()
            }
            (_, ECat::NegPos) => EI::unbounded().into(),
            (ECat::NegPos, ECat::Pos(_)) => {
                // CASE 1: [a<0, b=+inf] / [c>=0, d>e] = {a/c, +inf}
                // CASE 2: [a=-inf, b>0] / [c>=0, d>e] = {-inf, b/c}
                if c.value() == &T::zero() {
                    EI::unbounded().into()
                } else {
                    EI::half_bounded(ab_side, unsafe { non_zero_div_unchecked(ab_bound, c) }).into()
                }
            }
            (ECat::NegPos, ECat::Neg(_)) => {
                // CASE 1: [a<0, b=+inf] / [c<-e, d<=0] = {-inf, a/d}
                // CASE 2: [a=-inf, b>0] / [c<-e, d<=0] = {b/d, +inf}
                if d.value() == &T::zero() {
                    EI::unbounded().into()
                } else {
                    EI::half_bounded(ab_side.flip(), unsafe {
                        non_zero_div_unchecked(ab_bound, d)
                    })
                    .into()
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
    /// +epsilon repr is LeftOpen(0) and also must be checked.
    ///
    /// # SAFETY
    ///
    /// Numerator is not checked and may not be Closed(0).
    /// Denominator **is** checked for 0 and epsilon internally.
    #[inline(always)]
    unsafe fn div_same_sign_max<T>(min: FB<T>, numer: FB<T>, denom: FB<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Zero,
    {
        if denom.value() == &T::zero() {
            // denom = (0 or +e) | (-e or 0)
            EI::left_bounded(min).into()
        } else {
            let max = non_zero_div_unchecked(numer, denom);
            EI::finite(min, max).into()
        }
    }

    /// # Safety
    ///
    /// Numerator is checked for closed(0), but the user is responsible for
    /// ensuring that the denominator is not 0 or epsilon
    #[inline(always)]
    unsafe fn div_denom_unchecked<T>(nz: MaybeZero, numer: FB<T>, denom: FB<T>) -> FB<T>
    where
        T: Div<Output = T> + Element + Zero,
    {
        match nz {
            MaybeZero::Zero => FB::closed(T::zero()),
            MaybeZero::NonZero => unsafe { non_zero_div_unchecked(numer, denom) },
        }
    }

    /// # Safety
    ///
    #[inline(always)]
    unsafe fn div_opp_sign_min<T>(max: FB<T>, numer: FB<T>, denom: FB<T>) -> MaybeDisjoint<T>
    where
        T: Div<Output = T> + Element + Zero,
    {
        if denom.value() == &T::zero() {
            // denom = (0 or +e) | (0 or -e)
            EI::right_bounded(max).into()
        } else {
            let min = non_zero_div_unchecked(numer, denom);
            EI::finite(min, max).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

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
        assert_eq!(fc(10.0, 100.0) / fc(1.0, 2.0), fc(5.0, 100.0).into());
        assert_eq!(fo(10.0, 100.0) / fc(1.0, 2.0), fo(5.0, 100.0).into());
        assert_eq!(fc(10.0, 100.0) / fo(1.0, 2.0), fo(5.0, 100.0).into());
        assert_eq!(fo(10.0, 100.0) / fo(1.0, 2.0), fo(5.0, 100.0).into());

        // closed/open pos numer, strict-pos denom
        assert_eq!(fc(0.0, 10.0) / fc(1.0, 5.0), fc(0.0, 10.0).into());
        assert_eq!(fo(0.0, 10.0) / fc(1.0, 5.0), fo(0.0, 10.0).into());

        assert_eq!(fc(0.0, 10.0) / fo(1.0, 5.0), fco(0.0, 10.0).into());
        assert_eq!(fo(0.0, 10.0) / fo(1.0, 5.0), fo(0.0, 10.0).into());

        // strict pos numer, zero pos denom
        assert_eq!(fc(0.5, 10.0) / fc(0.0, 2.0), ecu(0.5 / 2.0).into());

        // closed-zero pos numer, closed-zero pos denom
        assert_eq!(fc(0.0, 10.0) / fc(0.0, 5.0), ecu(0.0).into());

        // (+e, 1.0) / [-1.0, 1.0] => (<-, 0.0) U (0.0, ->)
        assert_eq!(fo(0.0, 1.0) / fc(-1.0, 1.0), (euo(0.0), eou(0.0)).into());
    }

    #[test]
    fn test_half_by_half() {
        let cu = EnumInterval::closed_unbound;
        let ou = EnumInterval::open_unbound;
        let uc = EnumInterval::unbound_closed;
        //let uo = EnumInterval::unbound_open;
        //let u = EnumInterval::unbounded();

        assert_eq!(cu(10.0) / cu(10.0), ou(0.0).into());
        assert_eq!(cu(0.0) / cu(10.0), cu(0.0).into());
        assert_eq!(cu(-10.0) / cu(10.0), cu(-1.0).into());
        assert_eq!(cu(-100.0) / cu(10.0), cu(-10.0).into());

        assert_eq!(uc(0.0) / cu(10.0), uc(0.0).into());
        assert_eq!(cu(0.0) / uc(-10.0), uc(0.0).into());

        assert_eq!(uc(-10.0) / uc(-10.0), ou(0.0).into());
    }
}

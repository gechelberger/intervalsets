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
    use crate::bound::FiniteBound as FB;
    use crate::bound::Side::{self, Left, Right};
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

    // anything divided by the zero singleton set.
    // [a, b] / [0, 0]
    fn any_by_zero<T>() -> MaybeDisjoint<T> {
        MaybeDisjoint::Consumed
    }

    fn zero_by_non_zero<T: Zero + Element + Clone>() -> MaybeDisjoint<T> {
        MaybeDisjoint::Connected(EI::singleton(T::zero()))
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
            return MaybeDisjoint::Consumed;
        };

        let Some((c, d)) = cd.into_raw() else {
            return MaybeDisjoint::Consumed;
        };

        match (ab_cat, cd_cat) {
            (_, ECat::Zero) => any_by_zero(),
            (ECat::Zero, _) => zero_by_non_zero(),
            (ECat::Pos(lz), ECat::Pos(_)) => {
                let min = match lz {
                    MaybeZero::Zero => FB::closed(T::zero()),
                    MaybeZero::NonZero => unsafe {
                        // SAFETY:
                        // a is numer so +e allowed and checked not closed(0) ^^
                        // cd is Pos(_) so if c=0  then 0 < d or ECat::Zero
                        // cd is Pos(_) so if c=+e then e < d bc (+e, +e) impos
                        // therefore d > +e
                        non_zero_div_unchecked(a, d)
                    },
                };

                // ignore MaybeZero for denominator: it only detects Closed(0).
                // +epsilon repr is LeftOpen(0) and also may not be in denom.
                if c.value() == &T::zero() {
                    // +e repr is LeftOpen(0.0)
                    EI::left_bounded(min).into()
                } else {
                    // SAFETY: ab is Pos(_) so b > 0; checked c=0/c=+e ^^^
                    EI::finite(min, unsafe { non_zero_div_unchecked(b, c) }).into()
                }
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
                let max = match lz {
                    MaybeZero::Zero => FB::closed(T::zero()),
                    MaybeZero::NonZero => unsafe {
                        // SAFETY: Pos(_) denom so d > +e, b < 0 checked ^^^
                        non_zero_div_unchecked(b, d)
                    },
                };

                if c.value() == &T::zero() {
                    // c = 0 or c = +e => (<-, max)
                    EI::right_bounded(max).into()
                } else {
                    // SAFETY: numer Neg(_) => a < -e
                    let min = unsafe { non_zero_div_unchecked(a, c) };
                    EI::finite(min, max).into()
                }
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
                let max = match lz {
                    MaybeZero::Zero => FB::closed(T::zero()),
                    MaybeZero::NonZero => unsafe {
                        // SAFETY: cd Neg(_) => c < -e && checked a ^^
                        non_zero_div_unchecked(a, c)
                    },
                };

                if d.value() == &T::zero() {
                    EI::half_bounded(Right, max).into()
                } else {
                    // ab Pos(_) => b > +e, checked d ^^
                    let min = unsafe { non_zero_div_unchecked(b, d) };
                    EI::finite(min, max).into()
                }
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
                let min = match lz {
                    MaybeZero::Zero => FB::closed(T::zero()),
                    MaybeZero::NonZero => unsafe {
                        // SAFETY: cd Neg(_) => c < -e, checked b ^^^
                        non_zero_div_unchecked(b, c)
                    },
                };

                if d.value() == &T::zero() {
                    EI::left_bounded(min).into()
                } else {
                    EI::finite(min, unsafe {
                        // SAFETY: ab Neg(_) => a < 0 && checked d ^^^
                        non_zero_div_unchecked(a, d)
                    })
                    .into()
                }
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
            (ECat::Pos(nz), ECat::Pos(_)) | (ECat::Neg(nz), ECat::Neg(_)) => match nz {
                MaybeZero::Zero => EI::closed_unbound(T::zero()).into(),
                MaybeZero::NonZero => EI::open_unbound(T::zero()).into(),
            },
            (ECat::Neg(nz), ECat::Pos(_)) | (ECat::Pos(nz), ECat::Neg(_)) => match nz {
                MaybeZero::Zero => EI::unbound_closed(T::zero()).into(),
                MaybeZero::NonZero => EI::unbound_open(T::zero()).into(),
            },
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
                todo!() // disjoint
            }
            (ECat::Neg(MaybeZero::NonZero), ECat::NegPos) => {
                todo!() // disjoint
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

        todo!()
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
            (ECat::Pos(_), ECat::Pos(_)) => todo!(),
            _ => unreachable!(),
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

pub use intervalsets_core::factory::{
    traits, Converter, ConvertingFactory, EIFactory, EmptyFactory, FiniteFactory,
    HalfBoundedFactory, Identity, UnboundedFactory,
};
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::bound::FiniteBound;
use crate::numeric::{Domain, Zero};
use crate::{Interval, IntervalSet, Side};

impl<T: Domain> ConvertingFactory<T, Identity> for Interval<T> {
    type Output = Self;
}

impl<T: Domain> EmptyFactory<T, Identity> for Interval<T> {
    fn empty() -> Self::Output {
        FiniteInterval::Empty.into()
    }
}

impl<T: Domain> FiniteFactory<T, Identity> for Interval<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(lhs, rhs).into()
    }

    fn strict_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Option<Self::Output> {
        FiniteInterval::new_strict(lhs, rhs).map(Interval::from)
    }
}

impl<T: Domain + Zero> HalfBoundedFactory<T, Identity> for Interval<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<T>) -> Option<Self::Output>
    where
        T: num_traits::Zero,
    {
        HalfInterval::new_strict(side, bound).map(Interval::from)
    }
}

impl<T: Domain> UnboundedFactory<T, Identity> for Interval<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

impl<T: Domain> ConvertingFactory<T, Identity> for IntervalSet<T> {
    type Output = Self;
}

impl<T: Domain> EmptyFactory<T, Identity> for IntervalSet<T> {
    fn empty() -> Self::Output {
        IntervalSet::empty()
    }
}

impl<T: Domain> FiniteFactory<T, Identity> for IntervalSet<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(lhs, rhs).into()
    }

    fn strict_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Option<Self::Output> {
        FiniteInterval::new_strict(lhs, rhs).map(IntervalSet::from)
    }
}

impl<T: Domain + Zero> HalfBoundedFactory<T, Identity> for IntervalSet<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<T>) -> Option<Self::Output>
    where
        T: num_traits::Zero,
    {
        HalfInterval::new_strict(side, bound).map(IntervalSet::from)
    }
}

impl<T: Domain> UnboundedFactory<T, Identity> for IntervalSet<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

pub struct IFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> ConvertingFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    type Output = Interval<C::To>;
}

impl<T, C> EmptyFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn empty() -> Self::Output {
        Interval::empty()
    }
}

impl<T, C> FiniteFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
        Interval::from(EIFactory::<T, C>::finite(lhs, rhs))
    }

    fn strict_finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Option<Self::Output> {
        EIFactory::<T, C>::strict_finite(lhs, rhs).map(Interval::from)
    }
}

impl<T, C> HalfBoundedFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain + Zero,
{
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        Interval::from(EIFactory::<T, C>::half_bounded(side, bound))
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<C::To>) -> Option<Self::Output> {
        EIFactory::<T, C>::strict_half_bounded(side, bound).map(Interval::from)
    }
}

impl<T, C> UnboundedFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn unbounded() -> Self::Output {
        Interval::unbounded()
    }
}

pub struct ISFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> ConvertingFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    type Output = IntervalSet<C::To>;
}

impl<T, C> EmptyFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn empty() -> Self::Output {
        IntervalSet::empty()
    }
}

impl<T, C> FiniteFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
        IFactory::<T, C>::finite(lhs, rhs).into()
    }

    fn strict_finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Option<Self::Output> {
        IFactory::<T, C>::strict_finite(lhs, rhs).map(IntervalSet::from)
    }
}

impl<T, C> HalfBoundedFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain + Zero,
{
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        IFactory::<T, C>::half_bounded(side, bound).into()
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<C::To>) -> Option<Self::Output> {
        IFactory::<T, C>::strict_half_bounded(side, bound).map(IntervalSet::from)
    }
}

impl<T, C> UnboundedFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    fn unbounded() -> Self::Output {
        IntervalSet::unbounded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_factory() {
        let a: Interval<_> = EIFactory::<u32, Identity>::closed(0, 10).into();
        let b = Interval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_interval_set_factory() {
        let x = IntervalSet::closed(0, 10);
        assert_eq!(x.expect_interval(), Interval::closed(0, 10));
    }
}

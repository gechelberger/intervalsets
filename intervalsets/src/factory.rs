pub use intervalsets_core::factory::{
    traits, Converter, ConvertingFactory, EIFactory, EmptyFactory, FiniteFactory,
    HalfBoundedFactory, Identity, UnboundedFactory,
};
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::bound::FiniteBound;
use crate::numeric::{Element, Zero};
use crate::{Interval, IntervalSet, Side};

impl<T: Element> ConvertingFactory<T, Identity> for Interval<T> {
    type Output = Self;
    type Error = crate::error::Error;
}

impl<T: Element> EmptyFactory<T, Identity> for Interval<T> {
    fn empty() -> Self::Output {
        FiniteInterval::empty().into()
    }
}

impl<T: Element> FiniteFactory<T, Identity> for Interval<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(lhs, rhs).into()
    }

    fn strict_finite(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::new_strict(lhs, rhs).map(Interval::from)
    }
}

impl<T: Element + Zero> HalfBoundedFactory<T, Identity> for Interval<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error>
    where
        T: num_traits::Zero,
    {
        HalfInterval::new_strict(side, bound).map(Interval::from)
    }
}

impl<T: Element> UnboundedFactory<T, Identity> for Interval<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

impl<T: Element> ConvertingFactory<T, Identity> for IntervalSet<T> {
    type Output = Self;
    type Error = crate::error::Error;
}

impl<T: Element> EmptyFactory<T, Identity> for IntervalSet<T> {
    fn empty() -> Self::Output {
        IntervalSet::empty()
    }
}

impl<T: Element> FiniteFactory<T, Identity> for IntervalSet<T> {
    fn finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(lhs, rhs).into()
    }

    fn strict_finite(
        lhs: FiniteBound<T>,
        rhs: FiniteBound<T>,
    ) -> Result<Self::Output, Self::Error> {
        FiniteInterval::new_strict(lhs, rhs).map(IntervalSet::from)
    }
}

impl<T: Element + Zero> HalfBoundedFactory<T, Identity> for IntervalSet<T> {
    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn strict_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error>
    where
        T: num_traits::Zero,
    {
        HalfInterval::new_strict(side, bound).map(IntervalSet::from)
    }
}

impl<T: Element> UnboundedFactory<T, Identity> for IntervalSet<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

pub struct IFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> ConvertingFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    type Output = Interval<C::To>;
    type Error = crate::error::Error;
}

impl<T, C> EmptyFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn empty() -> Self::Output {
        Interval::empty()
    }
}

impl<T, C> FiniteFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
        Interval::from(EIFactory::<T, C>::finite(lhs, rhs))
    }

    fn strict_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        EIFactory::<T, C>::strict_finite(lhs, rhs).map(Interval::from)
    }
}

impl<T, C> HalfBoundedFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element + Zero,
{
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        Interval::from(EIFactory::<T, C>::half_bounded(side, bound))
    }

    fn strict_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        EIFactory::<T, C>::strict_half_bounded(side, bound).map(Interval::from)
    }
}

impl<T, C> UnboundedFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn unbounded() -> Self::Output {
        Interval::unbounded()
    }
}

pub struct ISFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> ConvertingFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    type Output = IntervalSet<C::To>;
    type Error = crate::error::Error;
}

impl<T, C> EmptyFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn empty() -> Self::Output {
        IntervalSet::empty()
    }
}

impl<T, C> FiniteFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn finite(lhs: FiniteBound<C::To>, rhs: FiniteBound<C::To>) -> Self::Output {
        IFactory::<T, C>::finite(lhs, rhs).into()
    }

    fn strict_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        IFactory::<T, C>::strict_finite(lhs, rhs).map(IntervalSet::from)
    }
}

impl<T, C> HalfBoundedFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element + Zero,
{
    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        IFactory::<T, C>::half_bounded(side, bound).into()
    }

    fn strict_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        IFactory::<T, C>::strict_half_bounded(side, bound).map(IntervalSet::from)
    }
}

impl<T, C> UnboundedFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
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

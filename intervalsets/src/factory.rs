pub use intervalsets_core::factory::{
    traits, Converter, ConvertingFactory, EIFactory, EmptyFactory, FiniteFactory,
    HalfBoundedFactory, Identity, TryFiniteFactory, TryHalfBoundedFactory, UnboundedFactory,
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

impl<T: Element> TryFiniteFactory<T, Identity> for Interval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T: Element + Zero> TryHalfBoundedFactory<T, Identity> for Interval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound)
            .map_err(Into::into)
            .map(Interval::from)
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

impl<T: Element> TryFiniteFactory<T, Identity> for IntervalSet<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs)
            .map_err(Into::into)
            .map(IntervalSet::from)
    }
}

impl<T: Element + Zero> TryHalfBoundedFactory<T, Identity> for IntervalSet<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound)
            .map_err(Into::into)
            .map(IntervalSet::from)
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

impl<T, C> TryFiniteFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn try_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        EIFactory::<T, C>::try_finite(lhs, rhs)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T, C> TryHalfBoundedFactory<T, C> for IFactory<T, C>
where
    C: Converter<T>,
    C::To: Element + Zero,
{
    fn try_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        EIFactory::<T, C>::try_half_bounded(side, bound)
            .map_err(Into::into)
            .map(Interval::from)
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

impl<T, C> TryFiniteFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element,
{
    fn try_finite(
        lhs: FiniteBound<C::To>,
        rhs: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        IFactory::<T, C>::try_finite(lhs, rhs).map(IntervalSet::from)
    }
}

impl<T, C> TryHalfBoundedFactory<T, C> for ISFactory<T, C>
where
    C: Converter<T>,
    C::To: Element + Zero,
{
    fn try_half_bounded(
        side: Side,
        bound: FiniteBound<C::To>,
    ) -> Result<Self::Output, Self::Error> {
        IFactory::<T, C>::try_half_bounded(side, bound).map(IntervalSet::from)
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

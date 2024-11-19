pub use intervalsets_core::factory::{Converter, EIFactory, Factory, Identity};
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::bound::FiniteBound;
use crate::numeric::Domain;
use crate::{Interval, IntervalSet, Side};

impl<T: Domain> Factory<T, Identity> for Interval<T> {
    type Output = Interval<T>;

    fn empty() -> Self::Output {
        FiniteInterval::Empty.into()
    }

    fn finite(left: FiniteBound<T>, right: FiniteBound<T>) -> Self::Output {
        FiniteInterval::new(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        HalfInterval::new(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

pub struct SFactory<T, C>(std::marker::PhantomData<(T, C)>);

impl<T, C> Factory<T, C> for SFactory<T, C>
where
    C: Converter<T>,
    C::To: Domain,
{
    type Output = IntervalSet<C::To>;

    fn empty() -> Self::Output {
        Self::Output::empty()
    }

    fn finite(left: FiniteBound<C::To>, right: FiniteBound<C::To>) -> Self::Output {
        Interval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<C::To>) -> Self::Output {
        Interval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        Interval::unbounded().into()
    }
}

impl<T: Domain> Factory<T, Identity> for IntervalSet<T> {
    type Output = IntervalSet<T>;

    fn empty() -> Self::Output {
        Interval::empty().into()
    }

    fn finite(left: FiniteBound<T>, right: FiniteBound<T>) -> Self::Output {
        Interval::finite(left, right).into()
    }

    fn half_bounded(side: Side, bound: FiniteBound<T>) -> Self::Output {
        Interval::half_bounded(side, bound).into()
    }

    fn unbounded() -> Self::Output {
        Interval::unbounded().into()
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

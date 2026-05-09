pub use intervalsets_core::factory::{
    traits, EmptyFactory, Factory, FiniteFactory, HalfBoundedFactory, TryFiniteFactory,
    TryHalfBoundedFactory, UnboundedFactory,
};
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::bound::FiniteBound;
use crate::numeric::{Element, Zero};
use crate::{Interval, IntervalSet, Side};

impl<T: Element> Factory<T> for Interval<T> {
    type Output = Self;
    type Error = crate::error::Error;
}

impl<T: Element> EmptyFactory<T> for Interval<T> {
    fn empty() -> Self::Output {
        FiniteInterval::empty().into()
    }
}

impl<T: Element> TryFiniteFactory<T> for Interval<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T: Element + Zero> TryHalfBoundedFactory<T> for Interval<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T: Element> UnboundedFactory<T> for Interval<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

impl<T: Element> Factory<T> for IntervalSet<T> {
    type Output = Self;
    type Error = crate::error::Error;
}

impl<T: Element> EmptyFactory<T> for IntervalSet<T> {
    fn empty() -> Self::Output {
        IntervalSet::empty()
    }
}

impl<T: Element> TryFiniteFactory<T> for IntervalSet<T> {
    fn try_finite(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        FiniteInterval::try_new_or_empty(lhs, rhs)
            .map_err(Into::into)
            .map(IntervalSet::from)
    }
}

impl<T: Element + Zero> TryHalfBoundedFactory<T> for IntervalSet<T> {
    fn try_half_bounded(side: Side, bound: FiniteBound<T>) -> Result<Self::Output, Self::Error> {
        HalfInterval::try_new(side, bound)
            .map_err(Into::into)
            .map(IntervalSet::from)
    }
}

impl<T: Element> UnboundedFactory<T> for IntervalSet<T> {
    fn unbounded() -> Self::Output {
        EnumInterval::Unbounded.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_factory() {
        let a = Interval::<u32>::closed(0, 10);
        let b = Interval::<u32>::closed(0, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_interval_set_factory() {
        let x = IntervalSet::closed(0, 10);
        assert_eq!(x.expect_interval(), Interval::closed(0, 10));
    }
}

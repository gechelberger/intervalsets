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

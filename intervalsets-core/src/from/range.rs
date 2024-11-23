use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::bound::FiniteBound;
use crate::numeric::{Element, Zero};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Element> From<Range<T>> for FiniteInterval<T> {
    fn from(value: Range<T>) -> Self {
        FiniteInterval::new(
            FiniteBound::closed(value.start),
            FiniteBound::open(value.end),
        )
    }
}

impl<T: Element> From<RangeInclusive<T>> for FiniteInterval<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (start, end) = value.into_inner();
        FiniteInterval::new(FiniteBound::closed(start), FiniteBound::closed(end))
    }
}

impl<T: Element + Zero> From<RangeFrom<T>> for HalfInterval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        HalfInterval::left(FiniteBound::closed(value.start))
    }
}

impl<T: Element + Zero> From<RangeTo<T>> for HalfInterval<T> {
    fn from(value: RangeTo<T>) -> Self {
        HalfInterval::right(FiniteBound::open(value.end))
    }
}

impl<T: Element + Zero> From<RangeToInclusive<T>> for HalfInterval<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        HalfInterval::right(FiniteBound::closed(value.end))
    }
}

impl<T> From<RangeFull> for EnumInterval<T> {
    fn from(_: RangeFull) -> Self {
        EnumInterval::Unbounded
    }
}

impl<T: Element> From<Range<T>> for EnumInterval<T> {
    fn from(value: Range<T>) -> Self {
        Self::from(FiniteInterval::from(value))
    }
}

impl<T: Element> From<RangeInclusive<T>> for EnumInterval<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        Self::from(FiniteInterval::from(value))
    }
}

impl<T: Element + Zero> From<RangeFrom<T>> for EnumInterval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

impl<T: Element + Zero> From<RangeTo<T>> for EnumInterval<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

impl<T: Element + Zero> From<RangeToInclusive<T>> for EnumInterval<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};

    #[test]
    fn test_from_range() {
        fn eq<T: PartialEq + Debug>(a: EnumInterval<T>, b: EnumInterval<T>) {
            assert_eq!(a, b);
        }

        eq((0..10).into(), EnumInterval::closed_open(0, 10));
        eq((0.0..10.0).into(), EnumInterval::closed_open(0.0, 10.0));
        eq((0..=10).into(), EnumInterval::closed(0, 10));
        eq((0.0..=10.0).into(), EnumInterval::closed(0.0, 10.0));

        eq((0..).into(), EnumInterval::closed_unbound(0));
        eq((0.0..).into(), EnumInterval::closed_unbound(0.0));
        eq((..0).into(), EnumInterval::unbound_open(0));
        eq((..0.0).into(), EnumInterval::unbound_open(0.0));
        eq((..=0).into(), EnumInterval::unbound_closed(0));
        eq((..=0.0).into(), EnumInterval::unbound_closed(0.0));

        eq((..).into(), EnumInterval::<i32>::Unbounded);
    }
}

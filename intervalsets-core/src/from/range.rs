use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::bound::FiniteBound;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Domain> From<Range<T>> for FiniteInterval<T> {
    fn from(value: Range<T>) -> Self {
        FiniteInterval::new(
            FiniteBound::closed(value.start),
            FiniteBound::open(value.end),
        )
    }
}

impl<T: Domain> From<RangeInclusive<T>> for FiniteInterval<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        let (start, end) = value.into_inner();
        FiniteInterval::new(FiniteBound::closed(start), FiniteBound::closed(end))
    }
}

impl<T: Domain> From<RangeFrom<T>> for HalfInterval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        HalfInterval::left(FiniteBound::closed(value.start))
    }
}

impl<T: Domain> From<RangeTo<T>> for HalfInterval<T> {
    fn from(value: RangeTo<T>) -> Self {
        HalfInterval::right(FiniteBound::open(value.end))
    }
}

impl<T: Domain> From<RangeToInclusive<T>> for HalfInterval<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        HalfInterval::right(FiniteBound::closed(value.end))
    }
}

impl<T> From<RangeFull> for EnumInterval<T> {
    fn from(_: RangeFull) -> Self {
        EnumInterval::Unbounded
    }
}

impl<T: Domain> From<Range<T>> for EnumInterval<T> {
    fn from(value: Range<T>) -> Self {
        Self::from(FiniteInterval::from(value))
    }
}

impl<T: Domain> From<RangeInclusive<T>> for EnumInterval<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        Self::from(FiniteInterval::from(value))
    }
}

impl<T: Domain> From<RangeFrom<T>> for EnumInterval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

impl<T: Domain> From<RangeTo<T>> for EnumInterval<T> {
    fn from(value: RangeTo<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

impl<T: Domain> From<RangeToInclusive<T>> for EnumInterval<T> {
    fn from(value: RangeToInclusive<T>) -> Self {
        Self::from(HalfInterval::from(value))
    }
}

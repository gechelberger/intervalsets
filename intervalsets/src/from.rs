use intervalsets_core::bound::ord::OrdBoundPair;
use intervalsets_core::error::Error;
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::numeric::Element;
use crate::{Interval, IntervalSet, MaybeEmpty};

impl<T> From<EnumInterval<T>> for Interval<T> {
    fn from(value: EnumInterval<T>) -> Self {
        Self(value)
    }
}

macro_rules! interval_delegate_from_impl {
    ($t:ty) => {
        impl<T> From<$t> for Interval<T> {
            fn from(value: $t) -> Self {
                Self::from(EnumInterval::from(value))
            }
        }
    };
}

macro_rules! interval_delegate_w_domain_from_impl {
    ($t:ty) => {
        impl<T: $crate::numeric::Element> From<$t> for Interval<T> {
            fn from(value: $t) -> Self {
                Self::from(EnumInterval::from(value))
            }
        }
    };
}

macro_rules! interval_delegate_w_domain_zero_from_impl {
    ($t:ty) => {
        impl<T: $crate::numeric::Element + $crate::numeric::Zero> From<$t> for Interval<T> {
            fn from(value: $t) -> Self {
                Self::from(EnumInterval::from(value))
            }
        }
    };
}

interval_delegate_from_impl!(FiniteInterval<T>);
interval_delegate_from_impl!(HalfInterval<T>);
interval_delegate_w_domain_from_impl!((T, T));
interval_delegate_w_domain_from_impl!([T; 2]);
interval_delegate_w_domain_from_impl!(core::ops::Range<T>);
interval_delegate_w_domain_from_impl!(core::ops::RangeInclusive<T>);
interval_delegate_w_domain_zero_from_impl!(core::ops::RangeFrom<T>);
interval_delegate_w_domain_zero_from_impl!(core::ops::RangeTo<T>);
interval_delegate_w_domain_zero_from_impl!(core::ops::RangeToInclusive<T>);
interval_delegate_from_impl!(core::ops::RangeFull);

impl<T: Element> TryFrom<OrdBoundPair<T>> for Interval<T> {
    type Error = Error;

    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        let success = EnumInterval::<T>::try_from(value)?;
        Ok(Interval::from(success))
    }
}

/*impl<T: Clone> From<&Interval<T>> for OrdBoundPair<T> {
    fn from(value: &Interval<T>) -> Self {
        OrdBoundPair::from(value.0.clone())
    }
}*/

impl<T> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        if value.is_empty() {
            IntervalSet::empty()
        } else {
            IntervalSet::new_unchecked([value])
        }
    }
}

macro_rules! interval_set_delegate_from_impl {
    ($t:ty) => {
        impl<T> From<$t> for IntervalSet<T> {
            fn from(value: $t) -> Self {
                Self::from(Interval::from(value))
            }
        }
    };
}

macro_rules! interval_set_delegate_w_domain_from_impl {
    ($t:ty) => {
        impl<T: $crate::numeric::Element> From<$t> for IntervalSet<T> {
            fn from(value: $t) -> Self {
                Self::from(Interval::from(value))
            }
        }
    };
}

macro_rules! interval_set_delegate_w_domain_zero_from_impl {
    ($t:ty) => {
        impl<T: $crate::numeric::Element + $crate::numeric::Zero> From<$t> for IntervalSet<T> {
            fn from(value: $t) -> Self {
                Self::from(Interval::from(value))
            }
        }
    };
}

interval_set_delegate_from_impl!(FiniteInterval<T>);
interval_set_delegate_from_impl!(HalfInterval<T>);
interval_set_delegate_from_impl!(EnumInterval<T>);
interval_set_delegate_w_domain_from_impl!((T, T));
interval_set_delegate_w_domain_from_impl!([T; 2]);
interval_set_delegate_w_domain_from_impl!(core::ops::Range<T>);
interval_set_delegate_w_domain_from_impl!(core::ops::RangeInclusive<T>);
interval_set_delegate_w_domain_zero_from_impl!(core::ops::RangeFrom<T>);
interval_set_delegate_w_domain_zero_from_impl!(core::ops::RangeTo<T>);
interval_set_delegate_w_domain_zero_from_impl!(core::ops::RangeToInclusive<T>);
interval_set_delegate_from_impl!(core::ops::RangeFull);

impl<T: Element> TryFrom<OrdBoundPair<T>> for IntervalSet<T> {
    type Error = Error;

    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        let success = Interval::<T>::try_from(value)?;
        Ok(IntervalSet::from(success))
    }
}

impl<T> From<Interval<T>> for OrdBoundPair<T> {
    fn from(value: Interval<T>) -> Self {
        OrdBoundPair::from(value.0)
    }
}

impl<'a, T> From<&'a Interval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a Interval<T>) -> Self {
        OrdBoundPair::from(&value.0)
    }
}

impl<T> From<IntervalSet<T>> for OrdBoundPair<T> {
    fn from(value: IntervalSet<T>) -> Self {
        let mut intervals = value.into_raw();
        match intervals.len() {
            0 => OrdBoundPair::empty(),
            1 => intervals.remove(0).into(),
            _ => {
                let first = intervals.swap_remove(0);
                let last = intervals.swap_remove(0);
                let (min, _) = OrdBoundPair::from(first).into_raw();
                let (_, max) = OrdBoundPair::from(last).into_raw();
                OrdBoundPair::new(min, max)
            }
        }
    }
}

impl<'a, T> From<&'a IntervalSet<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a IntervalSet<T>) -> Self {
        let intervals = value.slice();
        match intervals.len() {
            0 => OrdBoundPair::empty(),
            1 => OrdBoundPair::from(intervals.first().unwrap()),
            _ => {
                let first = intervals.first().unwrap();
                let last = intervals.last().unwrap();
                let (min, _) = OrdBoundPair::from(first).into_raw();
                let (_, max) = OrdBoundPair::from(last).into_raw();
                OrdBoundPair::new(min, max)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_convert_tuple() {
        let _: Interval<_> = (0, 10).into();
        let _: Interval<_> = (0.0, 10.0).into();

        let _ = IntervalSet::new([(0, 10), (10, 20)].into_iter().map_into());
        let _: IntervalSet<_> = [(0, 10), (20, 30), (40, 50)].into_iter().collect();

        let _ = IntervalSet::from_iter([(0, 10), (20, 30), (40, 50)]);

        let _ = [(0, 10), (20, 30), (30, 40)]
            .into_iter()
            .map(Interval::from)
            .collect::<IntervalSet<_>>();

        let _ = IntervalSet::from_iter([(0, 5), (20, 25)]);
        let _ = IntervalSet::from_iter([(0.0, 5.0)]);

        //let zzz = IntervalSet::coerse([(0, 10), (20, 30), (40, 50)]);
    }
}

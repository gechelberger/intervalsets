use crate::numeric::Domain;
use crate::{Interval, IntervalSet, MaybeEmpty};

use super::{BoundCase, Finite, HalfBounded};

impl<T: Domain> From<Finite<T>> for BoundCase<T> {
    fn from(value: Finite<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T: Domain> From<HalfBounded<T>> for BoundCase<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self::Half(value)
    }
}

impl<T: Domain> From<BoundCase<T>> for Interval<T> {
    fn from(value: BoundCase<T>) -> Self {
        Self(value)
    }
}

impl<T: Domain> From<Finite<T>> for Interval<T> {
    fn from(value: Finite<T>) -> Self {
        Self::from(BoundCase::from(value))
    }
}

impl<T: Domain> From<HalfBounded<T>> for Interval<T> {
    fn from(value: HalfBounded<T>) -> Self {
        Self::from(BoundCase::from(value))
    }
}

impl<T: Domain> From<(T, T)> for Interval<T> {
    fn from(value: (T, T)) -> Self {
        Self::closed(value.0, value.1)
    }
}

impl<T: Domain> From<&(T, T)> for Interval<T> {
    fn from(value: &(T, T)) -> Self {
        Self::closed(value.0.clone(), value.1.clone())
    }
}

impl<T: Domain> From<Interval<T>> for IntervalSet<T> {
    fn from(value: Interval<T>) -> Self {
        if value.is_empty() {
            return IntervalSet::new_unchecked(vec![]);
        }
        IntervalSet::new_unchecked(vec![value])
    }
}

macro_rules! interval_set_delegate_from_impl {
    ($t:ty) => {
        impl<T: Domain> From<$t> for IntervalSet<T> {
            fn from(value: $t) -> Self {
                Self::from(Interval::from(value))
            }
        }
    };
}

interval_set_delegate_from_impl!(BoundCase<T>);
interval_set_delegate_from_impl!(Finite<T>);
interval_set_delegate_from_impl!(HalfBounded<T>);
interval_set_delegate_from_impl!((T, T));
interval_set_delegate_from_impl!(&(T, T));

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_convert_tuple() {
        let x: Interval<_> = (0, 10).into();
        let x: Interval<_> = (0.0, 10.0).into();

        let z = IntervalSet::new([(0, 10), (10, 20)].iter().map_into());
        let zz: IntervalSet<_> = [(0, 10), (20, 30), (40, 50)].iter().collect();

        let y = IntervalSet::from_iter([(0, 10), (20, 30), (40, 50)]);

        let yy = [(0, 10), (20, 30), (30, 40)]
            .iter()
            .map(Interval::from)
            .collect::<IntervalSet<_>>();

        let y = IntervalSet::from_iter([(0, 5), (20, 25)]);
        let y = IntervalSet::from_iter([(0.0, 5.0)]);

        //let zzz = IntervalSet::coerse([(0, 10), (20, 30), (40, 50)]);
    }
}

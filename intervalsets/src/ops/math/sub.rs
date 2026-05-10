use core::ops::Sub;

use intervalsets_core::sets::EnumInterval;

use crate::error::Error;
use crate::numeric::Element;
use crate::ops::{TrySub, Union};
use crate::{Interval, IntervalSet};

impl<T> TrySub for Interval<T>
where
    EnumInterval<T>: TrySub<EnumInterval<T>, Output = EnumInterval<T>>,
    <EnumInterval<T> as TrySub<EnumInterval<T>>>::Error: Into<Error>,
{
    type Output = Interval<T>;
    type Error = Error;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .try_sub(rhs.0)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T> Sub for Interval<T>
where
    Self: TrySub<Output = Self>,
    <Self as TrySub>::Error: core::fmt::Debug,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> TrySub<Interval<T>> for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TrySub<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Union-fold over already-valid subsets; bypasses IntervalSet::new's
    // re-validation overhead.
    fn try_sub(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter()
            .try_fold(IntervalSet::empty(), |acc, subset| {
                Ok(acc.union(subset.try_sub(rhs.clone())?))
            })
    }
}

impl<T> Sub<Interval<T>> for IntervalSet<T>
where
    Self: TrySub<Interval<T>, Output = Self>,
    <Self as TrySub<Interval<T>>>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn sub(self, rhs: Interval<T>) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> TrySub<IntervalSet<T>> for Interval<T>
where
    T: Element + Clone,
    Self: TrySub<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Sub doesn't commute, so we can't delegate -- iterate rhs directly.
    // Union-fold maintains a sorted accumulator regardless of the order
    // self - subset produces (descending: as subset grows, self - subset
    // shrinks).
    fn try_sub(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        rhs.into_iter()
            .try_fold(IntervalSet::empty(), |acc, subset| {
                Ok(acc.union(self.clone().try_sub(subset)?))
            })
    }
}

impl<T> Sub<IntervalSet<T>> for Interval<T>
where
    Self: TrySub<IntervalSet<T>, Output = IntervalSet<T>>,
    <Self as TrySub<IntervalSet<T>>>::Error: core::fmt::Debug,
{
    type Output = IntervalSet<T>;

    fn sub(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

impl<T> TrySub for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TrySub<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Cartesian product results are unsorted; union-fold incrementally maintains
    // a sorted/disjoint accumulator without paying for IntervalSet::new's
    // re-validation each step.
    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let mut result = IntervalSet::empty();
        for a in self {
            for b in rhs.iter() {
                result = result.union(a.clone().try_sub(b.clone())?);
            }
        }
        Ok(result)
    }
}

impl<T> Sub for IntervalSet<T>
where
    Self: TrySub<Output = Self>,
    <Self as TrySub>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).unwrap()
    }
}

#[cfg(test)]
mod try_tests {
    use super::*;
    use crate::factory::traits::*;

    /// TrySub does not require T: Ord, so raw f64 (which only impls
    /// PartialOrd) can use the panic-free arithmetic path that the
    /// infix Sub operator rejects.
    #[test]
    fn test_try_sub_raw_f64() {
        let a = Interval::open(0.0_f64, 10.0);
        assert_eq!(a.try_sub(a).unwrap(), Interval::open(-10.0_f64, 10.0));
    }

    #[test]
    fn set_level_unsigned_underflow_returns_err() {
        use intervalsets_core::error::MathError;

        let a = Interval::<u32>::closed(0, 5);
        let b = Interval::<u32>::closed(10, 20);
        let r = a.try_sub(b);
        assert!(matches!(r, Err(Error::Math(MathError::Range))));
    }

    #[test]
    #[should_panic]
    fn set_level_unsigned_underflow_infix_panics() {
        let a = Interval::<u32>::closed(0, 5);
        let b = Interval::<u32>::closed(10, 20);
        let _ = a - b;
    }
}

#[cfg(all(test, feature = "ordered-float"))]
mod tests {
    use ordered_float::OrderedFloat as O;

    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_sub_interval() {
        let a = Interval::open(O(0.0), O(10.0));
        assert_eq!(a - a, Interval::open(O(-10.0), O(10.0)));
    }
}

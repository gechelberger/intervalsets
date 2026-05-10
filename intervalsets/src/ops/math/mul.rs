use core::ops::Mul;

use intervalsets_core::sets::EnumInterval;

use crate::error::Error;
use crate::numeric::Element;
use crate::ops::{TryMul, Union};
use crate::{Interval, IntervalSet};

impl<T> TryMul for Interval<T>
where
    EnumInterval<T>: TryMul<EnumInterval<T>, Output = EnumInterval<T>>,
    <EnumInterval<T> as TryMul<EnumInterval<T>>>::Error: Into<Error>,
{
    type Output = Interval<T>;
    type Error = Error;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .try_mul(rhs.0)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T> Mul for Interval<T>
where
    Self: TryMul<Output = Self>,
    <Self as TryMul>::Error: core::fmt::Debug,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.try_mul(rhs).unwrap()
    }
}

impl<T> TryMul<Interval<T>> for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TryMul<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Union-fold over already-valid subsets; bypasses IntervalSet::new's
    // re-validation overhead.
    fn try_mul(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter()
            .try_fold(IntervalSet::empty(), |acc, subset| {
                Ok(acc.union(subset.try_mul(rhs.clone())?))
            })
    }
}

impl<T> Mul<Interval<T>> for IntervalSet<T>
where
    Self: TryMul<Interval<T>, Output = Self>,
    <Self as TryMul<Interval<T>>>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn mul(self, rhs: Interval<T>) -> Self::Output {
        self.try_mul(rhs).unwrap()
    }
}

impl<T> TryMul<IntervalSet<T>> for Interval<T>
where
    T: Element + Clone,
    IntervalSet<T>: TryMul<Interval<T>, Output = IntervalSet<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_mul(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        // commutative: delegate to IntervalSet * Interval
        rhs.try_mul(self)
    }
}

impl<T> Mul<IntervalSet<T>> for Interval<T>
where
    Self: TryMul<IntervalSet<T>, Output = IntervalSet<T>>,
    <Self as TryMul<IntervalSet<T>>>::Error: core::fmt::Debug,
{
    type Output = IntervalSet<T>;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_mul(rhs).unwrap()
    }
}

impl<T> TryMul<IntervalSet<T>> for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TryMul<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Cartesian product results are unsorted; union-fold incrementally maintains
    // a sorted/disjoint accumulator without paying for IntervalSet::new's
    // re-validation each step.
    fn try_mul(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        let mut result = IntervalSet::empty();
        for a in self {
            for b in rhs.iter() {
                result = result.union(a.clone().try_mul(b.clone())?);
            }
        }
        Ok(result)
    }
}

impl<T> Mul<IntervalSet<T>> for IntervalSet<T>
where
    Self: TryMul<Output = Self>,
    <Self as TryMul>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_mul(rhs).unwrap()
    }
}

#[cfg(test)]
mod try_tests {
    use super::*;
    use crate::factory::traits::*;

    /// TryMul does not require T: Ord, so raw f64 (which only impls
    /// PartialOrd) can use the panic-free arithmetic path that the
    /// infix Mul operator rejects.
    #[test]
    fn test_try_mul_raw_f64() {
        let a = Interval::open(2.0_f64, 4.0);
        let b = Interval::open(3.0_f64, 5.0);
        assert_eq!(a.try_mul(b).unwrap(), Interval::open(6.0_f64, 20.0));
    }

    #[test]
    fn set_level_int_overflow_returns_err() {
        use intervalsets_core::error::MathError;

        let a = Interval::<i32>::closed(i32::MAX, i32::MAX);
        let b = Interval::<i32>::closed(2, 2);
        let r = a.try_mul(b);
        assert!(matches!(r, Err(Error::Math(MathError::Range))));
    }

    #[test]
    #[should_panic]
    fn set_level_int_overflow_infix_panics() {
        let a = Interval::<i32>::closed(i32::MAX, i32::MAX);
        let b = Interval::<i32>::closed(2, 2);
        let _ = a * b;
    }
}

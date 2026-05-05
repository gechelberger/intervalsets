use core::ops::Sub;

use crate::error::Error;
use crate::numeric::{Element, Zero};
use crate::ops::{TrySub, Union};
use crate::{Interval, IntervalSet};

impl<T> TrySub for Interval<T>
where
    T: Sub,
    <T as Sub>::Output: Element,
{
    type Output = Interval<<T as Sub>::Output>;
    type Error = Error;

    #[inline]
    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0.try_sub(rhs.0).map_err(Into::into).map(Interval::from)
    }
}

impl<T> Sub for Interval<T>
where
    T: Sub + Ord,
    <T as Sub>::Output: Element + Ord + Zero,
{
    type Output = Interval<<T as Sub>::Output>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).expect("infix Sub invariants guarantee try_sub infallibility")
    }
}

impl<T> TrySub<Interval<T>> for IntervalSet<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element,
{
    type Output = IntervalSet<<T as Sub>::Output>;
    type Error = Error;

    // Union-fold over already-valid subsets; bypasses IntervalSet::new's
    // re-validation overhead.
    fn try_sub(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(subset.try_sub(rhs.clone())?))
        })
    }
}

impl<T> Sub<Interval<T>> for IntervalSet<T>
where
    T: Sub + Element + Ord + Clone,
    <T as Sub>::Output: Element + Ord + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: Interval<T>) -> Self::Output {
        self.try_sub(rhs).expect("infix Sub invariants guarantee try_sub infallibility")
    }
}

impl<T> TrySub<IntervalSet<T>> for Interval<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element,
{
    type Output = IntervalSet<<T as Sub>::Output>;
    type Error = Error;

    // Sub doesn't commute, so we can't delegate -- iterate rhs directly.
    // Union-fold maintains a sorted accumulator regardless of the order
    // self - subset produces (descending: as subset grows, self - subset
    // shrinks).
    fn try_sub(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        rhs.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(self.clone().try_sub(subset)?))
        })
    }
}

impl<T> Sub<IntervalSet<T>> for Interval<T>
where
    T: Sub + Element + Ord + Clone,
    <T as Sub>::Output: Element + Ord + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_sub(rhs).expect("infix Sub invariants guarantee try_sub infallibility")
    }
}

impl<T> TrySub for IntervalSet<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element,
{
    type Output = IntervalSet<<T as Sub>::Output>;
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
    T: Sub + Element + Ord + Clone,
    <T as Sub>::Output: Element + Ord + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(rhs).expect("infix Sub invariants guarantee try_sub infallibility")
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
}

// Float arithmetic tests use OrderedFloat<f64> because the infix Sub
// operator now requires T: Ord and raw f64 doesn't satisfy that.
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

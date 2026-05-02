use core::ops::Mul;

use crate::error::Error;
use crate::numeric::{Element, Zero};
use crate::ops::{TryMul, Union};
use crate::{Interval, IntervalSet};

impl<T> TryMul for Interval<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = Interval<<T as Mul>::Output>;
    type Error = Error;

    #[inline]
    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0.try_mul(rhs.0).map(Interval::from)
    }
}

impl<T> Mul for Interval<T>
where
    T: Mul + Element + Ord + Clone + Zero,
    <T as Mul>::Output: Element + Ord + Zero + Clone,
{
    type Output = Interval<<T as Mul>::Output>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.try_mul(rhs).expect("infix Mul invariants guarantee try_mul infallibility")
    }
}

impl<T> TryMul<Interval<T>> for IntervalSet<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;
    type Error = Error;

    // Union-fold over already-valid subsets; bypasses IntervalSet::new's
    // re-validation overhead.
    fn try_mul(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(subset.try_mul(rhs.clone())?))
        })
    }
}

impl<T> Mul<Interval<T>> for IntervalSet<T>
where
    T: Mul + Element + Ord + Clone + Zero,
    <T as Mul>::Output: Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: Interval<T>) -> Self::Output {
        self.try_mul(rhs).expect("infix Mul invariants guarantee try_mul infallibility")
    }
}

impl<T> TryMul<IntervalSet<T>> for Interval<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;
    type Error = Error;

    fn try_mul(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        // commutative: delegate to IntervalSet * Interval
        rhs.try_mul(self)
    }
}

impl<T> Mul<IntervalSet<T>> for Interval<T>
where
    T: Mul + Element + Ord + Clone + Zero,
    <T as Mul>::Output: Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_mul(rhs).expect("infix Mul invariants guarantee try_mul infallibility")
    }
}

impl<T> TryMul<IntervalSet<T>> for IntervalSet<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;
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
    T: Mul + Element + Ord + Clone + Zero,
    <T as Mul>::Output: Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_mul(rhs).expect("infix Mul invariants guarantee try_mul infallibility")
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
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_foo() {
        let nz: f32 = -0.0;
        let pz: f32 = 0.0;

        assert_eq!(nz, pz);
    }
}

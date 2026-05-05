use core::ops::Add;

use crate::error::Error;
use crate::numeric::{Element, Zero};
use crate::ops::{TryAdd, Union};
use crate::{Interval, IntervalSet};

impl<T> TryAdd for Interval<T>
where
    T: Add,
    <T as Add>::Output: Element,
{
    type Output = Interval<<T as Add>::Output>;
    type Error = Error;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0.try_add(rhs.0).map_err(Into::into).map(Interval::from)
    }
}

impl<T> Add for Interval<T>
where
    T: Add + Ord + Clone + Zero,
    <T as Add>::Output: Element + Ord + Zero,
{
    type Output = Interval<<T as Add>::Output>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        // T: Ord makes partial_cmp on bounds total, so try_add is provably
        // infallible -- no path inside reaches a TotalOrderError.
        self.try_add(rhs).expect("infix Add invariants guarantee try_add infallibility")
    }
}

impl<T> TryAdd<Interval<T>> for IntervalSet<T>
where
    T: Add + Element + Clone,
    <T as Add>::Output: Element,
{
    type Output = IntervalSet<<T as Add>::Output>;
    type Error = Error;

    // Union-fold: each subset is already a valid Interval and Self maintains
    // the IntervalSet invariants by construction; calling Union avoids the
    // public IntervalSet::new validation overhead (and its NaN-panic path).
    fn try_add(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(subset.try_add(rhs.clone())?))
        })
    }
}

impl<T> Add<Interval<T>> for IntervalSet<T>
where
    T: Add + Element + Ord + Clone + Zero,
    <T as Add>::Output: Element + Ord + Zero,
{
    type Output = IntervalSet<<T as Add>::Output>;

    fn add(self, rhs: Interval<T>) -> Self::Output {
        self.try_add(rhs).expect("infix Add invariants guarantee try_add infallibility")
    }
}

impl<T> TryAdd<IntervalSet<T>> for Interval<T>
where
    T: Add + Element + Clone,
    <T as Add>::Output: Element,
{
    type Output = IntervalSet<<T as Add>::Output>;
    type Error = Error;

    fn try_add(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        // commutative: delegate to IntervalSet + Interval
        rhs.try_add(self)
    }
}

impl<T> Add<IntervalSet<T>> for Interval<T>
where
    T: Add + Element + Ord + Clone + Zero,
    <T as Add>::Output: Element + Ord + Zero,
{
    type Output = IntervalSet<<T as Add>::Output>;

    fn add(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_add(rhs).expect("infix Add invariants guarantee try_add infallibility")
    }
}

impl<T> TryAdd for IntervalSet<T>
where
    T: Add<T, Output = T> + Element + Clone,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Cartesian product results are unsorted; union-fold incrementally maintains
    // a sorted/disjoint accumulator without paying for IntervalSet::new's
    // re-validation each step.
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let mut result = IntervalSet::empty();
        for l in self {
            for r in rhs.iter() {
                result = result.union(l.clone().try_add(r.clone())?);
            }
        }
        Ok(result)
    }
}

impl<T> Add for IntervalSet<T>
where
    T: Add<T, Output = T> + Ord + Clone + Zero + Element,
{
    type Output = IntervalSet<T>;

    fn add(self, rhs: Self) -> Self::Output {
        self.try_add(rhs).expect("infix Add invariants guarantee try_add infallibility")
    }
}

#[cfg(test)]
mod try_tests {
    use super::*;
    use crate::factory::traits::*;

    /// TryAdd does not require T: Ord, so raw f64 (which only impls
    /// PartialOrd) can use the panic-free arithmetic path that the
    /// infix Add operator rejects.
    #[test]
    fn test_try_add_raw_f64() {
        let a = Interval::open(0.0_f64, 10.0);
        let b = Interval::open(10.0_f64, 20.0);
        assert_eq!(a.try_add(b).unwrap(), Interval::open(10.0_f64, 30.0));
    }
}

// Float arithmetic tests use OrderedFloat<f64> because the infix Add
// operator now requires T: Ord and raw f64 doesn't satisfy that.
#[cfg(all(test, feature = "ordered-float"))]
mod tests {
    use ordered_float::OrderedFloat as O;

    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_add_interval() {
        let a = Interval::open(O(0.0), O(10.0));
        let b = Interval::open(O(10.0), O(20.0));
        assert_eq!(a + b, Interval::open(O(10.0), O(30.0)));
    }

    #[test]
    fn test_add_sets() {
        let a = IntervalSet::new([(O(-100.0), O(-90.0)).into(), [O(0.0), O(10.0)].into()]);
        let b = IntervalSet::new([[O(0.0), O(10.0)].into(), [O(20.0), O(30.0)].into()]);

        assert_eq!(
            a + b,
            IntervalSet::new([
                (O(-100.0), O(-80.0)).into(),
                (O(-80.0), O(-60.0)).into(),
                [O(0.0), O(40.0)].into(),
            ])
        );
    }

    #[test]
    fn test_re_anchor() {
        let a = Interval::singleton(O(100.0));
        let b = Interval::open(O(10.0), O(20.0));

        let offset = a - b;
        assert_eq!(offset, Interval::open(O(80.0), O(90.0)))
    }
}

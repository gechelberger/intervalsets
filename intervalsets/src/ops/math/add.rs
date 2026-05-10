use core::ops::Add;

use intervalsets_core::sets::EnumInterval;

use crate::error::Error;
use crate::numeric::Element;
use crate::ops::{TryAdd, Union};
use crate::{Interval, IntervalSet};

// Wrapper-crate set-level math delegates to the core impls and lifts
// any `intervalsets_core::error::Error` to `crate::error::Error` via
// the `From<CoreError>` impl in `crate::error`. The bounds talk only
// in terms of "the wrapped type can do the op" + "its error converts
// to ours", so callers don't see core's error type at the wrapper
// surface.

impl<T> TryAdd for Interval<T>
where
    EnumInterval<T>: TryAdd<EnumInterval<T>, Output = EnumInterval<T>>,
    <EnumInterval<T> as TryAdd<EnumInterval<T>>>::Error: Into<Error>,
{
    type Output = Interval<T>;
    type Error = Error;

    #[inline]
    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .try_add(rhs.0)
            .map_err(Into::into)
            .map(Interval::from)
    }
}

impl<T> Add for Interval<T>
where
    Self: TryAdd<Output = Self>,
    <Self as TryAdd>::Error: core::fmt::Debug,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.try_add(rhs).unwrap()
    }
}

impl<T> TryAdd<Interval<T>> for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TryAdd<Interval<T>, Output = Interval<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    // Union-fold: each subset is already a valid Interval and Self maintains
    // the IntervalSet invariants by construction; calling Union avoids the
    // public IntervalSet::new validation overhead (and its NaN-panic path).
    fn try_add(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter()
            .try_fold(IntervalSet::empty(), |acc, subset| {
                Ok(acc.union(subset.try_add(rhs.clone())?))
            })
    }
}

impl<T> Add<Interval<T>> for IntervalSet<T>
where
    Self: TryAdd<Interval<T>, Output = Self>,
    <Self as TryAdd<Interval<T>>>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn add(self, rhs: Interval<T>) -> Self::Output {
        self.try_add(rhs).unwrap()
    }
}

impl<T> TryAdd<IntervalSet<T>> for Interval<T>
where
    T: Element + Clone,
    IntervalSet<T>: TryAdd<Interval<T>, Output = IntervalSet<T>, Error = Error>,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_add(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        // commutative: delegate to IntervalSet + Interval
        rhs.try_add(self)
    }
}

impl<T> Add<IntervalSet<T>> for Interval<T>
where
    Self: TryAdd<IntervalSet<T>, Output = IntervalSet<T>>,
    <Self as TryAdd<IntervalSet<T>>>::Error: core::fmt::Debug,
{
    type Output = IntervalSet<T>;

    fn add(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_add(rhs).unwrap()
    }
}

impl<T> TryAdd for IntervalSet<T>
where
    T: Element + Clone,
    Interval<T>: TryAdd<Interval<T>, Output = Interval<T>, Error = Error>,
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
    Self: TryAdd<Output = Self>,
    <Self as TryAdd>::Error: core::fmt::Debug,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.try_add(rhs).unwrap()
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

    /// Set-level integer overflow surfaces as `Err` on `try_add` and
    /// panics on `+` (Tier 3b).
    #[test]
    fn set_level_int_overflow_returns_err() {
        use intervalsets_core::error::MathError;

        let a = Interval::<i32>::closed(i32::MAX, i32::MAX);
        let b = Interval::<i32>::closed(1, 1);
        let r = a.try_add(b);
        assert!(matches!(r, Err(Error::Math(MathError::Range))));
    }

    #[test]
    #[should_panic]
    fn set_level_int_overflow_infix_panics() {
        let a = Interval::<i32>::closed(i32::MAX, i32::MAX);
        let b = Interval::<i32>::closed(1, 1);
        let _ = a + b;
    }
}

// OrderedFloat tests — exercise the infix path, since OrderedFloat
// satisfies the Try* bounds via the core's macro-generated impls.
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

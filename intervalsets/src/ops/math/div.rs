use core::ops::Div;

use crate::error::Error;
use crate::numeric::{Element, Zero};
use crate::ops::{TryDiv, Union};
use crate::{Interval, IntervalSet};

impl<T> TryDiv for Interval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let divided = self.0.try_div(rhs.0)?;
        // SAFETY: MaybeDisjoint guarantees sorted, disjoint, non-empty intervals.
        Ok(unsafe { IntervalSet::new_unchecked(divided.map(Interval::from)) })
    }
}

impl<T> Div for Interval<T>
where
    T: Div<Output = T> + Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Self) -> Self::Output {
        self.try_div(rhs).expect("infix Div invariants guarantee try_div infallibility")
    }
}

impl<T> TryDiv<Interval<T>> for IntervalSet<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_div(self, rhs: Interval<T>) -> Result<Self::Output, Self::Error> {
        self.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(subset.try_div(rhs.clone())?))
        })
    }
}

impl<T> Div<Interval<T>> for IntervalSet<T>
where
    T: Div<Output = T> + Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Interval<T>) -> Self::Output {
        self.try_div(rhs).expect("infix Div invariants guarantee try_div infallibility")
    }
}

impl<T> TryDiv<IntervalSet<T>> for Interval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_div(self, rhs: IntervalSet<T>) -> Result<Self::Output, Self::Error> {
        rhs.into_iter().try_fold(IntervalSet::empty(), |acc, subset| {
            Ok(acc.union(self.clone().try_div(subset)?))
        })
    }
}

impl<T> Div<IntervalSet<T>> for Interval<T>
where
    T: Div<Output = T> + Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: IntervalSet<T>) -> Self::Output {
        self.try_div(rhs).expect("infix Div invariants guarantee try_div infallibility")
    }
}

impl<T> TryDiv for IntervalSet<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;
    type Error = Error;

    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.into_iter().try_fold(IntervalSet::empty(), |acc, l_subset| {
            Ok(acc.union(l_subset.try_div(rhs.clone())?))
        })
    }
}

impl<T> Div for IntervalSet<T>
where
    T: Div<Output = T> + Element + Ord + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Self) -> Self::Output {
        self.try_div(rhs).expect("infix Div invariants guarantee try_div infallibility")
    }
}

#[cfg(test)]
mod try_tests {
    use super::*;
    use crate::factory::traits::*;

    /// TryDiv does not require T: Ord, so raw f64 (which only impls
    /// PartialOrd) can use the panic-free arithmetic path that the
    /// infix Div operator rejects.
    #[test]
    fn test_try_div_raw_f64() {
        let a = Interval::open(10.0_f64, 20.0);
        let b = Interval::closed_unbound(10.0_f64);
        assert_eq!(
            a.try_div(b).unwrap(),
            Interval::open(0.0_f64, 2.0).into()
        );
    }
}

// Float arithmetic tests use OrderedFloat<f64> because the infix Div
// operator now requires T: Ord and raw f64 doesn't satisfy that.
#[cfg(all(test, feature = "ordered-float"))]
mod tests {
    use ordered_float::OrderedFloat as O;

    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_set_div_set() {
        let numer = IntervalSet::from_iter([[O(10.0), O(12.0)], [O(20.0), O(22.0)]]);
        let denom = IntervalSet::from_iter([[O(1.0), O(2.0)], [O(10.0), O(11.0)]]);

        let expected = IntervalSet::from_iter([
            [O(5.0), O(22.0)],
            [O(10.0 / 11.0), O(1.2)],
            [O(20.0 / 11.0), O(2.2)],
        ]);

        assert_eq!(numer / denom, expected);
    }

    #[test]
    fn test_div_degenerate() {
        let e = Interval::<O<f64>>::empty();
        let zero = Interval::singleton(O(0.0));
        assert_eq!(e / e, e.into());
        assert_eq!(zero / e, e.into());
        assert_eq!(e / zero, e.into());

        let one = Interval::singleton(O(1.0));
        assert_eq!(zero / one, zero.into());
        assert_eq!(one / zero, e.into());
    }

    #[test]
    fn test_div_halfs() {
        let x = Interval::closed_unbound(O(-1.0));
        let y = Interval::unbound_closed(O(-10.0));
        assert_eq!(x / y, Interval::unbound_closed(O(0.1)).into());

        assert_eq!(
            x / Interval::unbound_open(O(0.0)),
            Interval::unbounded().into()
        );

        let x = Interval::unbound_closed(O(1.0));
        assert_eq!(x / y, Interval::closed_unbound(O(-0.1)).into());
    }

    #[test]
    fn test_div_by_half() {
        assert_eq!(
            Interval::open(O(10.0), O(20.0)) / Interval::closed_unbound(O(10.0)),
            Interval::open(O(0.0), O(2.0)).into()
        );

        assert_eq!(
            Interval::open(O(-20.0), O(-10.0)) / Interval::unbound_closed(O(-10.0)),
            Interval::open(O(0.0), O(2.0)).into()
        );

        assert_eq!(
            Interval::open(O(10.0), O(20.0)) / Interval::unbound_closed(O(-10.0)),
            Interval::open(O(-2.0), O(0.0)).into()
        );

        assert_eq!(
            Interval::open(O(-20.0), O(-10.0)) / Interval::closed_unbound(O(10.0)),
            Interval::open(O(-2.0), O(0.0)).into()
        );

        assert_eq!(
            Interval::closed(O(-1.0), O(2.0)) / Interval::closed_unbound(O(10.0)),
            Interval::closed(O(-0.1), O(0.2)).into()
        );

        assert_eq!(
            Interval::closed(O(-1.0), O(2.0)) / Interval::unbound_closed(O(-10.0)),
            Interval::closed(O(-0.2), O(0.1)).into()
        );

        assert_eq!(
            Interval::closed(O(0.0), O(1.0)) / Interval::unbound_closed(O(1.0)),
            Interval::unbounded().into()
        );
    }

    #[test]
    fn test_div_intervals() {
        let fc = Interval::closed;
        let uc = Interval::unbound_closed;
        let cu = Interval::closed_unbound;

        assert_eq!(
            fc(O(-10.0), O(-1.0)) / fc(O(-1.0), O(1.0)),
            IntervalSet::new([uc(O(-1.0)), cu(O(1.0))])
        );
    }
}

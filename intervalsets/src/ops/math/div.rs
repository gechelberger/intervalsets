use core::ops::Div;

use crate::numeric::{Element, Zero};
use crate::ops::Union;
use crate::{Interval, IntervalSet};

impl<T> Div for Interval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Self) -> Self::Output {
        let divided = self.0 / rhs.0;
        unsafe { IntervalSet::new_unchecked(divided.map(Interval::from)) }
    }
}

impl<T> Div<Interval<T>> for IntervalSet<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Interval<T>) -> Self::Output {
        self.into_iter()
            .map(|subset| subset / rhs.clone())
            .fold(IntervalSet::empty(), IntervalSet::union)
    }
}

impl<T> Div<IntervalSet<T>> for Interval<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: IntervalSet<T>) -> Self::Output {
        rhs.into_iter()
            .map(|subset| self.clone() / subset)
            .fold(IntervalSet::empty(), IntervalSet::union)
    }
}

impl<T> Div for IntervalSet<T>
where
    T: Div<Output = T> + Element + Zero + Clone,
{
    type Output = IntervalSet<T>;

    fn div(self, rhs: Self) -> Self::Output {
        self.into_iter()
            .map(|il| il / rhs.clone())
            .fold(IntervalSet::empty(), IntervalSet::union)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_set_div_set() {
        let numer = IntervalSet::from_iter([[10.0, 12.0], [20.0, 22.0]]);
        let denom = IntervalSet::from_iter([[1.0, 2.0], [10.0, 11.0]]);

        let expected =
            IntervalSet::from_iter([[5.0, 22.0], [10.0 / 11.0, 1.2], [20.0 / 11.0, 2.2]]);

        assert_eq!(numer / denom, expected);
    }

    #[test]
    fn test_div_degenerate() {
        let e = Interval::<f64>::empty();
        let zero = Interval::singleton(0.0);
        assert_eq!(e / e, e.into());
        assert_eq!(zero / e, e.into());
        assert_eq!(e / zero, e.into());

        let one = Interval::singleton(1.0);
        assert_eq!(zero / one, zero.into());
        assert_eq!(one / zero, e.into());
    }

    #[test]
    fn test_div_halfs() {
        let x = Interval::closed_unbound(-1.0);
        let y = Interval::unbound_closed(-10.0);
        assert_eq!(x / y, Interval::unbound_closed(0.1).into());

        assert_eq!(
            x / Interval::unbound_open(0.0),
            Interval::unbounded().into()
        );

        let x = Interval::unbound_closed(1.0);
        assert_eq!(x / y, Interval::closed_unbound(-0.1).into());
    }

    #[test]
    fn test_div_by_half() {
        assert_eq!(
            Interval::open(10.0, 20.0) / Interval::closed_unbound(10.0),
            Interval::open(0.0, 2.0).into()
        );

        assert_eq!(
            Interval::open(-20.0, -10.0) / Interval::unbound_closed(-10.0),
            Interval::open(0.0, 2.0).into()
        );

        assert_eq!(
            Interval::open(10.0, 20.0) / Interval::unbound_closed(-10.0),
            Interval::open(-2.0, 0.0).into()
        );

        assert_eq!(
            Interval::open(-20.0, -10.0) / Interval::closed_unbound(10.0),
            Interval::open(-2.0, 0.0).into()
        );

        assert_eq!(
            Interval::closed(-1.0, 2.0) / Interval::closed_unbound(10.0),
            Interval::closed(-0.1, 0.2).into()
        );

        assert_eq!(
            Interval::closed(-1.0, 2.0) / Interval::unbound_closed(-10.0),
            Interval::closed(-0.2, 0.1).into()
        );

        assert_eq!(
            Interval::closed(0.0, 1.0) / Interval::unbound_closed(1.0),
            Interval::unbounded().into()
        );
    }

    #[test]
    fn test_div_intervals() {
        let fc = Interval::closed;
        let uc = Interval::unbound_closed;
        let cu = Interval::closed_unbound;

        assert_eq!(
            fc(-10.0, -1.0) / fc(-1.0, 1.0),
            IntervalSet::new([uc(-1.0), cu(1.0)])
        );
    }
}

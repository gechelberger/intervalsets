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

    #[test]
    fn test_foo() {
        let numer = IntervalSet::from_iter([[10.0, 12.0], [20.0, 22.0]]);

        let denom = IntervalSet::from_iter([[1.0, 2.0], [10.0, 11.0]]);

        let expected =
            IntervalSet::from_iter([[5.0, 22.0], [10.0 / 11.0, 1.2], [20.0 / 11.0, 2.2]]);

        assert_eq!(numer / denom, expected);
    }
}

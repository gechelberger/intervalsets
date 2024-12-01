use core::ops::Add;

use crate::numeric::{Element, Zero};
use crate::ops::Union;
use crate::{Interval, IntervalSet};

impl<T> Add for Interval<T>
where
    T: Add + PartialOrd + Clone + Zero,
    <T as Add>::Output: Element + Zero,
{
    type Output = Interval<<T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Interval::from(self.0 + rhs.0)
    }
}

impl<T> Add<Interval<T>> for IntervalSet<T>
where
    T: Add + PartialOrd + Clone + Zero,
    <T as Add>::Output: Element + Zero,
{
    type Output = IntervalSet<<T as Add>::Output>;

    fn add(self, rhs: Interval<T>) -> Self::Output {
        IntervalSet::new(self.into_iter().map(|s| s + rhs.clone()))
    }
}

impl<T> Add<IntervalSet<T>> for Interval<T>
where
    T: Add + PartialOrd + Clone + Zero,
    <T as Add>::Output: Element + Zero,
{
    type Output = IntervalSet<<T as Add>::Output>;

    fn add(self, rhs: IntervalSet<T>) -> Self::Output {
        rhs + self
    }
}

impl<T> Add for IntervalSet<T>
where
    T: Add<T, Output = T> + PartialOrd + Clone + Zero + Element,
{
    type Output = IntervalSet<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = IntervalSet::empty();
        for l_subset in self {
            for r_subset in rhs.iter() {
                result = result.union(l_subset.clone() + r_subset.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_add_interval() {
        let a = Interval::open(0.0, 10.0);
        let b = Interval::open(10.0, 20.0);
        assert_eq!(a + b, Interval::open(10.0, 30.0));
    }

    #[test]
    fn test_add_sets() {
        let a = IntervalSet::new([(-100.0, -90.0).into(), [0.0, 10.0].into()]);
        let b = IntervalSet::new([[0.0, 10.0].into(), [20.0, 30.0].into()]);

        assert_eq!(
            a + b,
            IntervalSet::new([
                (-100.0, -80.0).into(),
                (-80.0, -60.0).into(),
                [0.0, 40.0].into(),
            ])
        );
    }

    #[test]
    fn test_re_anchor() {
        let a = Interval::singleton(100.0);
        let b = Interval::open(10.0, 20.0);

        let offset = a - b;
        assert_eq!(offset, Interval::open(80.0, 90.0))
    }
}

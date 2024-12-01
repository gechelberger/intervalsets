use core::ops::Sub;

use crate::numeric::{Element, Zero};
use crate::ops::Union;
use crate::{Interval, IntervalSet};

impl<T> Sub for Interval<T>
where
    T: Sub,
    <T as Sub>::Output: Element + Zero,
{
    type Output = Interval<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Interval::from(self.0 - rhs.0)
    }
}

impl<T> Sub<Interval<T>> for IntervalSet<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: Interval<T>) -> Self::Output {
        Self::Output::new(self.into_iter().map(|subset| subset - rhs.clone()))
    }
}

impl<T> Sub<IntervalSet<T>> for Interval<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: IntervalSet<T>) -> Self::Output {
        Self::Output::new(rhs.into_iter().map(|subset| self.clone() - subset))
    }
}

impl<T> Sub for IntervalSet<T>
where
    T: Sub + Element + Clone,
    <T as Sub>::Output: Element + Zero,
{
    type Output = IntervalSet<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = IntervalSet::empty();
        for a in self {
            for b in rhs.iter() {
                res = res.union(a.clone() - b.clone())
            }
        }
        res

        // self.into_iter().fold(IntervalSet::empty(), |res, l_subset| {
        //     rhs.iter().fold(res, |res, r_subset| {
        //         res.union(l_subset.clone() - r_subset.clone())
        //     })
        // })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_sub_interval() {
        let a = Interval::open(0.0, 10.0);
        assert_eq!(a - a, Interval::open(-10.0, 10.0));
    }
}

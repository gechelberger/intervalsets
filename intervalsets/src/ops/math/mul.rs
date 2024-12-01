use std::ops::Mul;

use crate::numeric::{Element, Zero};
use crate::ops::Union;
use crate::sets::{Interval, IntervalSet};

impl<T> Mul for Interval<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = Interval<<T as Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        Interval::from(self.0 * rhs.0)
    }
}

impl<T> Mul<IntervalSet<T>> for Interval<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        IntervalSet::new(rhs.into_iter().map(|subset| self.clone() * subset))
    }
}

impl<T> Mul<Interval<T>> for IntervalSet<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: Interval<T>) -> Self::Output {
        IntervalSet::new(self.into_iter().map(|subset| subset * rhs.clone()))
    }
}

impl<T> Mul<IntervalSet<T>> for IntervalSet<T>
where
    T: Mul + Element + Clone + Zero,
    <T as Mul>::Output: Element + Zero + Clone,
{
    type Output = IntervalSet<<T as Mul>::Output>;

    fn mul(self, rhs: IntervalSet<T>) -> Self::Output {
        let mut res = IntervalSet::empty();
        for a in self {
            for b in rhs.iter() {
                res = res.union(a.clone() * b.clone())
            }
        }
        res
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

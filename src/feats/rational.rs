use num_rational::Ratio;
use num_traits::{CheckedAdd, CheckedMul, CheckedSub};

use crate::numeric::{Numeric, NumericSet};

impl<T> Numeric for Ratio<T>
where
    T: Clone + CheckedSub + CheckedAdd + CheckedMul + num_integer::Integer,
{
    fn numeric_set() -> NumericSet {
        NumericSet::Real //todo
    }

    fn try_finite_add(&self, rhs: &Self) -> Option<Self> {
        self.checked_add(rhs)
    }

    fn try_finite_sub(&self, rhs: &Self) -> Option<Self> {
        self.checked_sub(rhs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ISize, Interval, Sizable};
    use num_rational::BigRational;

    #[test]
    fn test_rationals() {
        let a: BigRational = BigRational::new(100.into(), 1.into());
        let b: BigRational = BigRational::new(200.into(), 1.into());

        let iv = Interval::closed(a.clone(), b);
        assert_eq!(iv.size().unwrap(), a);
    }
}

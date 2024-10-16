use crate::continuous_domain_impl;
use crate::ival::Side;
use crate::numeric::Domain;
use num_rational::{BigRational, Ratio};

continuous_domain_impl!(Ratio<i32>);
continuous_domain_impl!(Ratio<i64>);
continuous_domain_impl!(BigRational);

#[cfg(test)]
mod tests {
    use crate::measure::width::Width;
    use crate::Interval;
    use num_rational::BigRational;

    #[test]
    fn test_rationals() {
        let a: BigRational = BigRational::new(100.into(), 1.into());
        let b: BigRational = BigRational::new(200.into(), 1.into());

        let iv = Interval::closed(a.clone(), b);
        assert_eq!(iv.width().finite(), a);
    }
}

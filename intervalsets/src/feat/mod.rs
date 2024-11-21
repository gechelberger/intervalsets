//! optional features
//!
//! Most functionality lives in intervalsets-core, and these
//! modules mostly test interoperability with the 'Interval'
//! and 'IntervalSet' types.

#[cfg(feature = "arbitrary")]
mod arbitrary;

#[cfg(feature = "quickcheck")]
mod quickcheck;

#[cfg(all(test, feature = "rust_decimal"))]
mod rust_decimal_tests {
    use ::rust_decimal::Decimal;

    use crate::prelude::*;

    #[test]
    fn test_decimal_interval() {
        let interval = Interval::open(Decimal::new(202, 2), Decimal::new(100, 1));

        assert!(interval.contains(&Decimal::new(5, 0)));
        assert!(!interval.contains(&Decimal::new(10, 0)));

        assert_eq!(interval.width().finite(), Decimal::new(798, 2));
    }
}

#[cfg(all(test, feature = "num-bigint"))]
mod num_bigint_tests {
    use ::num_bigint::ToBigInt;

    use crate::factory::FiniteFactory;
    use crate::measure::Width;
    use crate::Interval;

    #[test]
    fn test_bigint() {
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a.clone(), b);
        assert_eq!(interval.width().finite(), a);
    }
}

// #[cfg(feature = "uom")]
// pub mod uom;

#[cfg(all(test, feature = "ordered-float"))]
mod ordered_float_tests {

    use ::ordered_float::{NotNan, OrderedFloat};

    use crate::factory::{FiniteFactory, IFactory};
    use crate::ops::Intersection;
    use crate::{Interval, SetBounds};

    #[test]
    fn test_not_nan() {
        let x = Interval::open(NotNan::new(5.0).unwrap(), NotNan::new(15.0).unwrap());
        let y = Interval::open(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap());

        assert_eq!(
            x.intersection(y),
            Interval::open(NotNan::new(5.0).unwrap(), NotNan::new(10.0).unwrap())
        );
    }

    #[test]
    fn test_ordered_float() {
        let x = Interval::open(OrderedFloat(0.0), OrderedFloat(10.0));
        let y = Interval::open(OrderedFloat(5.0), OrderedFloat(15.0));

        assert_eq!(
            x.intersection(y),
            Interval::open(OrderedFloat(5.0), OrderedFloat(10.0)),
        );
    }

    #[test]
    fn test_float_not_nan_cvt() {
        type A = IFactory<f32, NotNan<f32>>;
        let x = A::closed(0.0, 5.0);

        assert_eq!(x.left().unwrap().value(), &NotNan::<f32>::new(0.0).unwrap());
    }

    #[test]
    fn test_float_ordered_cvt() {
        type A = IFactory<f32, OrderedFloat<f32>>;
        let x = A::open(0.0, 5.0);
        assert_eq!(x.left().unwrap().value(), &OrderedFloat(0.0));
    }
}

#[cfg(all(test, feature = "fixed"))]
mod fixed_tests {
    use crate::prelude::*;

    #[test]
    fn test_fixed() {
        let x = Interval::closed(
            ::fixed::types::I6F2::from_num(10.5),
            ::fixed::types::I6F2::from_num(15.75),
        );

        assert_eq!(x.width().finite(), fixed::types::I6F2::from_num(5.25));
    }
}

#[cfg(all(test, feature = "bigdecimal"))]
mod bigdecimal_tests {
    use ::bigdecimal::BigDecimal;
    use ::num_traits::FromPrimitive;

    use crate::prelude::*;

    #[test]
    fn test_bigdecimal() {
        let width = BigDecimal::from_f32(123847383748.0).unwrap();
        let x = Interval::closed(BigDecimal::from_f32(0.0).unwrap(), width.clone());

        assert_eq!(x.width().finite(), width);
    }
}

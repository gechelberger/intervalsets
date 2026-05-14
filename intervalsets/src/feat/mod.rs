//! optional features
//!
//! Most functionality lives in intervalsets-core, and these
//! modules mostly test interoperability with the 'Interval'
//! and 'IntervalSet' types.

#[cfg(feature = "approx")]
mod approx;

#[cfg(feature = "arbitrary")]
mod arbitrary;

#[cfg(any(feature = "quickcheck", test))]
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

        assert_eq!(interval.measure().finite(), Decimal::new(798, 2));
    }
}

#[cfg(all(test, feature = "num-bigint"))]
mod num_bigint_tests {
    use ::num_bigint::{BigInt, ToBigInt};

    use crate::factory::FiniteFactory;
    use crate::measure::Measure;
    use crate::Interval;

    #[test]
    fn test_bigint() {
        // BigInt is discrete, so .measure() is cardinality (count = b-a+1).
        let a = 100.to_bigint().unwrap();
        let b = 200.to_bigint().unwrap();
        let interval = Interval::closed(a, b);
        assert_eq!(interval.measure().finite(), BigInt::from(101));
    }
}

#[cfg(all(test, feature = "ordered-float"))]
mod ordered_float_tests {

    use ::ordered_float::{NotNan, OrderedFloat};

    use crate::factory::FiniteFactory;
    use crate::ops::Intersection;
    use crate::Interval;

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

    // === Cast coverage at the Interval / IntervalSet layer ===

    #[test]
    fn test_interval_try_cast_ordered_float() {
        use crate::prelude::TryCast;
        let x = Interval::closed(OrderedFloat(0.0_f32), OrderedFloat(10.0_f32));
        let y: Interval<OrderedFloat<f64>> = x.try_cast().unwrap();
        assert_eq!(
            y,
            Interval::closed(OrderedFloat(0.0_f64), OrderedFloat(10.0_f64))
        );
    }

    #[test]
    fn test_interval_lossy_cast_ordered_float_narrowing() {
        use crate::prelude::LossyCast;
        let x = Interval::closed(OrderedFloat(0.0_f64), OrderedFloat(f64::MAX));
        let y: Interval<OrderedFloat<f32>> = x.lossy_cast();
        assert!(y.is_fully_bounded());
    }

    #[test]
    fn test_interval_cast_not_nan_widening() {
        use crate::prelude::Cast;
        let x = Interval::closed(
            NotNan::new(0.0_f32).unwrap(),
            NotNan::new(10.0_f32).unwrap(),
        );
        let y: Interval<NotNan<f64>> = x.cast();
        assert_eq!(
            y,
            Interval::closed(
                NotNan::new(0.0_f64).unwrap(),
                NotNan::new(10.0_f64).unwrap()
            )
        );
    }

    #[test]
    fn test_interval_set_lossy_cast_ordered_float() {
        use crate::prelude::LossyCast;
        use crate::IntervalSet;
        let set: IntervalSet<OrderedFloat<f64>> = IntervalSet::new([
            Interval::closed(OrderedFloat(0.0), OrderedFloat(10.0)),
            Interval::closed(OrderedFloat(20.0), OrderedFloat(30.0)),
        ]);
        let narrowed: IntervalSet<OrderedFloat<f32>> = set.lossy_cast();
        assert_eq!(narrowed.slice().len(), 2);
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

        // Fixed-point is discrete: .measure() is cardinality (ULP count).
        // I6F2 step = 0.25; count of [10.5, 15.75] = 22 (5.25 / 0.25 + 1).
        assert_eq!(x.measure().finite(), 22_u128);
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

        assert_eq!(x.measure().finite(), width);
    }
}

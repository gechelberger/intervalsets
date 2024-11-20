#[cfg(test)]
mod tests {

    use ordered_float::{NotNan, OrderedFloat};

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

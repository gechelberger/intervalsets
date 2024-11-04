use ordered_float::{FloatCore, NotNan, OrderedFloat};

use crate::factory::Cvt;
use crate::numeric::{Domain, LibZero};

impl<T> LibZero for NotNan<T> {
    fn new_zero() -> Self {
        unsafe { Self::new_unchecked(0.0) }
    }
}

impl<T> LibZero for OrderedFloat<T> {
    fn new_zero() -> Self {
        Self(0.0)
    }
}

impl<T: Clone + PartialOrd> Domain for NotNan<T> {
    fn try_adjacent(&self, side: crate::Side) -> Option<Self> {
        None
    }
}

impl<T: FloatCore> Domain for OrderedFloat<T> {
    fn try_adjacent(&self, side: crate::Side) -> Option<Self> {
        None
    }
}

impl<T> crate::factory::Cvt<T> for NotNan<T>
where
    T: FloatCore,
{
    type To = Self;

    fn convert_to(value: T) -> Self::To {
        Self::new(value).expect("Value should not be NaN")
    }
}

impl<T> crate::factory::Cvt<T> for OrderedFloat<T>
where
    T: FloatCore,
{
    type To = Self;

    fn convert_to(value: T) -> Self::To {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{Factory, IFactory};
    use crate::ops::Intersection;
    use crate::{Bound, Bounding, Interval};

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
        type A = IFactory<f32, ordered_float::NotNan<f32>>;
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

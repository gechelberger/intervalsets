use ordered_float::{FloatCore, NotNan, OrderedFloat};

use crate::numeric::{Domain, LibZero};

impl LibZero for NotNan<f32> {
    fn new_zero() -> Self {
        // SAFETY: const 0.0 should never fail
        unsafe { Self::new_unchecked(0.0) }
    }
}

impl LibZero for NotNan<f64> {
    fn new_zero() -> Self {
        // SAFETY const 0.0 should never fail
        unsafe { Self::new_unchecked(0.0) }
    }
}

impl LibZero for OrderedFloat<f32> {
    fn new_zero() -> Self {
        Self(0.0)
    }
}

impl LibZero for OrderedFloat<f64> {
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

#[cfg(test)]
mod tests {
    use super::*;
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
}

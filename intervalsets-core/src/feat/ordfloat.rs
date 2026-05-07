use num_traits::float::FloatCore;
use ordered_float::{NotNan, OrderedFloat};

use crate::error::MidpointError;
use crate::factory::Converter;
use crate::numeric::{Element, Midpoint};

impl<T: FloatCore + Element> Element for NotNan<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }
}

impl<T: FloatCore + Element> Element for OrderedFloat<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }
}

impl<T: FloatCore + Element> Converter<T> for NotNan<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        NotNan::new(value).unwrap()
    }
}

impl<T: FloatCore + Element> Converter<T> for OrderedFloat<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        OrderedFloat(value)
    }
}

impl<T: FloatCore + Midpoint<Error = MidpointError>> Midpoint for NotNan<T> {
    type Error = MidpointError;

    /// Delegates to the inner `T::midpoint`. Returns
    /// [`MidpointError`](crate::error::MidpointError) when either
    /// input is non-finite (per the inner float impl's contract);
    /// the resulting midpoint is guaranteed finite, so re-wrapping
    /// in `NotNan` cannot fail.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        let mid = self.into_inner().midpoint(other.into_inner())?;
        Ok(NotNan::new(mid).expect("midpoint of finite floats is non-NaN"))
    }
}

impl<T: Midpoint<Error = MidpointError>> Midpoint for OrderedFloat<T> {
    type Error = MidpointError;

    /// Delegates to the inner `T::midpoint`. Returns
    /// [`MidpointError`](crate::error::MidpointError) when either
    /// input is non-finite — even though `OrderedFloat` admits NaN
    /// under its total order, NaN has no well-defined midpoint.
    fn midpoint(self, other: Self) -> Result<Self, Self::Error> {
        Ok(OrderedFloat(self.0.midpoint(other.0)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{EIFactory, FiniteFactory};
    use crate::EnumInterval;

    #[test]
    fn test_not_nan_converter() {
        type F = EIFactory<f32, NotNan<f32>>;

        let x = F::closed(0.0, 10.0);

        assert_eq!(
            x,
            EnumInterval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())
        );
    }

    #[test]
    fn test_ord_float_converter() {
        type F = EIFactory<f32, OrderedFloat<f32>>;

        let x = F::closed(0.0, 10.0);
        assert_eq!(
            x,
            EnumInterval::closed(OrderedFloat(0.0), OrderedFloat(10.0),)
        );
    }

    #[test]
    fn test_midpoint_not_nan() {
        let a = NotNan::new(2.0_f32).unwrap();
        let b = NotNan::new(4.0_f32).unwrap();
        assert_eq!(a.midpoint(b).unwrap(), NotNan::new(3.0_f32).unwrap());

        // Infinity is admitted by NotNan but rejected as a midpoint
        // endpoint -- inf.midpoint(-inf) would otherwise produce NaN.
        let inf = NotNan::new(f32::INFINITY).unwrap();
        let zero = NotNan::new(0.0_f32).unwrap();
        assert!(inf.midpoint(zero).is_err());
    }

    #[test]
    fn test_midpoint_ord_float() {
        let a = OrderedFloat(2.0_f32);
        let b = OrderedFloat(4.0_f32);
        assert_eq!(a.midpoint(b).unwrap(), OrderedFloat(3.0_f32));

        // NaN is admitted by OrderedFloat's total order but rejected
        // here -- NaN has no well-defined midpoint.
        let nan = OrderedFloat(f32::NAN);
        let zero = OrderedFloat(0.0_f32);
        assert!(nan.midpoint(zero).is_err());

        // Infinity is rejected for the same reason as the float impl.
        let inf = OrderedFloat(f32::INFINITY);
        assert!(inf.midpoint(zero).is_err());
    }
}

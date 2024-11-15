use ordered_float::{NotNan, OrderedFloat};

use crate::continuous_domain_impl;
use crate::factory::Converter;

continuous_domain_impl!(NotNan<f32>, NotNan<f64>);
continuous_domain_impl!(OrderedFloat<f32>, OrderedFloat<f64>);

impl<T: num_traits::float::FloatCore> Converter<T> for NotNan<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        NotNan::new(value).unwrap()
    }
}

impl<T: num_traits::float::FloatCore> Converter<T> for OrderedFloat<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        OrderedFloat(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::IFactory;
    use crate::{EnumInterval, Factory};

    #[test]
    fn test_not_nan_converter() {
        type F = IFactory<f32, NotNan<f32>>;

        let x = F::closed(0.0, 10.0);

        assert_eq!(
            x,
            EnumInterval::closed(NotNan::new(0.0).unwrap(), NotNan::new(10.0).unwrap())
        );
    }

    #[test]
    fn test_ord_float_converter() {
        type F = IFactory<f32, OrderedFloat<f32>>;

        let x = F::closed(0.0, 10.0);
        assert_eq!(
            x,
            EnumInterval::closed(OrderedFloat(0.0), OrderedFloat(10.0),)
        );
    }
}

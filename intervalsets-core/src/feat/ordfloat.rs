use num_traits::float::FloatCore;
use ordered_float::{NotNan, OrderedFloat};

use crate::factory::Converter;
use crate::numeric::Domain;

impl<T: FloatCore + Domain> Domain for NotNan<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }
}

impl<T: FloatCore + Domain> Domain for OrderedFloat<T> {
    fn try_adjacent(&self, _: crate::bound::Side) -> Option<Self> {
        None
    }
}

impl<T: FloatCore + Domain> Converter<T> for NotNan<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        NotNan::new(value).unwrap()
    }
}

impl<T: FloatCore + Domain> Converter<T> for OrderedFloat<T> {
    type To = Self;
    fn convert(value: T) -> Self::To {
        OrderedFloat(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::EIFactory;
    use crate::{EnumInterval, Factory};

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
}

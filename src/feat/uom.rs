//! Delegate traits to the datatype that[`uom`] is wrapping.

use num_traits::Num;
use uom::si::Quantity;
use uom::Conversion;

use crate::numeric::{Domain, LibZero};

impl<D, U, V> Domain for Quantity<D, U, V>
where
    D: uom::si::Dimension + ?Sized,
    U: uom::si::Units<V> + ?Sized,
    V: Domain + Num + Conversion<V>,
{
    fn try_adjacent(&self, side: crate::Side) -> Option<Quantity<D, U, V>> {
        V::try_adjacent(&self.value, side).map(|value| Quantity::<D, U, V> {
            dimension: self.dimension,
            units: self.units,
            value,
        })
    }
}

impl<D, U, V> LibZero for Quantity<D, U, V>
where
    D: uom::si::Dimension + ?Sized,
    U: uom::si::Units<V> + ?Sized,
    V: Domain + Num + Conversion<V> + LibZero,
{
    fn new_zero() -> Self {
        Self {
            dimension: core::marker::PhantomData,
            units: core::marker::PhantomData,
            value: V::new_zero(),
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::f32::*;
    use uom::si::length::kilometer;

    use crate::prelude::*;

    #[test]
    fn test_uom_width() {
        let a = Length::new::<kilometer>(10.0);
        let b = Length::new::<kilometer>(100.0);

        let interval = Interval::open(a, b);

        assert!(!interval.contains(&a));
        assert!(interval.contains(&Length::new::<kilometer>(50.0)));

        assert_eq!(interval.width().finite(), Length::new::<kilometer>(90.0));
    }
}

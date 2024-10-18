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

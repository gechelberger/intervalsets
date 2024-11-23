//! Delegate traits to the datatype that[`uom`] is wrapping.

use num_traits::Num;
use uom::si::Quantity;
use uom::Conversion;

use crate::numeric::Element;

impl<D, U, V> Element for Quantity<D, U, V>
where
    D: uom::si::Dimension + ?Sized,
    U: uom::si::Units<V> + ?Sized,
    V: Element + Num + Conversion<V>,
{
    fn try_adjacent(&self, side: crate::Side) -> Option<Quantity<D, U, V>> {
        V::try_adjacent(&self.value, side).map(|value| Quantity::<D, U, V> {
            dimension: self.dimension,
            units: self.units,
            value,
        })
    }
}

/*
impl<D, U, V> Zero for Quantity<D, U, V>
where
    D: uom::si::Dimension + ?Sized,
    U: uom::si::Units<V> + ?Sized,
    V: Element + Num + Conversion<V> + Zero,
{
    fn zero() -> Self {
        Self {
            dimension: core::marker::PhantomData,
            units: core::marker::PhantomData,
            value: V::zero(),
        }
    }
}*/

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

    #[test]
    fn test_uom_diff() {
        let a = Length::new::<kilometer>(0.0);
        let b = Length::new::<kilometer>(10.0);
        let c = Length::new::<kilometer>(20.0);
        let d = Length::new::<kilometer>(30.0);

        let ac = Interval::closed(a, c);
        let bd = Interval::closed(b, d);

        assert_eq!(ac.clone().intersection(bd.clone()), Interval::closed(b, c));
        assert_eq!(ac.union(bd).expect_interval(), Interval::closed(a, d));
    }
}

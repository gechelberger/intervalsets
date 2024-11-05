#[cfg(feature = "uom")]
mod uom_tests {

    use intervalsets::prelude::*;
    use uom::si::f32::*;
    use uom::si::length::{kilometer, meter};
    use uom::si::time::second;

    #[test]
    fn test_uom() {
        let length = Length::new::<kilometer>(5.0);
        let time = Time::new::<second>(10.0);
        let velocity = length / time;

        assert_eq!(velocity.get::<uom::si::velocity::meter_per_second>(), 500.0);

        let a = Length::new::<kilometer>(10.0);
        let b = Length::new::<kilometer>(20.0);

        let interval = Interval::open(a, b);
        assert_eq!(interval.contains(&Length::new::<kilometer>(15.0)), true);

        assert_eq!(interval.width().finite(), Length::new::<kilometer>(10.0));
    }
}

pub use intervalsets_core::ops::Intersects;

use crate::numeric::Element;
use crate::{Interval, IntervalSet};

impl<T: PartialOrd> Intersects<&Self> for Interval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.0.intersects(&rhs.0)
    }
}

impl<T: PartialOrd> Intersects<&Interval<T>> for IntervalSet<T> {
    fn intersects(&self, rhs: &Interval<T>) -> bool {
        self.iter().any(|subset| subset.intersects(rhs))
    }
}

impl<T: PartialOrd> Intersects<&IntervalSet<T>> for Interval<T> {
    fn intersects(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.intersects(self)
    }
}

impl<T: Element> Intersects<&Self> for IntervalSet<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.iter().any(|subset| rhs.intersects(subset))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_finite_intersects() {
        assert!(Interval::open(0, 10).intersects(&Interval::open(5, 15)));

        assert!(!Interval::open(0, 10).intersects(&Interval::closed(10, 20)));
    }

    #[test]
    fn test_set_set_intersects() {
        let a = IntervalSet::new(vec![
            Interval::unbound_open(0.0),
            Interval::closed(100.0, 110.0),
            Interval::open(1000.0, 1100.0),
        ]);
        let b = IntervalSet::new(vec![
            Interval::open(10.0, 20.0),     // no
            Interval::closed(110.0, 120.0), // [110.0, 110.0]
        ]);

        assert!(a.intersects(&b));
    }
}

pub use intervalsets_core::ops::Contains;

use crate::numeric::Element;
use crate::{Interval, IntervalSet};

impl<T: Element> Contains<&T> for Interval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.0.contains(rhs)
    }
}

impl<T: Element> Contains<&Interval<T>> for Interval<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        self.0.contains(&rhs.0)
    }
}

impl<T: Element> Contains<&IntervalSet<T>> for Interval<T> {
    fn contains(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

impl<T: Element> Contains<&T> for IntervalSet<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: Element> Contains<&Interval<T>> for IntervalSet<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        self.iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: Element> Contains<&IntervalSet<T>> for IntervalSet<T> {
    fn contains(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.iter().all(|subset| self.contains(subset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[quickcheck]
    fn check_empty_contains_integer(x: i8) {
        let interval = Interval::<i8>::empty();
        assert!(!interval.contains(&x))
    }

    #[quickcheck]
    fn check_empty_contains_float(x: f32) {
        let interval = Interval::<f32>::empty();
        assert!(!interval.contains(&x))
    }

    #[quickcheck]
    fn check_finite_contains_integer(x: i8) {
        let iv = Interval::open(-100, 100);
        assert_eq!(iv.contains(&x), -100 < x && x < 100);
    }

    #[quickcheck]
    fn check_finite_contains_float(x: f32) {
        let iv = Interval::closed(-100.0, 100.0);
        assert_eq!(iv.contains(&x), -100.0 < x && x < 100.0);
    }

    #[quickcheck]
    fn check_half_contains_integer(x: i8) {
        let left = Interval::unbound_closed(0);
        assert_eq!(left.contains(&x), x <= 0);

        let right = Interval::closed_unbound(0);
        assert_eq!(right.contains(&x), x >= 0);
    }

    #[quickcheck]
    fn check_half_contains_float(x: f32) {
        let left = Interval::unbound_closed(0.0);
        assert_eq!(left.contains(&x), x <= 0.0);

        let right = Interval::closed_unbound(0.0);
        assert_eq!(right.contains(&x), x >= 0.0);
    }

    #[quickcheck]
    fn check_unbounded_contains_float(x: f32) {
        let iv = Interval::unbounded();
        assert_eq!(iv.contains(&x), !x.is_nan());
    }

    #[quickcheck]
    fn check_finite_contains_finite_integer(a: i8, b: i8) {
        // Skip crossed pairs: factory is strict-by-default and would panic.
        if a > b {
            return;
        }
        let interval = Interval::closed(-50, 50);
        let candidate = Interval::closed(a, b);

        assert_eq!(interval.contains(&candidate), -50 <= a && b <= 50);
    }

    #[quickcheck]
    fn check_finite_contains_finite_float(a: f32, b: f32) {
        if !a.is_finite() || !b.is_finite() {
            return;
        }
        // Skip crossed pairs: factory is strict-by-default and would panic.
        if a >= b {
            return;
        }

        let interval = Interval::open(-100.0, 100.0);
        let candidate = Interval::open(a, b);

        assert_eq!(interval.contains(&candidate), -100.0 < a && b < 100.0)
    }

    #[quickcheck]
    fn check_finite_contains_unbounded_integer(x: i8) {
        let interval = Interval::closed(-100, 100);

        assert!(!interval.contains(&Interval::unbound_closed(x)));
        assert!(!interval.contains(&Interval::unbound_open(x)));
        assert!(!interval.contains(&Interval::open_unbound(x)));
        assert!(!interval.contains(&Interval::closed_unbound(x)));
        assert!(!interval.contains(&Interval::unbounded()));
    }

    #[quickcheck]
    fn check_half_contains_finite_integer(a: i8, b: i8) {
        // Skip crossed pairs: factory is strict-by-default and would panic.
        if a > b {
            return;
        }
        let interval = Interval::open_unbound(0);

        let finite = Interval::closed(a, b);
        assert_eq!(interval.contains(&finite), 0 < a);
    }

    #[quickcheck]
    fn check_unbounded_contains_finite_integer(a: i8, b: i8) {
        // Skip crossed pairs: factory is strict-by-default and would panic.
        if a > b {
            return;
        }
        let interval = Interval::<i8>::unbounded();

        let finite = Interval::closed(a, b);
        assert!(interval.contains(&finite));
    }

    #[test]
    fn test_iset_contains_iset() {
        let superset =
            IntervalSet::from_iter([Interval::closed(0, 100), Interval::closed(200, 300)]);

        let subset = IntervalSet::from_iter([Interval::closed(40, 60), Interval::closed(240, 260)]);

        assert!(superset.contains(&subset));
        assert!(!subset.contains(&superset));

        assert!(superset.contains(&superset));
        assert!(subset.contains(&subset));
    }
}

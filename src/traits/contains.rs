use crate::sets::IntervalSet;
use crate::{Domain, Interval};
/// Defines whether a set fully contains another.
///
/// For our purposes a point is the singleton set [T].
///
/// A contains B if and only if
/// for every element x of B,
/// x is also an element of A.
///
/// Contains is not commutative.
///
/// # Example
/// ```
/// use intervalsets::Interval;
/// use intervalsets::Contains;
///
/// let x = Interval::open(0, 10);
/// assert_eq!(x.contains(&5), true);
/// assert_eq!(x.contains(&10), false);
/// assert_eq!(x.contains(&Interval::open(0, 10)), true);
/// ```
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T: Domain> Contains<T> for Interval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.0.contains(rhs)
    }
}

impl<T: Domain> Contains<Self> for Interval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.0.contains(&rhs.0)
    }
}

impl<T: Domain> Contains<IntervalSet<T>> for Interval<T> {
    fn contains(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.intervals().iter().all(|subset| self.contains(subset))
    }
}

impl<T: Domain> Contains<T> for IntervalSet<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.intervals().iter().any(|subset| subset.contains(rhs))
    }
}

impl<T: Domain> Contains<Interval<T>> for IntervalSet<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        todo!()
    }
}

impl<T: Domain> Contains<Self> for IntervalSet<T> {
    fn contains(&self, rhs: &Self) -> bool {
        todo!()
    }
}

    
#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn check_empty_contains_integer(x: i8) {
        let interval = Interval::<i8>::empty();
        assert_eq!(interval.contains(&x), false)
    }

    #[quickcheck]
    fn check_empty_contains_float(x: f32) {
        let interval = Interval::<f32>::empty();
        assert_eq!(interval.contains(&x), false)
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
        assert!(iv.contains(&x));
    }

    #[quickcheck]
    fn check_finite_contains_finite_integer(a: i8, b: i8) {
        let interval = Interval::closed(-50, 50);
        let candidate = Interval::closed(a, b);

        assert_eq!(interval.contains(&candidate), a <= b && -50 <= a && b <= 50)
    }

    #[quickcheck]
    fn check_finite_contains_finite_float(a: f32, b: f32) {
        let interval = Interval::open(-100.0, 100.0);
        let candidate = Interval::open(a, b);

        assert_eq!(
            interval.contains(&candidate),
            a < b && -100.0 < a && b < 100.0
        )
    }

    #[quickcheck]
    fn check_finite_contains_unbounded_integer(x: i8) {
        let interval = Interval::closed(-100, 100);

        assert_eq!(interval.contains(&Interval::unbound_closed(x)), false);
        assert_eq!(interval.contains(&Interval::unbound_open(x)), false);
        assert_eq!(interval.contains(&Interval::open_unbound(x)), false);
        assert_eq!(interval.contains(&Interval::closed_unbound(x)), false);
        assert_eq!(interval.contains(&Interval::unbounded()), false);
    }

    #[quickcheck]
    fn check_half_contains_finite_integer(a: i8, b: i8) {
        let interval = Interval::open_unbound(0);

        let finite = Interval::closed(a, b);
        assert_eq!(interval.contains(&finite), 0 < a && a <= b);
    }

    #[quickcheck]
    fn check_unbounded_contains_finite_integer(a: i8, b: i8) {
        let interval = Interval::<i8>::unbounded();

        let finite = Interval::closed(a, b);
        assert_eq!(interval.contains(&finite), a <= b);
    }

}
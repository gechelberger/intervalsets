pub use intervalsets_core::ops::IntoFiniteInterval;

use crate::bound::ord::OrdBoundPair;
use crate::numeric::Element;
use crate::{Interval, IntervalSet};

impl<T: Element + num_traits::Bounded> IntoFiniteInterval for Interval<T> {
    type Output = Self;

    #[inline]
    fn into_finite_interval(self) -> Self::Output {
        Self::from(self.0.into_finite_interval())
    }
}

impl<T: Element + num_traits::Bounded> IntoFiniteInterval for IntervalSet<T> {
    type Output = Interval<T>;

    fn into_finite_interval(self) -> Self::Output {
        // Consume the set into its hull (first/last bound pair), then truncate.
        let pair = OrdBoundPair::from(self);
        let hull = Interval::try_from(pair).expect("IntervalSet invariants violated");
        hull.into_finite_interval()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_interval_into_finite_interval() {
        assert_eq!(
            Interval::closed(0, 10).into_finite_interval(),
            Interval::closed(0, 10)
        );

        assert_eq!(
            Interval::<u8>::unbounded().into_finite_interval(),
            Interval::closed(0, 255)
        );

        assert_eq!(
            Interval::<f32>::open_unbound(0.0).into_finite_interval(),
            Interval::open_closed(0.0, f32::MAX)
        );

        assert_eq!(
            Interval::<f32>::unbound_open(0.0).into_finite_interval(),
            Interval::closed_open(f32::MIN, 0.0)
        );
    }

    #[test]
    fn test_interval_set_into_finite_interval() {
        let x = IntervalSet::<i8>::unbounded();
        assert_eq!(x.into_finite_interval(), Interval::closed(-128, 127));

        // Hull of the disjoint set spans (-inf, +inf), so the finite
        // truncation collapses to the closed type-extent interval.
        let x = IntervalSet::<i8>::new([
            (..-100).into(),
            [-50, 0].into(),
            [50, 90].into(),
            (100..).into(),
        ]);
        assert_eq!(x.into_finite_interval(), Interval::closed(-128, 127));

        // Bounded disjoint set: hull is the outer envelope, truncation is the same.
        let x = IntervalSet::<i8>::new([[-50, 0].into(), [50, 90].into()]);
        assert_eq!(x.into_finite_interval(), Interval::closed(-50, 90));

        // Empty set survives the round-trip.
        let x = IntervalSet::<i8>::empty();
        assert_eq!(x.into_finite_interval(), Interval::empty());
    }
}

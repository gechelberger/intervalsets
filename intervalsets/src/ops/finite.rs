pub use intervalsets_core::ops::IntoFinite;
use intervalsets_core::MaybeEmpty;

use crate::numeric::Domain;
use crate::{Interval, IntervalSet};

impl<T: num_traits::Bounded + PartialOrd> IntoFinite for Interval<T> {
    type Output = Self;

    #[inline]
    fn into_finite(self) -> Self::Output {
        Self::from(self.0.into_finite())
    }
}

impl<T: num_traits::Bounded + Domain> IntoFinite for IntervalSet<T> {
    type Output = Self;

    fn into_finite(self) -> Self::Output {
        //unsafe {
        Self::new_unchecked(
            self.into_iter()
                .map(IntoFinite::into_finite)
                .filter(MaybeEmpty::is_inhabited),
        )
        //}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_interval_into_finite() {
        assert_eq!(
            Interval::closed(0, 10).into_finite(),
            Interval::closed(0, 10)
        );

        assert_eq!(
            Interval::<u8>::unbounded().into_finite(),
            Interval::closed(0, 255)
        );

        assert_eq!(
            Interval::<f32>::open_unbound(0.0).into_finite(),
            Interval::open_closed(0.0, f32::MAX)
        );

        assert_eq!(
            Interval::<f32>::unbound_open(0.0).into_finite(),
            Interval::closed_open(f32::MIN, 0.0)
        );
    }

    #[test]
    fn test_interval_set_into_finite() {
        let x = IntervalSet::<i8>::unbounded();
        assert_eq!(x.into_finite(), IntervalSet::closed(-128, 127));

        let x = IntervalSet::<i8>::new([
            (..-100).into(),
            [-50, 0].into(),
            [50, 90].into(),
            (100..).into(),
        ]);

        assert_eq!(
            x.into_finite(),
            IntervalSet::<i8>::new([
                (-128..-100).into(),
                [-50, 0].into(),
                [50, 90].into(),
                (100..=127).into()
            ])
        );
    }
}

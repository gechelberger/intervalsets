use intervalsets_core::ops::math::TryAdd;

use super::{Count, CountOverflowError, Countable, Measurement};
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Count for Interval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = CountOverflowError;

    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        self.0.try_count()
    }
}

impl<T, Out> Count for IntervalSet<T>
where
    T: Countable<Output = Out>,
    Out: Zero + TryAdd<Out, Output = Out>,
    <Out as TryAdd>::Error: Into<CountOverflowError>,
{
    type Output = Out;
    type Error = CountOverflowError;

    /// Sum per-component counts via [`TryAdd`] so a summation that
    /// exceeds `Out`'s representable range surfaces as
    /// `CountOverflowError` rather than panicking in debug / wrapping
    /// in release.
    fn try_count(&self) -> Result<Measurement<Self::Output>, Self::Error> {
        self.iter()
            .try_fold(Measurement::Finite(Out::zero()), |accum, subset| {
                accum.try_binop_map(subset.try_count()?, |a, b| a.try_add(b).map_err(Into::into))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn intervalset_count_disjoint() {
        let s = IntervalSet::new(vec![Interval::closed(0_i32, 1), Interval::closed(5, 6)]);
        assert_eq!(s.try_count().unwrap().finite(), 4_u128);
    }

    #[test]
    fn intervalset_count_summation_overflow_returns_err() {
        // Each [0, i128::MAX] contains i128::MAX + 1 = 2^127 elements.
        // Two of them sum to 2^128, which overflows u128 by exactly 1.
        let s = IntervalSet::new(vec![
            Interval::closed(0_i128, i128::MAX),
            Interval::closed(i128::MIN, -1_i128),
        ]);
        assert!(s.try_count().is_err());
    }
}

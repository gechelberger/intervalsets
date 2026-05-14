use intervalsets_core::ops::math::TryAdd;

use super::{Cardinality, Countable, Extent};
use crate::error::MathError;
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Cardinality for Interval<T>
where
    T: Countable,
    T::Output: Zero,
{
    type Output = T::Output;
    type Error = MathError;

    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.0.try_cardinality()
    }
}

impl<T, Out> Cardinality for IntervalSet<T>
where
    T: Countable<Output = Out>,
    Out: Zero + TryAdd<Out, Output = Out>,
    <Out as TryAdd>::Error: Into<MathError>,
{
    type Output = Out;
    type Error = MathError;

    /// Sum per-component cardinalities via [`TryAdd`] so a summation that
    /// exceeds `Out`'s representable range surfaces as [`MathError`]
    /// rather than panicking in debug / wrapping in release.
    fn try_cardinality(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.iter()
            .try_fold(Extent::Finite(Out::zero()), |accum, subset| {
                accum.try_binop_map(subset.try_cardinality()?, |a, b| {
                    a.try_add(b).map_err(Into::into)
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn intervalset_cardinality_disjoint() {
        let s = IntervalSet::new(vec![Interval::closed(0_i32, 1), Interval::closed(5, 6)]);
        assert_eq!(s.try_cardinality().unwrap().finite(), 4_u128);
    }

    #[test]
    fn intervalset_cardinality_summation_overflow_returns_err() {
        // Each [0, i128::MAX] contains i128::MAX + 1 = 2^127 elements.
        // Two of them sum to 2^128, which overflows u128 by exactly 1.
        let s = IntervalSet::new(vec![
            Interval::closed(0_i128, i128::MAX),
            Interval::closed(i128::MIN, -1_i128),
        ]);
        assert!(s.try_cardinality().is_err());
    }

    #[test]
    fn intervalset_continuous_singletons_sum() {
        let s = IntervalSet::new(vec![
            Interval::closed(0.0_f64, 0.0),
            Interval::closed(1.0, 1.0),
        ]);
        assert_eq!(s.cardinality().finite(), 2_u128);
    }

    #[test]
    fn intervalset_continuous_with_nondegenerate_is_infinite() {
        let s = IntervalSet::new(vec![
            Interval::closed(0.0_f64, 0.0),
            Interval::closed(2.0, 3.0),
        ]);
        assert!(s.try_cardinality().unwrap().is_infinite());
    }
}

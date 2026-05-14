use intervalsets_core::measure::{Extent, Measure};
use intervalsets_core::numeric::Element;
use intervalsets_core::ops::math::TryAdd;

use crate::error::MathError;
use crate::numeric::Zero;
use crate::{Interval, IntervalSet};

impl<T> Measure for Interval<T>
where
    T: Element,
{
    type Output = T::Measure;
    type Error = MathError;

    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.0.try_measure()
    }
}

impl<T> Measure for IntervalSet<T>
where
    T: Element,
    <T::Measure as TryAdd>::Error: Into<MathError>,
{
    type Output = T::Measure;
    type Error = MathError;

    /// Sum per-component measures via [`TryAdd`] so a summation that
    /// exceeds `T::Measure`'s representable range surfaces as
    /// [`MathError`] rather than panicking in debug / wrapping in
    /// release. `Infinite` from any piece propagates.
    fn try_measure(&self) -> Result<Extent<Self::Output>, Self::Error> {
        self.iter().try_fold(
            Extent::Finite(<T::Measure as Zero>::zero()),
            |accum, subset| {
                accum.try_binop_map(subset.try_measure()?, |a, b| {
                    a.try_add(b).map_err(Into::into)
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::*;
    use crate::factory::FiniteFactory;
    use crate::ops::Intersects;

    // ===== Discrete: cardinality semantics =====

    #[test]
    fn intervalset_discrete_disjoint_sums_cardinalities() {
        // i32 → u64 under stepwise widening.
        // [0,1] (2 elements) ∪ [5,6] (2 elements) → 4
        let s = IntervalSet::new(vec![Interval::closed(0_i32, 1), Interval::closed(5, 6)]);
        assert_eq!(s.try_measure().unwrap().finite(), 4_u64);
    }

    #[test]
    fn intervalset_discrete_summation_overflow_returns_err() {
        // Each i128 piece's cardinality already overflows u128 at the limit;
        // two large pieces likewise.
        let s = IntervalSet::new(vec![
            Interval::closed(0_i128, i128::MAX),
            Interval::closed(i128::MIN, -1_i128),
        ]);
        assert!(s.try_measure().is_err());
    }

    // ===== Continuous: Lebesgue width semantics =====

    #[test]
    fn intervalset_continuous_singletons_sum_to_zero() {
        // Two singletons each have Lebesgue width 0 on continuous T.
        let s = IntervalSet::new(vec![
            Interval::closed(0.0_f64, 0.0),
            Interval::closed(1.0, 1.0),
        ]);
        assert_eq!(s.measure().finite(), 0.0);
    }

    #[test]
    fn intervalset_continuous_with_nondegenerate_is_finite_width() {
        // [0,0] (width 0) ∪ [2,3] (width 1) → 1.0
        let s = IntervalSet::new(vec![
            Interval::closed(0.0_f64, 0.0),
            Interval::closed(2.0, 3.0),
        ]);
        assert_eq!(s.measure().finite(), 1.0);
    }

    // ===== Quickcheck: continuous floats =====

    #[quickcheck]
    fn check_finite_measure_float(a: f32, b: f32) {
        if f32::is_nan(a) || f32::is_nan(b) || f32::is_infinite(a) || f32::is_infinite(b) {
            return;
        }
        if a >= b {
            return;
        }

        let expected = b - a;
        if !expected.is_finite() {
            return;
        }
        let open_interval = Interval::open(a, b);
        let closed_interval = Interval::closed(a, b);

        assert_eq!(open_interval.measure().finite(), expected);
        assert_eq!(closed_interval.measure().finite(), expected);
    }

    #[quickcheck]
    fn check_set_measure_float(a: f32, b: f32, c: f32, d: f32) -> bool {
        if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() {
            return true;
        }
        if a >= b || c >= d {
            return true;
        }

        let ab = Interval::open(a, b);
        let cd = Interval::open(c, d);

        let expected = (b - a) + (d - c);
        if !expected.is_finite() {
            return true;
        }
        let x = IntervalSet::new(vec![ab, cd]);

        if ab.intersects(&cd) {
            x.measure().finite() <= expected
        } else {
            relative_eq!(x.measure().finite(), expected)
        }
    }

    // ===== Quickcheck: discrete integers (cardinality semantics) =====

    #[quickcheck]
    fn check_set_measure_integer(a: i32, b: i32, c: i32, d: i32) -> bool {
        if a > b || c > d {
            return true;
        }

        // i32 cardinality = b - a + 1; widened to u64.
        let ab_card = (b as i64 - a as i64 + 1) as u64;
        let ab_ivl = Interval::closed(a, b);

        let cd_card = (d as i64 - c as i64 + 1) as u64;
        let cd_ivl = Interval::closed(c, d);

        let expected = ab_card + cd_card;
        let x = IntervalSet::new(vec![ab_ivl, cd_ivl]);

        // Subadditivity: measure of union ≤ sum of per-piece measures.
        if ab_ivl.intersects(&cd_ivl) {
            x.measure().finite() <= expected
        } else {
            x.measure().finite() == expected
        }
    }
}

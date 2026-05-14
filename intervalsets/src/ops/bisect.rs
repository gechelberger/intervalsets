//! [`Bisect`] impls for the outer crate's [`Interval`] and
//! [`IntervalSet`]. The iteration algorithm lives in
//! [`intervalsets_core::ops::bisect_core`]; the single-interval path
//! splits at the bound midpoint in one step.

use core::convert::Infallible;

use intervalsets_core::error::MathError;
use intervalsets_core::ops::math::TryAdd;
use intervalsets_core::ops::{bisect_core, Split};
pub use intervalsets_core::ops::{Bisect, Bisection};

use crate::bound::{SetBounds, Side};
use crate::numeric::{Element, Midpointable};
use crate::{Interval, IntervalSet};

impl<T> Bisect<T> for Interval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>> {
        // A connected interval — split at the bound midpoint in one
        // step. Uniform measure density across the range means the
        // midpoint is already the balance point (off by ±1 for
        // discrete T due to integer rounding).
        let lo = self.lval()?.clone();
        let hi = self.rval()?.clone();
        let midpoint = T::midpoint(lo, hi).unwrap();
        let (left, right) = self.clone().split(midpoint.clone(), closed);
        Some(Bisection {
            midpoint,
            left,
            right,
            closed,
        })
    }
}

impl<T> Bisect<T> for IntervalSet<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
    T::Measure: PartialOrd,
    <T::Measure as TryAdd>::Error: Into<MathError>,
{
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>> {
        let hull = self.hull();
        let lo = hull.lval()?.clone();
        let hi = hull.rval()?.clone();
        let (midpoint, left, right) = bisect_core(lo, hi, |m| self.clone().split(m, closed));
        Some(Bisection {
            midpoint,
            left,
            right,
            closed,
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;
    use quickcheck::TestResult;

    use super::*;
    use crate::factory::traits::*;
    use crate::measure::{Extent, Measure};
    use crate::ops::Union;

    #[test]
    fn interval_finite_bisects() {
        let iv = Interval::closed(0_i64, 100);
        let b = iv.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn interval_empty_returns_none() {
        let iv = Interval::<i64>::empty();
        assert!(iv.bisect(Side::Left).is_none());
    }

    #[test]
    fn interval_unbounded_returns_none() {
        let iv = Interval::<i64>::unbounded();
        assert!(iv.bisect(Side::Left).is_none());
    }

    #[test]
    fn interval_half_bounded_returns_none() {
        let iv = Interval::closed_unbound(0_i64);
        assert!(iv.bisect(Side::Left).is_none());
    }

    #[test]
    fn interval_set_single_piece_bisects() {
        let set: IntervalSet<i64> = Interval::closed(0, 100).into();
        let b = set.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn interval_set_two_pieces_bisects() {
        // [0, 10] U [90, 100]: total cardinality 22 (11 each), half is 11.
        // Equal pieces, so the geometric midpoint falls in the gap and balances.
        let set = Interval::closed(0_i64, 10).union(Interval::closed(90, 100));
        let b = set.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_eq!(lw, 11);
        assert_eq!(rw, 11);
        assert!(b.midpoint > 10 && b.midpoint < 90);
    }

    #[test]
    fn interval_set_empty_returns_none() {
        let set = IntervalSet::<i64>::empty();
        assert!(set.bisect(Side::Left).is_none());
    }

    #[test]
    fn interval_set_unbounded_piece_returns_none() {
        // A set containing any half-bounded or unbounded piece has a
        // non-finite hull — bisection isn't defined.
        let set: IntervalSet<i64> = Interval::closed(0, 10).union(Interval::closed_unbound(100));
        assert!(set.bisect(Side::Left).is_none());
    }

    /// Bisects an arbitrary `IntervalSet` and verifies the balance
    /// property: `measure(left) ≈ measure(right) ≈ total / 2`.
    #[quickcheck]
    fn check_bisect_balances_measure(set: IntervalSet<f64>) -> TestResult {
        let total = match set.try_measure() {
            Ok(Extent::Finite(v)) if v.is_finite() && v > 0.0 => v,
            _ => return TestResult::discard(),
        };
        let half = total / 2.0;

        let bisection = match set.bisect(Side::Left) {
            Some(b) => b,
            None => return TestResult::discard(),
        };
        let lw = bisection.left.measure().finite();
        let rw = bisection.right.measure().finite();

        // Compare each half to total/2 individually — avoids `lw + rw`
        // overflowing to INF when the set's total measure approaches
        // f64::MAX. Both halves close to total/2 implies they're close
        // to each other.
        assert!(
            relative_eq!(lw, half, max_relative = 1e-9),
            "lw != total/2: lw={lw}, half={half}, set={set:?}",
        );
        assert!(
            relative_eq!(rw, half, max_relative = 1e-9),
            "rw != total/2: rw={rw}, half={half}, set={set:?}",
        );
        TestResult::passed()
    }

    #[test]
    fn interval_f64_smoke() {
        let iv = Interval::closed(0.0_f64, 10.0);
        let b = iv.bisect(Side::Left).expect("bounded");
        assert!((b.midpoint - 5.0).abs() < 1e-12);
    }

    #[test]
    fn interval_set_u32_smoke() {
        let set: IntervalSet<u32> = Interval::closed(0, 10).union(Interval::closed(90, 100));
        let b = set.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_eq!(lw, rw);
    }
}

//! Optional support for the [`approx`] crate.
//!
//! `Interval<T>` forwards to its inner [`EnumInterval`]. `IntervalSet<T>`
//! compares element-wise against the canonical-form vector — same length,
//! corresponding intervals approximately equal.
//!
//! # What this is not
//!
//! This is a *structural* comparison: same shape (variant / length /
//! side), with limit values approximately equal. It is not a test for
//! set-theoretic equivalence under tolerance. For example, the union
//! `(-∞, x) ∪ (x, ∞)` on a continuous domain differs from the unbounded
//! universe by a single point of measure zero and is arguably
//! "approximately" equivalent to it, but the structural comparison sees
//! a 2-element `IntervalSet` against a 1-element one and returns
//! `false`. Callers wanting set-equivalence semantics need to compose
//! their own predicate (e.g. via symmetric difference + measure).
//!
//! [`EnumInterval`]: intervalsets_core::sets::EnumInterval

use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::sets::{Interval, IntervalSet};

impl<T: AbsDiffEq> AbsDiffEq for Interval<T>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
    }
}

impl<T: RelativeEq> RelativeEq for Interval<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        self.0.relative_eq(&other.0, epsilon, max_relative)
    }
}

impl<T: UlpsEq> UlpsEq for Interval<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.0.ulps_eq(&other.0, epsilon, max_ulps)
    }
}

impl<T: AbsDiffEq> AbsDiffEq for IntervalSet<T>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        let a = self.slice();
        let b = other.slice();
        a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.abs_diff_eq(y, epsilon.clone()))
    }
}

impl<T: RelativeEq> RelativeEq for IntervalSet<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        let a = self.slice();
        let b = other.slice();
        a.len() == b.len()
            && a.iter().zip(b.iter()).all(|(x, y)| {
                x.relative_eq(y, epsilon.clone(), max_relative.clone())
            })
    }
}

impl<T: UlpsEq> UlpsEq for IntervalSet<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        let a = self.slice();
        let b = other.slice();
        a.len() == b.len()
            && a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.ulps_eq(y, epsilon.clone(), max_ulps))
    }
}

#[cfg(test)]
mod tests {
    use approx::{abs_diff_eq, relative_eq, ulps_eq};

    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn interval_forwards_to_inner() {
        let a = Interval::closed(1.0_f64, 2.0);
        let b = Interval::closed(1.0 + 1e-12, 2.0 - 1e-12);
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));
    }

    #[test]
    fn interval_set_pairwise() {
        let a = IntervalSet::new([
            Interval::closed(1.0_f64, 2.0),
            Interval::closed(5.0, 6.0),
        ]);
        let b = IntervalSet::new([
            Interval::closed(1.0 + 1e-12, 2.0),
            Interval::closed(5.0, 6.0 - 1e-12),
        ]);
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));
        assert!(ulps_eq!(a, b, max_ulps = u32::MAX));
    }

    #[test]
    fn interval_set_length_mismatch_never_equal() {
        let one = IntervalSet::new([Interval::closed(1.0_f64, 2.0)]);
        let two = IntervalSet::new([
            Interval::closed(1.0_f64, 2.0),
            Interval::closed(5.0, 6.0),
        ]);
        assert!(!abs_diff_eq!(one, two, epsilon = f64::INFINITY));
    }

    #[test]
    fn interval_set_empty_matches_empty() {
        let a: IntervalSet<f64> = IntervalSet::new([]);
        let b: IntervalSet<f64> = IntervalSet::new([]);
        assert!(abs_diff_eq!(a, b, epsilon = 0.0));
        assert!(relative_eq!(a, b, max_relative = 0.0));
        assert!(ulps_eq!(a, b, max_ulps = 0));
    }
}

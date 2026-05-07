//! Optional support for the [`approx`] crate.
//!
//! Compares the underlying limit value only. Bound type (open vs.
//! closed) is structural and not part of numeric closeness — combine
//! with [`FiniteBound::bound_type`] if you also need that to match.

use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use crate::bound::FiniteBound;
use crate::disjoint::MaybeDisjoint;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: AbsDiffEq> AbsDiffEq for FiniteBound<T> {
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.value().abs_diff_eq(other.value(), epsilon)
    }
}

impl<T: RelativeEq> RelativeEq for FiniteBound<T> {
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
        self.value()
            .relative_eq(other.value(), epsilon, max_relative)
    }
}

impl<T: UlpsEq> UlpsEq for FiniteBound<T> {
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.value().ulps_eq(other.value(), epsilon, max_ulps)
    }
}

impl<T: AbsDiffEq> AbsDiffEq for FiniteInterval<T>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self.view_raw(), other.view_raw()) {
            (None, None) => true,
            (Some((l1, r1)), Some((l2, r2))) => {
                l1.abs_diff_eq(l2, epsilon.clone()) && r1.abs_diff_eq(r2, epsilon)
            }
            _ => false,
        }
    }
}

impl<T: RelativeEq> RelativeEq for FiniteInterval<T>
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
        match (self.view_raw(), other.view_raw()) {
            (None, None) => true,
            (Some((l1, r1)), Some((l2, r2))) => {
                l1.relative_eq(l2, epsilon.clone(), max_relative.clone())
                    && r1.relative_eq(r2, epsilon, max_relative)
            }
            _ => false,
        }
    }
}

impl<T: UlpsEq> UlpsEq for FiniteInterval<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        match (self.view_raw(), other.view_raw()) {
            (None, None) => true,
            (Some((l1, r1)), Some((l2, r2))) => {
                l1.ulps_eq(l2, epsilon.clone(), max_ulps) && r1.ulps_eq(r2, epsilon, max_ulps)
            }
            _ => false,
        }
    }
}

impl<T: AbsDiffEq> AbsDiffEq for HalfInterval<T> {
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.side() == other.side()
            && self
                .finite_bound()
                .abs_diff_eq(other.finite_bound(), epsilon)
    }
}

impl<T: RelativeEq> RelativeEq for HalfInterval<T> {
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
        self.side() == other.side()
            && self
                .finite_bound()
                .relative_eq(other.finite_bound(), epsilon, max_relative)
    }
}

impl<T: UlpsEq> UlpsEq for HalfInterval<T> {
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.side() == other.side()
            && self
                .finite_bound()
                .ulps_eq(other.finite_bound(), epsilon, max_ulps)
    }
}

impl<T: AbsDiffEq> AbsDiffEq for EnumInterval<T>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => a.abs_diff_eq(b, epsilon),
            (Self::Half(a), Self::Half(b)) => a.abs_diff_eq(b, epsilon),
            (Self::Unbounded, Self::Unbounded) => true,
            _ => false,
        }
    }
}

impl<T: RelativeEq> RelativeEq for EnumInterval<T>
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
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => a.relative_eq(b, epsilon, max_relative),
            (Self::Half(a), Self::Half(b)) => a.relative_eq(b, epsilon, max_relative),
            (Self::Unbounded, Self::Unbounded) => true,
            _ => false,
        }
    }
}

impl<T: UlpsEq> UlpsEq for EnumInterval<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        match (self, other) {
            (Self::Finite(a), Self::Finite(b)) => a.ulps_eq(b, epsilon, max_ulps),
            (Self::Half(a), Self::Half(b)) => a.ulps_eq(b, epsilon, max_ulps),
            (Self::Unbounded, Self::Unbounded) => true,
            _ => false,
        }
    }
}

impl<T: AbsDiffEq> AbsDiffEq for MaybeDisjoint<T>
where
    T::Epsilon: Clone,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self, other) {
            (Self::Consumed, Self::Consumed) => true,
            (Self::Connected(a), Self::Connected(b)) => a.abs_diff_eq(b, epsilon),
            (Self::Disjoint(a1, a2), Self::Disjoint(b1, b2)) => {
                a1.abs_diff_eq(b1, epsilon.clone()) && a2.abs_diff_eq(b2, epsilon)
            }
            _ => false,
        }
    }
}

impl<T: RelativeEq> RelativeEq for MaybeDisjoint<T>
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
        match (self, other) {
            (Self::Consumed, Self::Consumed) => true,
            (Self::Connected(a), Self::Connected(b)) => a.relative_eq(b, epsilon, max_relative),
            (Self::Disjoint(a1, a2), Self::Disjoint(b1, b2)) => {
                a1.relative_eq(b1, epsilon.clone(), max_relative.clone())
                    && a2.relative_eq(b2, epsilon, max_relative)
            }
            _ => false,
        }
    }
}

impl<T: UlpsEq> UlpsEq for MaybeDisjoint<T>
where
    T::Epsilon: Clone,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        match (self, other) {
            (Self::Consumed, Self::Consumed) => true,
            (Self::Connected(a), Self::Connected(b)) => a.ulps_eq(b, epsilon, max_ulps),
            (Self::Disjoint(a1, a2), Self::Disjoint(b1, b2)) => {
                a1.ulps_eq(b1, epsilon.clone(), max_ulps) && a2.ulps_eq(b2, epsilon, max_ulps)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::{abs_diff_eq, relative_eq, ulps_eq};

    use super::*;

    #[test]
    fn finite_bound_abs_diff_eq() {
        let a = FiniteBound::closed(1.0_f64);
        let b = FiniteBound::closed(1.0 + 1e-12);
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
    }

    #[test]
    fn finite_bound_relative_eq() {
        let a = FiniteBound::open(1_000_000.0_f64);
        let b = FiniteBound::open(1_000_000.0 + 1.0);
        assert!(relative_eq!(a, b, max_relative = 1e-5));
        assert!(!relative_eq!(a, b, max_relative = 1e-9));
    }

    #[test]
    fn finite_bound_ulps_eq() {
        let a = FiniteBound::closed(1.0_f64);
        let b = FiniteBound::closed(f64::from_bits(1.0_f64.to_bits() + 2));
        assert!(ulps_eq!(a, b, max_ulps = 4));
        assert!(!ulps_eq!(a, b, max_ulps = 1));
    }

    #[test]
    fn bound_type_is_ignored() {
        let closed = FiniteBound::closed(1.0_f64);
        let open = FiniteBound::open(1.0_f64);
        assert!(abs_diff_eq!(closed, open, epsilon = 0.0));
        assert!(relative_eq!(closed, open, max_relative = 0.0));
        assert!(ulps_eq!(closed, open, max_ulps = 0));
    }

    fn bounded(lo: f64, hi: f64) -> FiniteInterval<f64> {
        FiniteInterval::new(FiniteBound::closed(lo), FiniteBound::closed(hi))
    }

    #[test]
    fn finite_interval_bounded_pairs() {
        let a = bounded(1.0, 2.0);
        let b = bounded(1.0 + 1e-12, 2.0 - 1e-12);
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));
    }

    #[test]
    fn finite_interval_ulps() {
        let lo = 1.0_f64;
        let hi = 2.0_f64;
        let lo_perturbed = f64::from_bits(lo.to_bits() + 2);
        let hi_perturbed = f64::from_bits(hi.to_bits() + 2);
        let a = bounded(lo, hi);
        let b = bounded(lo_perturbed, hi_perturbed);
        assert!(ulps_eq!(a, b, max_ulps = 4));
        assert!(!ulps_eq!(a, b, max_ulps = 1));
    }

    #[test]
    fn finite_interval_empty_matches_empty() {
        let e: FiniteInterval<f64> = FiniteInterval::empty();
        assert!(abs_diff_eq!(e, e, epsilon = 0.0));
        assert!(relative_eq!(e, e, max_relative = 0.0));
        assert!(ulps_eq!(e, e, max_ulps = 0));
    }

    #[test]
    fn finite_interval_empty_vs_bounded_never_equal() {
        let e: FiniteInterval<f64> = FiniteInterval::empty();
        let b = bounded(1.0, 2.0);
        assert!(!abs_diff_eq!(e, b, epsilon = f64::INFINITY));
        assert!(!relative_eq!(e, b, max_relative = f64::INFINITY));
        assert!(!ulps_eq!(e, b, max_ulps = u32::MAX));
    }

    #[test]
    fn half_interval_same_side() {
        let a = HalfInterval::left(FiniteBound::closed(1.0_f64));
        let b = HalfInterval::left(FiniteBound::closed(1.0 + 1e-12));
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));
    }

    #[test]
    fn half_interval_side_mismatch_never_equal() {
        let l = HalfInterval::left(FiniteBound::closed(1.0_f64));
        let r = HalfInterval::right(FiniteBound::closed(1.0_f64));
        assert!(!abs_diff_eq!(l, r, epsilon = f64::INFINITY));
        assert!(!relative_eq!(l, r, max_relative = f64::INFINITY));
        assert!(!ulps_eq!(l, r, max_ulps = u32::MAX));
    }

    #[test]
    fn enum_interval_same_variant_forwards() {
        let a = EnumInterval::Finite(bounded(1.0, 2.0));
        let b = EnumInterval::Finite(bounded(1.0 + 1e-12, 2.0 - 1e-12));
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));

        let h1 = EnumInterval::Half(HalfInterval::left(FiniteBound::closed(1.0_f64)));
        let h2 = EnumInterval::Half(HalfInterval::left(FiniteBound::closed(1.0 + 1e-12)));
        assert!(abs_diff_eq!(h1, h2, epsilon = 1e-9));
    }

    #[test]
    fn enum_interval_unbounded_matches_only_unbounded() {
        let u: EnumInterval<f64> = EnumInterval::Unbounded;
        assert!(abs_diff_eq!(u, u, epsilon = 0.0));
        assert!(relative_eq!(u, u, max_relative = 0.0));
        assert!(ulps_eq!(u, u, max_ulps = 0));
    }

    #[test]
    fn enum_interval_variant_mismatch_never_equal() {
        let f = EnumInterval::Finite(bounded(1.0, 2.0));
        let h = EnumInterval::Half(HalfInterval::left(FiniteBound::closed(1.0_f64)));
        let u: EnumInterval<f64> = EnumInterval::Unbounded;
        assert!(!abs_diff_eq!(f, h, epsilon = f64::INFINITY));
        assert!(!abs_diff_eq!(f, u, epsilon = f64::INFINITY));
        assert!(!abs_diff_eq!(h, u, epsilon = f64::INFINITY));
    }

    #[test]
    fn maybe_disjoint_consumed_matches_consumed() {
        let a: MaybeDisjoint<f64> = MaybeDisjoint::Consumed;
        assert!(abs_diff_eq!(a, a, epsilon = 0.0));
        assert!(relative_eq!(a, a, max_relative = 0.0));
        assert!(ulps_eq!(a, a, max_ulps = 0));
    }

    #[test]
    fn maybe_disjoint_connected_forwards() {
        let a = MaybeDisjoint::Connected(EnumInterval::Finite(bounded(1.0, 2.0)));
        let b = MaybeDisjoint::Connected(EnumInterval::Finite(bounded(1.0 + 1e-12, 2.0 - 1e-12)));
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
        assert!(relative_eq!(a, b, max_relative = 1e-9));
    }

    #[test]
    fn maybe_disjoint_pairs_compared_pairwise() {
        let a = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(bounded(1.0, 2.0)),
            EnumInterval::Finite(bounded(5.0, 6.0)),
        );
        let b = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(bounded(1.0 + 1e-12, 2.0)),
            EnumInterval::Finite(bounded(5.0, 6.0 - 1e-12)),
        );
        assert!(abs_diff_eq!(a, b, epsilon = 1e-9));
        assert!(!abs_diff_eq!(a, b, epsilon = 1e-15));
    }

    #[test]
    fn maybe_disjoint_variant_mismatch_never_equal() {
        let consumed: MaybeDisjoint<f64> = MaybeDisjoint::Consumed;
        let connected = MaybeDisjoint::Connected(EnumInterval::Finite(bounded(1.0, 2.0)));
        let disjoint = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(bounded(1.0, 2.0)),
            EnumInterval::Finite(bounded(5.0, 6.0)),
        );
        assert!(!abs_diff_eq!(consumed, connected, epsilon = f64::INFINITY));
        assert!(!abs_diff_eq!(consumed, disjoint, epsilon = f64::INFINITY));
        assert!(!abs_diff_eq!(connected, disjoint, epsilon = f64::INFINITY));
    }
}

//! Set-layer [`Cast`] / [`LossyCast`] / [`TryCast`] impls.
//!
//! The pattern mirrors the `TryAdd` precedent at
//! `intervalsets-core/src/ops/math/add.rs`: each set type's impl
//! extracts via `into_raw`, applies the per-bound element op, and
//! reassembles through the appropriate constructor chokepoint.
//! `try_new` runs `Element::validate` and the bound-pair invariant;
//! `try_satisfy_bounds` is the coercive sibling used by `LossyCast`
//! on `FiniteInterval` (crossed bounds → `Empty`).

use num_traits::Bounded;

use super::{Cast, CastElement, LossyCast, LossyCastElement, TryCast, TryCastElement};
use crate::bound::{BoundType, FiniteBound};
use crate::disjoint::MaybeDisjoint;
use crate::error::Error;
use crate::factory::TrySatisfyFiniteInterval;
use crate::numeric::Element;
use crate::ops::Connects;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};
use crate::MaybeEmpty;

// =====================================================================
// FiniteBound
// =====================================================================

impl<T, U> Cast<FiniteBound<U>> for FiniteBound<T>
where
    T: CastElement<U>,
{
    type Output = FiniteBound<U>;

    #[inline]
    fn cast(self) -> Self::Output {
        let (bt, v) = self.into_raw();
        FiniteBound::new_assume_valid(bt, v.cast_element())
    }
}

impl<T, U> LossyCast<FiniteBound<U>> for FiniteBound<T>
where
    T: LossyCastElement<U>,
    U: PartialEq + Bounded,
{
    type Output = FiniteBound<U>;

    #[inline]
    fn lossy_cast(self) -> Self::Output {
        let (bt, v) = self.into_raw();
        let u = v.lossy_cast_element();
        // Snap to closed when the value saturated to U's extremum: the
        // open/closed distinction at the discarded boundary has no
        // remaining meaning once out-of-range elements are projected
        // away.
        let bt = if u == U::min_value() || u == U::max_value() {
            BoundType::Closed
        } else {
            bt
        };
        FiniteBound::new_assume_valid(bt, u)
    }
}

impl<T, U> TryCast<FiniteBound<U>> for FiniteBound<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = FiniteBound<U>;
    type Error = Error;

    #[inline]
    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        let (bt, v) = self.into_raw();
        let u = v.try_cast_element().ok_or(Error::InvalidBoundLimit)?;
        FiniteBound::try_new(bt, u)
    }
}

// =====================================================================
// FiniteInterval
// =====================================================================

impl<T, U> Cast<FiniteInterval<U>> for FiniteInterval<T>
where
    T: CastElement<U>,
    U: Element,
{
    type Output = FiniteInterval<U>;

    fn cast(self) -> Self::Output {
        match self.into_raw() {
            None => FiniteInterval::empty(),
            Some((lhs, rhs)) => {
                let l = lhs.cast();
                let r = rhs.cast();
                // try_new normalizes discrete bounds and re-checks the
                // bound-pair invariant. For standard-library `Into`
                // pairs the result is always Ok; the `.expect` would
                // fire only on a contract violation by a user-defined
                // `From`/`Into`.
                FiniteInterval::try_new(l, r)
                    .expect("monotone Cast must preserve FiniteInterval invariants")
            }
        }
    }
}

impl<T, U> LossyCast<FiniteInterval<U>> for FiniteInterval<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = FiniteInterval<U>;

    fn lossy_cast(self) -> Self::Output {
        match self.into_raw() {
            None => FiniteInterval::empty(),
            Some((lhs, rhs)) => {
                let l = lhs.lossy_cast();
                let r = rhs.lossy_cast();
                // Coercive: crossed bounds (two distinct T's collapsing
                // to the same U) → Empty. InvalidBoundLimit is
                // unreachable for library types — az::SaturatingCast
                // always produces finite, in-range U — so the
                // unwrap_or_else is a Tier-1 safety floor only
                // exercisable by user-defined T/U with non-standard
                // `validate` predicates.
                FiniteInterval::try_satisfy_bounds(l, r).unwrap_or_else(|_| FiniteInterval::empty())
            }
        }
    }
}

impl<T, U> TryCast<FiniteInterval<U>> for FiniteInterval<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = FiniteInterval<U>;
    type Error = Error;

    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        match self.into_raw() {
            None => Ok(FiniteInterval::empty()),
            Some((lhs, rhs)) => {
                let l = lhs.try_cast()?;
                let r = rhs.try_cast()?;
                FiniteInterval::try_new(l, r)
            }
        }
    }
}

// =====================================================================
// HalfInterval
// =====================================================================

impl<T, U> Cast<HalfInterval<U>> for HalfInterval<T>
where
    T: CastElement<U>,
    U: Element,
{
    type Output = HalfInterval<U>;

    fn cast(self) -> Self::Output {
        let (side, b) = self.into_raw();
        let b = b.cast();
        HalfInterval::try_new(side, b).expect("monotone Cast must preserve HalfInterval invariants")
    }
}

impl<T, U> LossyCast<HalfInterval<U>> for HalfInterval<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = HalfInterval<U>;

    fn lossy_cast(self) -> Self::Output {
        let (side, b) = self.into_raw();
        let b = b.lossy_cast();
        HalfInterval::try_new(side, b)
            .unwrap_or_else(|_| panic!("lossy_cast must preserve HalfInterval invariants"))
    }
}

impl<T, U> TryCast<HalfInterval<U>> for HalfInterval<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = HalfInterval<U>;
    type Error = Error;

    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        let (side, b) = self.into_raw();
        let b = b.try_cast()?;
        HalfInterval::try_new(side, b)
    }
}

// =====================================================================
// EnumInterval
// =====================================================================

impl<T, U> Cast<EnumInterval<U>> for EnumInterval<T>
where
    T: CastElement<U>,
    U: Element,
{
    type Output = EnumInterval<U>;

    fn cast(self) -> Self::Output {
        match self {
            EnumInterval::Finite(i) => EnumInterval::Finite(i.cast()),
            EnumInterval::Half(i) => EnumInterval::Half(i.cast()),
            EnumInterval::Unbounded => EnumInterval::Unbounded,
        }
    }
}

impl<T, U> LossyCast<EnumInterval<U>> for EnumInterval<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = EnumInterval<U>;

    fn lossy_cast(self) -> Self::Output {
        match self {
            EnumInterval::Finite(i) => EnumInterval::Finite(i.lossy_cast()),
            EnumInterval::Half(i) => EnumInterval::Half(i.lossy_cast()),
            EnumInterval::Unbounded => EnumInterval::Unbounded,
        }
    }
}

impl<T, U> TryCast<EnumInterval<U>> for EnumInterval<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = EnumInterval<U>;
    type Error = Error;

    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        match self {
            EnumInterval::Finite(i) => i.try_cast().map(EnumInterval::Finite),
            EnumInterval::Half(i) => i.try_cast().map(EnumInterval::Half),
            EnumInterval::Unbounded => Ok(EnumInterval::Unbounded),
        }
    }
}

// =====================================================================
// MaybeDisjoint — per-variant delegate to the inner `EnumInterval`
// casts. `Cast` (widening) preserves invariants by definition.
// `TryCast` errors on cast-induced invariant breakage (mirrors
// `IntervalSet::try_cast`). `LossyCast` repairs: empties drop,
// connecting pieces merge — consistent with element-layer distinctions
// already discarded.
// =====================================================================

impl<T, U> Cast<MaybeDisjoint<U>> for MaybeDisjoint<T>
where
    T: CastElement<U>,
    U: Element,
{
    type Output = MaybeDisjoint<U>;

    fn cast(self) -> MaybeDisjoint<U> {
        match self {
            MaybeDisjoint::Consumed => MaybeDisjoint::Consumed,
            MaybeDisjoint::Connected(i) => MaybeDisjoint::from_interval(i.cast()),
            // Monotone widening preserves non-empty, ordering, and
            // non-connecting invariants of `Disjoint`.
            MaybeDisjoint::Disjoint(a, b) => {
                MaybeDisjoint::new_disjoint_assume_valid(a.cast(), b.cast())
            }
        }
    }
}

impl<T, U> TryCast<MaybeDisjoint<U>> for MaybeDisjoint<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = MaybeDisjoint<U>;
    type Error = Error;

    fn try_cast(self) -> Result<MaybeDisjoint<U>, Error> {
        match self {
            MaybeDisjoint::Consumed => Ok(MaybeDisjoint::Consumed),
            MaybeDisjoint::Connected(i) => i.try_cast().map(MaybeDisjoint::from_interval),
            MaybeDisjoint::Disjoint(a, b) => {
                let a: EnumInterval<U> = a.try_cast()?;
                let b: EnumInterval<U> = b.try_cast()?;
                // Strict: post-cast intervals must still satisfy the
                // `Disjoint` invariants (non-empty, sorted, non-touching).
                // Any narrowing-induced violation surfaces as
                // `InvalidBoundPair` — the closest existing variant for
                // "a paired-structure invariant has broken".
                if a.is_empty() || b.is_empty() || a >= b || a.connects(&b) {
                    return Err(Error::InvalidBoundPair);
                }
                Ok(MaybeDisjoint::new_disjoint_assume_valid(a, b))
            }
        }
    }
}

impl<T, U> LossyCast<MaybeDisjoint<U>> for MaybeDisjoint<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = MaybeDisjoint<U>;

    fn lossy_cast(self) -> MaybeDisjoint<U> {
        match self {
            MaybeDisjoint::Consumed => MaybeDisjoint::Consumed,
            MaybeDisjoint::Connected(i) => MaybeDisjoint::from_interval(i.lossy_cast()),
            // `from_pair` absorbs every narrowing-induced repair:
            // empties drop to `Consumed`/`Connected`; reorder if both
            // saturate the same direction; merge if narrowing made the
            // gap vanish.
            MaybeDisjoint::Disjoint(a, b) => {
                MaybeDisjoint::from_pair(a.lossy_cast(), b.lossy_cast())
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::Side;
    use crate::factory::{FiniteFactory, TryFiniteFactory, UnboundedFactory};

    // ---------- Cast (infallible) ----------

    #[test]
    fn cast_finite_interval_int_widening() {
        let x = FiniteInterval::closed(0_i32, 10);
        let y: FiniteInterval<i64> = x.cast();
        assert_eq!(y, FiniteInterval::closed(0_i64, 10));
    }

    #[test]
    fn cast_finite_interval_float_widening() {
        let x = FiniteInterval::closed(0.0_f32, 1.0);
        let y: FiniteInterval<f64> = x.cast();
        assert_eq!(y, FiniteInterval::closed(0.0_f64, 1.0));
    }

    #[test]
    fn cast_finite_interval_empty_round_trips() {
        let x = FiniteInterval::<i32>::empty();
        let y: FiniteInterval<i64> = x.cast();
        assert!(y.is_empty());
    }

    #[test]
    fn cast_half_interval_preserves_side() {
        let left: HalfInterval<i32> = HalfInterval::left(FiniteBound::closed(5));
        let right: HalfInterval<i32> = HalfInterval::right(FiniteBound::closed(5));
        let left_64: HalfInterval<i64> = left.cast();
        let right_64: HalfInterval<i64> = right.cast();
        assert_eq!(left_64.side(), Side::Left);
        assert_eq!(right_64.side(), Side::Right);
    }

    #[test]
    fn cast_enum_unbounded_round_trips_for_any_target() {
        let x: EnumInterval<i32> = EnumInterval::unbounded();
        let y: EnumInterval<i64> = x.cast();
        assert!(matches!(y, EnumInterval::Unbounded));

        let z: EnumInterval<f64> = EnumInterval::<f32>::unbounded().cast();
        assert!(matches!(z, EnumInterval::Unbounded));
    }

    // ---------- LossyCast (saturating) ----------

    #[test]
    fn lossy_cast_int_narrowing_saturates() {
        let x = FiniteInterval::closed(0_i64, i64::MAX);
        let y: FiniteInterval<i32> = x.lossy_cast();
        assert_eq!(y, FiniteInterval::closed(0_i32, i32::MAX));
    }

    /// Regression: locks in az's f64→f32 saturation behavior — we
    /// require clamping to `[f32::MIN, f32::MAX]`, not producing
    /// `±INF` (which `Element::validate` would reject).
    #[test]
    fn lossy_cast_f64_to_f32_clamps_not_infinity() {
        let x = FiniteInterval::closed(0.0_f64, f64::MAX);
        let y: FiniteInterval<f32> = x.lossy_cast();
        assert_eq!(y.view_raw().unwrap().1.value(), &f32::MAX);
        assert!(y.view_raw().unwrap().1.value().is_finite());
    }

    #[test]
    fn lossy_cast_saturation_snaps_to_closed() {
        // Open bounds at f64::MAX values saturate to f32::MAX — once
        // the out-of-range region is discarded the open/closed mark
        // becomes meaningless and snaps to closed.
        let x = FiniteInterval::try_open(-f64::MAX, f64::MAX).unwrap();
        let y: FiniteInterval<f32> = x.lossy_cast();
        let (l, r) = y.view_raw().unwrap();
        assert!(l.is_closed());
        assert!(r.is_closed());
        assert_eq!(l.value(), &f32::MIN);
        assert_eq!(r.value(), &f32::MAX);
    }

    #[test]
    fn lossy_cast_collision_collapses_to_empty() {
        // Two distinct f64 values whose f32 projections are equal:
        // f64::EPSILON adds a tiny bit, but the rounded f32 of
        // 1.0 + f64::EPSILON is just 1.0_f32. So open(1.0, 1.0+EPS)
        // becomes open(1.0_f32, 1.0_f32) — degenerate, collapses to
        // Empty.
        let lo = 1.0_f64;
        let hi = 1.0_f64 + f64::EPSILON;
        assert_ne!(lo, hi);
        assert_eq!(lo as f32, hi as f32);
        let x = FiniteInterval::try_open(lo, hi).unwrap();
        let y: FiniteInterval<f32> = x.lossy_cast();
        assert!(y.is_empty());
    }

    #[test]
    fn lossy_cast_enum_unbounded_round_trips() {
        let x: EnumInterval<f64> = EnumInterval::unbounded();
        let y: EnumInterval<f32> = x.lossy_cast();
        assert!(matches!(y, EnumInterval::Unbounded));
    }

    // ---------- TryCast (strict) ----------

    #[test]
    fn try_cast_widening_succeeds() {
        let x = FiniteInterval::closed(0_i32, 10);
        let y: FiniteInterval<i64> = x.try_cast().unwrap();
        assert_eq!(y, FiniteInterval::closed(0_i64, 10));
    }

    #[test]
    fn try_cast_element_overflow_errors() {
        let x = FiniteInterval::closed(0_i64, i64::MAX);
        let y: Result<FiniteInterval<i32>, _> = x.try_cast();
        assert!(matches!(y, Err(Error::InvalidBoundLimit)));
    }

    #[test]
    fn try_cast_post_cast_non_finite_errors() {
        // NumCast::from(f64::MAX -> f32) returns Some(f32::INFINITY);
        // the trailing FiniteBound::try_new runs Element::validate
        // which rejects non-finite.
        let x = FiniteInterval::closed(0.0_f64, f64::MAX);
        let y: Result<FiniteInterval<f32>, _> = x.try_cast();
        assert!(matches!(y, Err(Error::InvalidBoundLimit)));
    }

    #[test]
    fn try_cast_bound_collision_errors() {
        let lo = 1.0_f64;
        let hi = 1.0_f64 + f64::EPSILON;
        assert_eq!(lo as f32, hi as f32);
        let x = FiniteInterval::try_open(lo, hi).unwrap();
        let y: Result<FiniteInterval<f32>, _> = x.try_cast();
        assert!(matches!(y, Err(Error::InvalidBoundPair)));
    }

    #[test]
    fn try_cast_empty_round_trips() {
        let x = FiniteInterval::<f64>::empty();
        let y: FiniteInterval<f32> = x.try_cast().unwrap();
        assert!(y.is_empty());
    }

    #[test]
    fn try_cast_half_interval_preserves_side() {
        let left: HalfInterval<i64> = HalfInterval::left(FiniteBound::closed(5));
        let right: HalfInterval<i64> = HalfInterval::right(FiniteBound::closed(5));
        let left_32: HalfInterval<i32> = left.try_cast().unwrap();
        let right_32: HalfInterval<i32> = right.try_cast().unwrap();
        assert_eq!(left_32.side(), Side::Left);
        assert_eq!(right_32.side(), Side::Right);
    }

    #[test]
    fn try_cast_enum_unbounded_round_trips() {
        let x: EnumInterval<i64> = EnumInterval::unbounded();
        let y: EnumInterval<i32> = x.try_cast().unwrap();
        assert!(matches!(y, EnumInterval::Unbounded));
    }

    // ---------- NaN-panic boundary (Tier 4 bypass documentation) ----------

    /// `Element::validate` rejects NaN at construction time, so the
    /// validating API never routes NaN into `lossy_cast`. This test
    /// uses the Tier 4 `new_assume_valid` bypass to construct an
    /// invariant-violating `FiniteBound<f64>` containing NaN, and
    /// confirms `az`'s NaN-to-int panic surfaces — documenting the
    /// bypass-misuse failure mode in test form.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn nan_via_tier4_bypass_panics_in_lossy_cast_to_int() {
        let bad = FiniteBound::new_assume_valid(BoundType::Closed, f64::NAN);
        let _: FiniteBound<i32> = bad.lossy_cast();
    }

    // ---------- MaybeDisjoint ----------

    #[test]
    fn cast_maybe_disjoint_consumed() {
        let x = MaybeDisjoint::<i32>::Consumed;
        let y: MaybeDisjoint<i64> = x.cast();
        assert!(matches!(y, MaybeDisjoint::Consumed));
    }

    #[test]
    fn cast_maybe_disjoint_connected_widening() {
        let x = MaybeDisjoint::Connected(EnumInterval::closed(0_i32, 10));
        let y: MaybeDisjoint<i64> = x.cast();
        match y {
            MaybeDisjoint::Connected(i) => assert_eq!(i, EnumInterval::closed(0_i64, 10)),
            _ => panic!("expected Connected"),
        }
    }

    #[test]
    fn cast_maybe_disjoint_disjoint_widening_preserves_invariants() {
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::closed(0_i32, 10),
            EnumInterval::closed(20_i32, 30),
        );
        let y: MaybeDisjoint<i64> = x.cast();
        match y {
            MaybeDisjoint::Disjoint(a, b) => {
                assert_eq!(a, EnumInterval::closed(0_i64, 10));
                assert_eq!(b, EnumInterval::closed(20_i64, 30));
            }
            _ => panic!("expected Disjoint"),
        }
    }

    #[test]
    fn try_cast_maybe_disjoint_widening() {
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::closed(0_i32, 10),
            EnumInterval::closed(20_i32, 30),
        );
        let y: MaybeDisjoint<i64> = x.try_cast().unwrap();
        assert!(matches!(y, MaybeDisjoint::Disjoint(_, _)));
    }

    #[test]
    fn try_cast_maybe_disjoint_collision_errors() {
        // Two f64 intervals whose f32 projections collide → narrowing
        // breaks Disjoint invariants → InvalidBoundPair.
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        assert_eq!(lo1 as f32, lo2 as f32);
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(
                FiniteInterval::try_new(FiniteBound::closed(lo1), FiniteBound::closed(hi1))
                    .unwrap(),
            ),
            EnumInterval::Finite(
                FiniteInterval::try_new(FiniteBound::closed(lo2), FiniteBound::closed(hi2))
                    .unwrap(),
            ),
        );
        let y: Result<MaybeDisjoint<f32>, _> = x.try_cast();
        assert!(matches!(y, Err(Error::InvalidBoundPair)));
    }

    #[test]
    fn lossy_cast_maybe_disjoint_collision_merges() {
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(
                FiniteInterval::try_new(FiniteBound::closed(lo1), FiniteBound::closed(hi1))
                    .unwrap(),
            ),
            EnumInterval::Finite(
                FiniteInterval::try_new(FiniteBound::closed(lo2), FiniteBound::closed(hi2))
                    .unwrap(),
            ),
        );
        let y: MaybeDisjoint<f32> = x.lossy_cast();
        assert!(!matches!(y, MaybeDisjoint::Disjoint(_, _)));
    }

    #[test]
    fn lossy_cast_maybe_disjoint_widening_preserves_invariants() {
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::closed(0_i64, 10),
            EnumInterval::closed(20_i64, 30),
        );
        let y: MaybeDisjoint<i32> = x.lossy_cast();
        match y {
            MaybeDisjoint::Disjoint(a, b) => {
                assert_eq!(a, EnumInterval::closed(0_i32, 10));
                assert_eq!(b, EnumInterval::closed(20_i32, 30));
            }
            _ => panic!("expected Disjoint after lossless widening"),
        }
    }
}

use num_traits::Bounded;

use crate::cast::{Cast, CastElement, LossyCast, LossyCastElement, TryCast, TryCastElement};
use crate::error::Error;
use crate::numeric::Element;
use crate::ops::{Connects, MergeConnected};
use crate::{EnumInterval, FiniteInterval, HalfInterval, MaybeEmpty};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaybeDisjoint<T> {
    Consumed,
    Connected(EnumInterval<T>),
    Disjoint(EnumInterval<T>, EnumInterval<T>),
}

impl<T> Iterator for MaybeDisjoint<T> {
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inst = Self::Consumed;
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Consumed => None,
            Self::Connected(interval) => Some(interval),
            Self::Disjoint(lhs, rhs) => {
                let mut put_back = Self::Connected(rhs);
                core::mem::swap(self, &mut put_back);
                Some(lhs)
            }
        }
    }
}

impl<T> MaybeDisjoint<T> {
    pub fn empty() -> Self {
        Self::Consumed
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; returns `None` if this is two disjoint intervals.
    pub fn into_interval(self) -> Option<EnumInterval<T>> {
        match self {
            Self::Consumed => Some(EnumInterval::empty()),
            Self::Connected(interval) => Some(interval),
            Self::Disjoint(_, _) => None,
        }
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; panics otherwise.
    ///
    /// # Panics
    ///
    /// Panics if this is two disjoint intervals. Use
    /// [`into_interval`](Self::into_interval) for a panic-free alternative.
    pub fn expect_interval(self) -> EnumInterval<T> {
        self.into_interval()
            .expect("expected a single connected interval")
    }
}

impl<T> From<FiniteInterval<T>> for MaybeDisjoint<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<HalfInterval<T>> for MaybeDisjoint<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<EnumInterval<T>> for MaybeDisjoint<T> {
    fn from(interval: EnumInterval<T>) -> Self {
        if interval.is_empty() {
            Self::Consumed
        } else {
            Self::Connected(interval)
        }
    }
}

impl<T: Element> From<(EnumInterval<T>, EnumInterval<T>)> for MaybeDisjoint<T> {
    fn from(value: (EnumInterval<T>, EnumInterval<T>)) -> Self {
        debug_assert!(!value.0.is_empty());
        debug_assert!(!value.1.is_empty());
        debug_assert!(value.0 < value.1);
        debug_assert!(!value.0.connects(&value.1));
        Self::Disjoint(value.0, value.1)
    }
}

// === Cast support ===
//
// Per-variant delegation to the inner `EnumInterval` casts, with
// repair where narrowing breaks the `Disjoint` invariants
// (non-empty + sorted + non-connecting).
//
// - `Cast` (infallible widening) preserves invariants by definition —
//   no repair needed.
// - `TryCast` (strict) errors on cast-induced invariant violation,
//   mirroring how `IntervalSet::try_cast` errors on set-invariant
//   breakage.
// - `LossyCast` (saturating) repairs: empties drop, connecting
//   intervals merge — consistent with "we already discarded element
//   distinctions".

impl<T, U> Cast<MaybeDisjoint<U>> for MaybeDisjoint<T>
where
    T: CastElement<U>,
    U: Element,
{
    type Output = MaybeDisjoint<U>;

    fn cast(self) -> MaybeDisjoint<U> {
        match self {
            Self::Consumed => MaybeDisjoint::Consumed,
            Self::Connected(i) => MaybeDisjoint::Connected(i.cast()),
            // Monotone widening preserves non-empty, ordering, and
            // non-connecting. The `.expect` would only fire on a
            // user-defined `From`/`Into` contract violation.
            Self::Disjoint(a, b) => MaybeDisjoint::Disjoint(a.cast(), b.cast()),
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
            Self::Consumed => Ok(MaybeDisjoint::Consumed),
            Self::Connected(i) => {
                // From<EnumInterval<U>> normalizes empty → Consumed.
                Ok(MaybeDisjoint::from(<EnumInterval<T> as TryCast<
                    EnumInterval<U>,
                >>::try_cast(i)?))
            }
            Self::Disjoint(a, b) => {
                let a = <EnumInterval<T> as TryCast<EnumInterval<U>>>::try_cast(a)?;
                let b = <EnumInterval<T> as TryCast<EnumInterval<U>>>::try_cast(b)?;
                // Strict: post-cast intervals must still satisfy the
                // `Disjoint` invariants (non-empty, sorted, non-touching).
                // Any narrowing-induced violation surfaces as
                // `InvalidBoundPair` — the closest existing variant for
                // "a paired-structure invariant has broken".
                if a.is_empty() || b.is_empty() || a >= b || a.connects(&b) {
                    return Err(Error::InvalidBoundPair);
                }
                Ok(MaybeDisjoint::Disjoint(a, b))
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
            Self::Consumed => MaybeDisjoint::Consumed,
            Self::Connected(i) => MaybeDisjoint::from(<EnumInterval<T> as LossyCast<
                EnumInterval<U>,
            >>::lossy_cast(i)),
            Self::Disjoint(a, b) => {
                let a = <EnumInterval<T> as LossyCast<EnumInterval<U>>>::lossy_cast(a);
                let b = <EnumInterval<T> as LossyCast<EnumInterval<U>>>::lossy_cast(b);

                // Repair: drop empties first.
                match (a.is_empty(), b.is_empty()) {
                    (true, true) => MaybeDisjoint::Consumed,
                    (true, false) => MaybeDisjoint::Connected(b),
                    (false, true) => MaybeDisjoint::Connected(a),
                    (false, false) => {
                        // Narrowing can flip relative order at the extremes
                        // (e.g. both saturate the same direction). Re-sort.
                        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                        if lo.connects(&hi) {
                            match lo.merge_connected(hi) {
                                Some(merged) => MaybeDisjoint::Connected(merged),
                                None => {
                                    unreachable!("connects() implies merge_connected returns Some")
                                }
                            }
                        } else {
                            MaybeDisjoint::Disjoint(lo, hi)
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod cast_tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn cast_consumed() {
        let x = MaybeDisjoint::<i32>::Consumed;
        let y: MaybeDisjoint<i64> = x.cast();
        assert!(matches!(y, MaybeDisjoint::Consumed));
    }

    #[test]
    fn cast_connected_widening() {
        let x = MaybeDisjoint::Connected(EnumInterval::closed(0_i32, 10));
        let y: MaybeDisjoint<i64> = x.cast();
        match y {
            MaybeDisjoint::Connected(i) => assert_eq!(i, EnumInterval::closed(0_i64, 10)),
            _ => panic!("expected Connected"),
        }
    }

    #[test]
    fn cast_disjoint_widening_preserves_invariants() {
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
    fn try_cast_disjoint_widening() {
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::closed(0_i32, 10),
            EnumInterval::closed(20_i32, 30),
        );
        let y: MaybeDisjoint<i64> = x.try_cast().unwrap();
        assert!(matches!(y, MaybeDisjoint::Disjoint(_, _)));
    }

    #[test]
    fn try_cast_disjoint_collision_errors() {
        // Two f64 intervals whose f32 projections collide → narrowing
        // breaks Disjoint invariants → InvalidBoundPair.
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        assert_eq!(lo1 as f32, lo2 as f32);
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(
                FiniteInterval::try_new(
                    crate::bound::FiniteBound::closed(lo1),
                    crate::bound::FiniteBound::closed(hi1),
                )
                .unwrap(),
            ),
            EnumInterval::Finite(
                FiniteInterval::try_new(
                    crate::bound::FiniteBound::closed(lo2),
                    crate::bound::FiniteBound::closed(hi2),
                )
                .unwrap(),
            ),
        );
        let y: Result<MaybeDisjoint<f32>, _> = x.try_cast();
        assert!(matches!(y, Err(Error::InvalidBoundPair)));
    }

    #[test]
    fn lossy_cast_disjoint_collision_merges() {
        // Same input as the strict-collision test; LossyCast merges
        // the collapsed intervals rather than erroring.
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        let x = MaybeDisjoint::Disjoint(
            EnumInterval::Finite(
                FiniteInterval::try_new(
                    crate::bound::FiniteBound::closed(lo1),
                    crate::bound::FiniteBound::closed(hi1),
                )
                .unwrap(),
            ),
            EnumInterval::Finite(
                FiniteInterval::try_new(
                    crate::bound::FiniteBound::closed(lo2),
                    crate::bound::FiniteBound::closed(hi2),
                )
                .unwrap(),
            ),
        );
        let y: MaybeDisjoint<f32> = x.lossy_cast();
        assert!(!matches!(y, MaybeDisjoint::Disjoint(_, _)));
    }

    #[test]
    fn lossy_cast_disjoint_widening_preserves_invariants() {
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

//! Storage-type cast impls for the outer-crate set types
//! [`Interval`] and [`IntervalSet`].
//!
//! [`Interval`] delegates to the inner [`EnumInterval`] cast impls
//! from `intervalsets_core`. [`IntervalSet`] maps over its constituent
//! intervals and reassembles through the appropriate constructor:
//!
//! - [`Cast`] routes through [`IntervalSet::try_new`].
//!   Monotone widenings preserve sort + disjointness invariants; the
//!   trailing `.expect` would fire only on a contract violation by a
//!   user-defined `From`/`Into`.
//! - [`LossyCast`] routes through [`IntervalSet::new`] (repairing).
//!   Two distinct intervals can project onto the same narrowed range,
//!   merging is the consistent completion of "we already discarded
//!   distinctions at the element layer".
//! - [`TryCast`] routes through [`IntervalSet::try_new`]; collisions
//!   produced by narrowing surface as
//!   [`Error::InvalidIntervalSet`].

use intervalsets_core::cast::{Cast, LossyCast, LossyCastElement, TryCast, TryCastElement};
use intervalsets_core::sets::EnumInterval;
use num_traits::Bounded;

use crate::error::Error;
use crate::numeric::Element;
use crate::{Interval, IntervalSet};

// =====================================================================
// Interval (newtype delegate)
// =====================================================================

impl<T, U> Cast<Interval<U>> for Interval<T>
where
    T: Into<U>,
    U: Element,
{
    type Output = Interval<U>;

    #[inline]
    fn cast(self) -> Self::Output {
        Interval(<EnumInterval<T> as Cast<EnumInterval<U>>>::cast(self.0))
    }
}

impl<T, U> LossyCast<Interval<U>> for Interval<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = Interval<U>;

    #[inline]
    fn lossy_cast(self) -> Self::Output {
        Interval(<EnumInterval<T> as LossyCast<EnumInterval<U>>>::lossy_cast(
            self.0,
        ))
    }
}

impl<T, U> TryCast<Interval<U>> for Interval<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = Interval<U>;
    type Error = Error;

    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        <EnumInterval<T> as TryCast<EnumInterval<U>>>::try_cast(self.0)
            .map(Interval)
            .map_err(Into::into)
    }
}

// =====================================================================
// IntervalSet
// =====================================================================

impl<T, U> Cast<IntervalSet<U>> for IntervalSet<T>
where
    T: Into<U>,
    U: Element,
{
    type Output = IntervalSet<U>;

    fn cast(self) -> Self::Output {
        let mapped: Vec<Interval<U>> = self.into_raw().into_iter().map(Cast::cast).collect();
        // Monotone widening preserves sort + disjointness + non-touching.
        // The .expect would fire only on a user-defined `From`/`Into`
        // contract violation.
        IntervalSet::try_new(mapped).expect("monotone Cast must preserve IntervalSet invariants")
    }
}

impl<T, U> LossyCast<IntervalSet<U>> for IntervalSet<T>
where
    T: LossyCastElement<U>,
    U: Element + Bounded,
{
    type Output = IntervalSet<U>;

    fn lossy_cast(self) -> Self::Output {
        let mapped: Vec<Interval<U>> = self
            .into_raw()
            .into_iter()
            .map(LossyCast::lossy_cast)
            .collect();
        // Repairing: narrowing can collapse distinct intervals onto
        // overlapping ranges; `new` sorts + merges. Consistent with
        // already having discarded element-layer distinctions.
        IntervalSet::new(mapped)
    }
}

impl<T, U> TryCast<IntervalSet<U>> for IntervalSet<T>
where
    T: TryCastElement<U>,
    U: Element,
{
    type Output = IntervalSet<U>;
    type Error = Error;

    fn try_cast(self) -> Result<Self::Output, Self::Error> {
        let mapped: Vec<Interval<U>> = self
            .into_raw()
            .into_iter()
            .map(TryCast::try_cast)
            .collect::<Result<_, Error>>()?;
        IntervalSet::try_new(mapped)
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use intervalsets_core::factory::traits::*;

    use super::*;

    #[test]
    fn interval_cast_widening() {
        let x: Interval<i32> = Interval::closed(0, 10);
        let y: Interval<i64> = x.cast();
        assert_eq!(y, Interval::closed(0_i64, 10));
    }

    #[test]
    fn interval_try_cast_widening() {
        let x: Interval<i32> = Interval::closed(0, 10);
        let y: Interval<i64> = x.try_cast().unwrap();
        assert_eq!(y, Interval::closed(0_i64, 10));
    }

    #[test]
    fn interval_try_cast_overflow_errors() {
        let x: Interval<i64> = Interval::closed(0, i64::MAX);
        let y: Result<Interval<i32>, _> = x.try_cast();
        assert!(y.is_err());
    }

    #[test]
    fn interval_lossy_cast_clamps() {
        let x: Interval<f64> = Interval::closed(0.0, f64::MAX);
        let y: Interval<f32> = x.lossy_cast();
        // Right bound saturates to f32::MAX (not INF).
        match y.0 {
            EnumInterval::Finite(fi) => {
                let (_, r) = fi.view_raw().unwrap();
                assert_eq!(r.value(), &f32::MAX);
                assert!(r.value().is_finite());
            }
            _ => panic!("expected Finite"),
        }
    }

    // ---------- IntervalSet ----------

    #[test]
    fn interval_set_cast_widening_preserves_sort() {
        let set: IntervalSet<i32> = IntervalSet::new([
            Interval::closed(0, 10),
            Interval::closed(20, 30),
            Interval::closed(40, 50),
        ]);
        let widened: IntervalSet<i64> = set.cast();
        let expected: IntervalSet<i64> = IntervalSet::new([
            Interval::closed(0_i64, 10),
            Interval::closed(20, 30),
            Interval::closed(40, 50),
        ]);
        assert_eq!(widened, expected);
    }

    #[test]
    fn interval_set_try_cast_widening() {
        let set: IntervalSet<i32> =
            IntervalSet::new([Interval::closed(0, 10), Interval::closed(20, 30)]);
        let widened: IntervalSet<i64> = set.try_cast().unwrap();
        assert_eq!(widened.slice().len(), 2);
    }

    #[test]
    fn interval_set_try_cast_collision_errors() {
        // Two f64 intervals that collapse onto the same f32
        // neighborhood after cast, producing overlap / touching.
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        // Sanity: distinct in f64...
        assert_ne!(lo1, lo2);
        assert_ne!(hi1, hi2);
        // ...and the projection to f32 collapses them.
        assert_eq!(lo1 as f32, lo2 as f32);

        let set: IntervalSet<f64> = IntervalSet::new([
            Interval::from(
                intervalsets_core::sets::FiniteInterval::try_new(
                    intervalsets_core::bound::FiniteBound::closed(lo1),
                    intervalsets_core::bound::FiniteBound::closed(hi1),
                )
                .unwrap(),
            ),
            Interval::from(
                intervalsets_core::sets::FiniteInterval::try_new(
                    intervalsets_core::bound::FiniteBound::closed(lo2),
                    intervalsets_core::bound::FiniteBound::closed(hi2),
                )
                .unwrap(),
            ),
        ]);
        let result: Result<IntervalSet<f32>, _> = set.try_cast();
        assert!(matches!(result, Err(Error::InvalidIntervalSet)));
    }

    #[test]
    fn interval_set_lossy_cast_collision_merges() {
        // Same setup as the strict-collision test; LossyCast routes
        // through repairing `new` instead of failing.
        let lo1 = 1.0_f64;
        let hi1 = 1.0_f64 + 4.0 * f64::EPSILON;
        let lo2 = 1.0_f64 + 5.0 * f64::EPSILON;
        let hi2 = 1.0_f64 + 9.0 * f64::EPSILON;
        let set: IntervalSet<f64> = IntervalSet::new([
            Interval::from(
                intervalsets_core::sets::FiniteInterval::try_new(
                    intervalsets_core::bound::FiniteBound::closed(lo1),
                    intervalsets_core::bound::FiniteBound::closed(hi1),
                )
                .unwrap(),
            ),
            Interval::from(
                intervalsets_core::sets::FiniteInterval::try_new(
                    intervalsets_core::bound::FiniteBound::closed(lo2),
                    intervalsets_core::bound::FiniteBound::closed(hi2),
                )
                .unwrap(),
            ),
        ]);
        let merged: IntervalSet<f32> = set.lossy_cast();
        // After projection the two intervals merge into one.
        assert_eq!(merged.slice().len(), 1);
    }
}

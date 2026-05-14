//! Bisection by a caller-supplied measure. Public surface: [`Bisect`].

use core::convert::Infallible;

use super::Split;
use crate::bound::{SetBounds, Side};
use crate::numeric::{Element, Midpointable};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Result of bisecting a set by some measure.
///
/// `left` and `right` partition the original set (boundary ownership at
/// `midpoint` determined by `closed`). Under the measure passed to
/// [`Bisect::bisect_by`], `measure(left) ≈ measure(right)` — exactly
/// equal when the search lands on an exact balance point, otherwise
/// within one step of the element type (one ulp for floats, one unit
/// for integers).
// Fields are deliberately `pub` for the initial trait surface; the
// settled design (see scratch/bisection-notes.md) calls for private
// fields + accessors before promotion past pre-alpha.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bisection<T, S> {
    pub midpoint: T,
    pub left: S,
    pub right: S,
    pub closed: Side,
}

/// Split a set into two halves of approximately equal measure.
///
/// Treats the convex hull as the initial search bracket, repeatedly
/// splits the set at the bracket midpoint, walks the bracket toward
/// the heavier half, and terminates when the midpoint stabilizes
/// (ulp-stable for floats, neighbor-stable for integers).
///
/// Common measures: `|s| s.measure().finite()` (the natural measure
/// of `T` — cardinality for discrete, Lebesgue width for continuous),
/// `|s| s.span().unwrap().finite()` for diameter-based balance.
///
/// # Measure contract
///
/// `measure` must be **monotonic under set inclusion**: if `A ⊆ B`,
/// then `measure(A) ≤ measure(B)`. Equivalently, as the cut sweeps
/// from low to high across the bracket, `measure(left)` must be
/// non-decreasing and `measure(right)` non-increasing — this is what
/// makes the search a 1D root-find with a unique balance point to
/// converge on.
///
/// [`Measure`](crate::measure::Measure) and [`Span`](crate::ops::Span)
/// satisfy this directly. Any monotonic-increasing transform of a
/// valid measure (scaling by a positive constant, shifting, composing
/// with a monotonic function) is also valid; an inverted measure
/// (e.g. `|s| -s.measure().finite()`) is NOT — the search will walk
/// the wrong way and converge on a meaningless point.
///
/// Violating the contract does not cause undefined behavior or
/// non-termination — the loop still halts when the bracket midpoint
/// stabilizes — but the returned `midpoint` carries no balance
/// guarantee. If you need to split by a non-monotonic objective, you
/// want optimization (e.g. dense sampling over the bracket), not
/// bisection.
///
/// # `PartialOrd` is partial
///
/// For `U = f64` (or any type whose `PartialOrd` admits incomparable
/// values), a measure that can yield `NaN` will silently bias the
/// search: `NaN < x` and `NaN == x` are both `false`, so every
/// comparison falls through to the "right is heavier" branch. Strip
/// `NaN` at the measure boundary — `s.measure().finite()` returns a
/// finite value and is the canonical pattern.
pub trait Bisect<T>: Sized {
    /// Bisect by `measure`. Returns `None` if the set is empty or not
    /// finitely bounded. `measure` must satisfy the monotonicity
    /// contract documented on [`Bisect`].
    fn bisect_by<F, U>(&self, closed: Side, measure: F) -> Option<Bisection<T, Self>>
    where
        F: Fn(&Self) -> U,
        U: PartialOrd;
}

#[doc(hidden)]
pub fn bisect_core<T, S, F, U, Sp>(mut lo: T, mut hi: T, measure: F, split: Sp) -> (T, S, S)
where
    T: Clone + PartialEq + Midpointable<Error = Infallible>,
    F: Fn(&S) -> U,
    U: PartialOrd,
    Sp: Fn(T) -> (S, S),
{
    let mut m = T::midpoint(lo.clone(), hi.clone()).unwrap();

    let (left, right) = loop {
        let (l, r) = split(m.clone());
        let lw = measure(&l);
        let rw = measure(&r);

        if lw == rw {
            break (l, r);
        }
        if lw < rw {
            lo = m.clone();
        } else {
            hi = m.clone();
        }

        let new_m = T::midpoint(lo.clone(), hi.clone()).unwrap();
        if new_m == m {
            break (l, r);
        }
        m = new_m;
    };

    (m, left, right)
}

fn finite_bounds<T: Clone, S: SetBounds<T>>(s: &S) -> Option<(T, T)> {
    Some((s.lval()?.clone(), s.rval()?.clone()))
}

impl<T> Bisect<T> for FiniteInterval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect_by<F, U>(&self, closed: Side, measure: F) -> Option<Bisection<T, Self>>
    where
        F: Fn(&Self) -> U,
        U: PartialOrd,
    {
        let (lo, hi) = finite_bounds(self)?;
        let (midpoint, left, right) =
            bisect_core(lo, hi, measure, |m| self.clone().split(m, closed));
        Some(Bisection {
            midpoint,
            left,
            right,
            closed,
        })
    }
}

impl<T> Bisect<T> for HalfInterval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect_by<F, U>(&self, _closed: Side, _measure: F) -> Option<Bisection<T, Self>>
    where
        F: Fn(&Self) -> U,
        U: PartialOrd,
    {
        None
    }
}

impl<T> Bisect<T> for EnumInterval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect_by<F, U>(&self, closed: Side, measure: F) -> Option<Bisection<T, Self>>
    where
        F: Fn(&Self) -> U,
        U: PartialOrd,
    {
        let (lo, hi) = finite_bounds(self)?;
        let (midpoint, left, right) =
            bisect_core(lo, hi, measure, |m| self.clone().split(m, closed));
        Some(Bisection {
            midpoint,
            left,
            right,
            closed,
        })
    }
}

impl<T> Bisect<T> for MaybeDisjoint<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect_by<F, U>(&self, closed: Side, measure: F) -> Option<Bisection<T, Self>>
    where
        F: Fn(&Self) -> U,
        U: PartialOrd,
    {
        let (lo, hi) = finite_bounds(&self.hull())?;
        let (midpoint, left, right) =
            bisect_core(lo, hi, measure, |m| self.clone().split(m, closed));
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
    use super::*;
    use crate::factory::traits::*;
    use crate::measure::Measure;

    fn assert_close(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() <= eps, "expected {a} ≈ {b} (eps {eps})");
    }

    #[test]
    fn connected_single_interval_splits_at_midpoint() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_close(b.midpoint, 5.0, 1e-12);
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-12);
    }

    #[test]
    fn two_equal_width_pieces_balance_in_gap() {
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 1.0),
            EnumInterval::closed(9.0, 10.0),
        );
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-12);
        assert!(b.midpoint > 1.0 && b.midpoint < 9.0);
    }

    #[test]
    fn uneven_pieces_balance_inside_larger_piece() {
        // [0, 1] ∪ [5, 15]: total width 11, half is 5.5.
        // Balanced cut puts width 5.5 on each side.
        // Left side must contain [0,1] (width 1) plus 4.5 of the right
        // piece, so it spans up to 5 + 4.5 = 9.5.
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 1.0),
            EnumInterval::closed(5.0, 15.0),
        );
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-9);
        assert_close(b.midpoint, 9.5, 1e-9);
    }

    #[test]
    fn empty_returns_none() {
        let md = MaybeDisjoint::<f64>::empty();
        assert!(md.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn half_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::unbound_closed(0.0_f64));
        assert!(md.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn fully_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::<f64>::unbounded());
        assert!(md.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    // ---- i64 cases ----

    #[test]
    fn i64_connected_balances_to_within_one() {
        // Width 10 over [0, 10]: discrete bisect can split as 5/5 (lw=5 from
        // [0,5], rw=5 from [6,10] — actually 4 because [6,10] has width 4),
        // so the balanced exact case doesn't exist; algorithm picks the
        // closest midpoint.
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_i64, 10));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn i64_two_pieces_balance_in_gap() {
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i64, 10),
            EnumInterval::closed(100, 110),
        );
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        // Under unified Measure, i64 cardinality = b-a+1; [0,10] has 11 elements.
        assert_eq!(lw, 11);
        assert_eq!(rw, 11);
        assert!(b.midpoint > 10 && b.midpoint < 100);
    }

    #[test]
    fn i64_empty_returns_none() {
        let md = MaybeDisjoint::<i64>::empty();
        assert!(md.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn i64_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed_unbound(0_i64));
        assert!(md.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn i64_singleton_terminates() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(7_i64, 7));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 7);
    }

    #[test]
    fn generic_smoke_u32() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_u32, 1000));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn generic_smoke_f32() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f32, 10.0));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert!((b.midpoint - 5.0).abs() < 1e-5);
    }

    // ---- degenerate cases ----

    #[test]
    fn singleton_left_closed_puts_point_on_left() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(5_i64, 5));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 5);
        assert_eq!(
            b.left,
            MaybeDisjoint::from_interval(EnumInterval::closed(5, 5))
        );
        assert_eq!(b.right, MaybeDisjoint::empty());
    }

    #[test]
    fn singleton_right_closed_puts_point_on_right() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(5_i64, 5));
        let b = md
            .bisect_by(Side::Right, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 5);
        assert_eq!(b.left, MaybeDisjoint::empty());
        assert_eq!(
            b.right,
            MaybeDisjoint::from_interval(EnumInterval::closed(5, 5))
        );
    }

    #[test]
    fn two_singletons_partition_at_hull_midpoint() {
        // Width is 0 on both halves regardless of where we cut, so the
        // first midpoint candidate (hull-mid = 3) is the answer.
        let md =
            MaybeDisjoint::from_pair(EnumInterval::closed(1_i64, 1), EnumInterval::closed(5, 5));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 3);
        assert_eq!(
            b.left,
            MaybeDisjoint::from_interval(EnumInterval::closed(1, 1))
        );
        assert_eq!(
            b.right,
            MaybeDisjoint::from_interval(EnumInterval::closed(5, 5))
        );
    }

    #[test]
    fn measure_zero_set_picks_hull_midpoint_not_median() {
        // {1, 100}: width-bisect returns hull-mid (50), NOT a value
        // near either singleton. This is "correct" — width has no
        // information to distinguish positions within a zero-measure
        // set — but documents that bisect's midpoint is NOT the
        // statistical median of the elements.
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(1_i64, 1),
            EnumInterval::closed(100, 100),
        );
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 50);
        assert_eq!(
            b.left,
            MaybeDisjoint::from_interval(EnumInterval::closed(1, 1))
        );
        assert_eq!(
            b.right,
            MaybeDisjoint::from_interval(EnumInterval::closed(100, 100))
        );
    }

    #[test]
    fn measure_zero_set_cardinality_bisect_is_population_aware() {
        // Same set as above. Cardinality-bisect sees 1 element on each
        // side of any cut in (1, 100), so it also reports balanced on the
        // first iteration — but unlike width, cardinality CAN distinguish
        // off-balance positions for non-singleton sets (see
        // bisect_by_cardinality_works for the non-degenerate case).
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(1_i64, 1),
            EnumInterval::closed(100, 100),
        );
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        assert_eq!(b.midpoint, 50);
        let lc = b.left.measure().finite();
        let rc = b.right.measure().finite();
        assert_eq!(lc, 1);
        assert_eq!(rc, 1);
    }

    #[test]
    fn constant_measure_short_circuits_at_hull_midpoint() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_i64, 100));
        let b = md.bisect_by(Side::Left, |_| 42_u32).expect("bounded");
        assert_eq!(b.midpoint, 50);
    }

    #[test]
    fn bisect_by_cardinality_works() {
        // Proves the closure path drives the algorithm correctly for
        // Cardinality. On integers, width-bisect and cardinality-bisect
        // converge to the same midpoint up to ±1 (width and cardinality
        // differ only by the +1 endpoint quirk per piece), so this just
        // verifies the cardinality-based path produces cardinality-balanced
        // halves — not that it differs from width-bisect.
        let md =
            MaybeDisjoint::from_pair(EnumInterval::closed(0_i64, 0), EnumInterval::closed(10, 20));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lc = b.left.measure().finite();
        let rc = b.right.measure().finite();
        assert!(lc.abs_diff(rc) <= 1, "lc={lc}, rc={rc}");
    }

    #[test]
    fn bisect_by_arbitrary_closure() {
        // Any monotonic function of width is a valid measure for
        // bisection — the algorithm only cares about ordering. Scaled
        // width gives the same midpoint as plain width.
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        let b = md
            .bisect_by(Side::Left, |s| s.measure().finite() * 2.0)
            .expect("bounded");
        assert!((b.midpoint - 5.0).abs() < 1e-9);
    }

    #[test]
    fn closed_side_is_recorded_in_result() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        assert_eq!(
            md.bisect_by(Side::Left, |s| s.measure().finite())
                .unwrap()
                .closed,
            Side::Left
        );
        assert_eq!(
            md.bisect_by(Side::Right, |s| s.measure().finite())
                .unwrap()
                .closed,
            Side::Right
        );
    }

    // ---- impls on other core set types ----

    #[test]
    fn finite_interval_bisects() {
        let fi = FiniteInterval::closed(0_i64, 100);
        let b = fi
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn finite_interval_empty_returns_none() {
        let fi = FiniteInterval::<i64>::empty();
        assert!(fi.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn half_interval_always_returns_none() {
        let hi = HalfInterval::closed_unbound(0_i64);
        assert!(hi.bisect_by(Side::Left, |_| 0_u128).is_none());
    }

    #[test]
    fn enum_interval_finite_variant_bisects() {
        let ei = EnumInterval::closed(0_i64, 100);
        let b = ei
            .bisect_by(Side::Left, |s| s.measure().finite())
            .expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn enum_interval_half_variant_returns_none() {
        let ei: EnumInterval<i64> = EnumInterval::closed_unbound(0);
        assert!(ei.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }

    #[test]
    fn enum_interval_unbounded_returns_none() {
        let ei = EnumInterval::<i64>::unbounded();
        assert!(ei.bisect_by(Side::Left, |s| s.measure().finite()).is_none());
    }
}

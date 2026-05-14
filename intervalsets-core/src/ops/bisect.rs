//! Bisection by the natural [`Measure`](crate::measure::Measure) of a
//! set. Public surface: [`Bisect`].

use core::convert::Infallible;

use super::Split;
use crate::bound::{SetBounds, Side};
use crate::error::MathError;
use crate::measure::Measure;
use crate::numeric::{Element, Midpointable};
use crate::ops::math::TryAdd;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Result of bisecting a set.
///
/// `left` and `right` partition the original set (boundary ownership
/// at `midpoint` determined by `closed`).
/// `measure(left) ≈ measure(right)` — exactly equal when an exact
/// balance point exists, otherwise within one step of the element
/// type (one ulp for floats, one unit for integers).
// Fields are deliberately `pub` for the initial trait surface;
// promote to private fields + accessors before pre-alpha is over.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bisection<T, S> {
    /// The midpoint that partitioned the set into 2 equal mass sets
    pub midpoint: T,
    /// The left half of the set
    pub left: S,
    /// The right half of the set
    pub right: S,
    /// Which side contains the midpoint (if midpoint exists in the set)
    pub closed: Side,
}

/// Split a set into two halves of approximately equal natural
/// [`Measure`](crate::measure::Measure) (cardinality on discrete `T`,
/// Lebesgue width on continuous `T`).
///
/// For a single connected interval, the split happens at the midpoint
/// of the bounds in one step — uniform measure density means the
/// midpoint is already the balance point (off by ±1 for discrete `T`
/// due to integer rounding).
///
/// For a multi-piece set, the convex hull is the initial search
/// bracket. The algorithm repeatedly splits the set at the bracket
/// midpoint, walks the bracket toward the heavier half, and
/// terminates when the midpoint stabilizes (ulp-stable for floats,
/// neighbor-stable for integers).
pub trait Bisect<T>: Sized {
    /// Bisect by the natural measure of the set. Returns `None` if
    /// the set is empty or not finitely bounded.
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>>;
}

#[doc(hidden)]
pub fn bisect_core<T, S, Sp>(mut lo: T, mut hi: T, split: Sp) -> (T, S, S)
where
    T: Clone + PartialEq + Midpointable<Error = Infallible>,
    S: Measure,
    S::Output: PartialOrd,
    Sp: Fn(T) -> (S, S),
{
    let mut m = T::midpoint(lo.clone(), hi.clone()).unwrap();

    let (left, right) = loop {
        let (l, r) = split(m.clone());
        let lw = l.measure();
        let rw = r.measure();

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

/// Split-at-midpoint helper. The bounds come from `bounds_src`; the
/// type that gets split is `set` (often the same value as `bounds_src`,
/// but for `MaybeDisjoint::Connected` the bounds live on the inner
/// interval while the value being split is the wrapping
/// `MaybeDisjoint`).
fn split_at_midpoint<T, B, S>(bounds_src: &B, set: S, closed: Side) -> Option<Bisection<T, S>>
where
    T: Clone + Midpointable<Error = Infallible>,
    B: SetBounds<T>,
    S: Split<T, Output = S>,
{
    let (lo, hi) = finite_bounds(bounds_src)?;
    let midpoint = T::midpoint(lo, hi).unwrap();
    let (left, right) = set.split(midpoint.clone(), closed);
    Some(Bisection {
        midpoint,
        left,
        right,
        closed,
    })
}

impl<T> Bisect<T> for FiniteInterval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>> {
        split_at_midpoint(self, self.clone(), closed)
    }
}

impl<T> Bisect<T> for HalfInterval<T> {
    fn bisect(&self, _closed: Side) -> Option<Bisection<T, Self>> {
        None
    }
}

impl<T> Bisect<T> for EnumInterval<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
{
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>> {
        split_at_midpoint(self, self.clone(), closed)
    }
}

impl<T> Bisect<T> for MaybeDisjoint<T>
where
    T: Element + Clone + Midpointable<Error = Infallible>,
    T::Measure: PartialOrd,
    <T::Measure as TryAdd>::Error: Into<MathError>,
{
    fn bisect(&self, closed: Side) -> Option<Bisection<T, Self>> {
        match self {
            // Structurally a single interval — split at midpoint, no iteration.
            Self::Connected(iv) => split_at_midpoint(iv, self.clone(), closed),
            Self::Disjoint(_, _) => {
                let (lo, hi) = finite_bounds(&self.hull())?;
                let (midpoint, left, right) =
                    bisect_core(lo, hi, |m| self.clone().split(m, closed));
                Some(Bisection {
                    midpoint,
                    left,
                    right,
                    closed,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    fn assert_close(a: f64, b: f64, eps: f64) {
        assert!((a - b).abs() <= eps, "expected {a} ≈ {b} (eps {eps})");
    }

    // ===== Single connected interval: split at midpoint, one step. =====

    #[test]
    fn connected_f64_splits_at_midpoint() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        let b = md.bisect(Side::Left).expect("bounded");
        assert_close(b.midpoint, 5.0, 1e-12);
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-12);
    }

    #[test]
    fn connected_i64_splits_at_midpoint() {
        // [0, 10]: midpoint 5. Split at 5 with Side::Left gives [0,5]
        // (cardinality 6) and (5,10]→[6,10] (cardinality 5).
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_i64, 10));
        let b = md.bisect(Side::Left).expect("bounded");
        assert_eq!(b.midpoint, 5);
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    // ===== Multi-piece bisect: iterate. =====

    #[test]
    fn two_equal_width_pieces_balance_in_gap() {
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 1.0),
            EnumInterval::closed(9.0, 10.0),
        );
        let b = md.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-12);
        assert!(b.midpoint > 1.0 && b.midpoint < 9.0);
    }

    #[test]
    fn uneven_pieces_balance_inside_larger_piece() {
        // [0, 1] ∪ [5, 15]: total width 11, half is 5.5. Balanced cut
        // puts width 5.5 on each side. Left = [0,1] (1.0) + [5, 9.5]
        // (4.5) = 5.5; right = (9.5, 15] (5.5).
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0.0_f64, 1.0),
            EnumInterval::closed(5.0, 15.0),
        );
        let b = md.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert_close(lw, rw, 1e-9);
        assert_close(b.midpoint, 9.5, 1e-9);
    }

    #[test]
    fn i64_two_pieces_balance_in_gap() {
        let md = MaybeDisjoint::from_pair(
            EnumInterval::closed(0_i64, 10),
            EnumInterval::closed(100, 110),
        );
        let b = md.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        // Each piece has 11 elements under cardinality semantics.
        assert_eq!(lw, 11);
        assert_eq!(rw, 11);
        assert!(b.midpoint > 10 && b.midpoint < 100);
    }

    // ===== Edge cases =====

    #[test]
    fn empty_returns_none() {
        let md = MaybeDisjoint::<f64>::empty();
        assert!(md.bisect(Side::Left).is_none());
    }

    #[test]
    fn half_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::unbound_closed(0.0_f64));
        assert!(md.bisect(Side::Left).is_none());
    }

    #[test]
    fn fully_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::<f64>::unbounded());
        assert!(md.bisect(Side::Left).is_none());
    }

    #[test]
    fn i64_empty_returns_none() {
        let md = MaybeDisjoint::<i64>::empty();
        assert!(md.bisect(Side::Left).is_none());
    }

    #[test]
    fn i64_unbounded_returns_none() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed_unbound(0_i64));
        assert!(md.bisect(Side::Left).is_none());
    }

    #[test]
    fn i64_singleton_terminates() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(7_i64, 7));
        let b = md.bisect(Side::Left).expect("bounded");
        assert_eq!(b.midpoint, 7);
    }

    #[test]
    fn generic_smoke_u32() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_u32, 1000));
        let b = md.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn generic_smoke_f32() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f32, 10.0));
        let b = md.bisect(Side::Left).expect("bounded");
        assert!((b.midpoint - 5.0).abs() < 1e-5);
    }

    // ===== Closed-side semantics on singletons =====

    #[test]
    fn singleton_left_closed_puts_point_on_left() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(5_i64, 5));
        let b = md.bisect(Side::Left).expect("bounded");
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
        let b = md.bisect(Side::Right).expect("bounded");
        assert_eq!(b.midpoint, 5);
        assert_eq!(b.left, MaybeDisjoint::empty());
        assert_eq!(
            b.right,
            MaybeDisjoint::from_interval(EnumInterval::closed(5, 5))
        );
    }

    #[test]
    fn two_singletons_partition_at_hull_midpoint() {
        let md =
            MaybeDisjoint::from_pair(EnumInterval::closed(1_i64, 1), EnumInterval::closed(5, 5));
        let b = md.bisect(Side::Left).expect("bounded");
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
    fn closed_side_is_recorded_in_result() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        assert_eq!(md.bisect(Side::Left).unwrap().closed, Side::Left);
        assert_eq!(md.bisect(Side::Right).unwrap().closed, Side::Right);
    }

    // ===== Impls on other core set types =====

    #[test]
    fn finite_interval_bisects() {
        let fi = FiniteInterval::closed(0_i64, 100);
        let b = fi.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn finite_interval_empty_returns_none() {
        let fi = FiniteInterval::<i64>::empty();
        assert!(fi.bisect(Side::Left).is_none());
    }

    #[test]
    fn half_interval_always_returns_none() {
        let hi = HalfInterval::closed_unbound(0_i64);
        assert!(hi.bisect(Side::Left).is_none());
    }

    #[test]
    fn enum_interval_finite_variant_bisects() {
        let ei = EnumInterval::closed(0_i64, 100);
        let b = ei.bisect(Side::Left).expect("bounded");
        let lw = b.left.measure().finite();
        let rw = b.right.measure().finite();
        assert!(lw.abs_diff(rw) <= 1, "lw={lw}, rw={rw}");
    }

    #[test]
    fn enum_interval_half_variant_returns_none() {
        let ei: EnumInterval<i64> = EnumInterval::closed_unbound(0);
        assert!(ei.bisect(Side::Left).is_none());
    }

    #[test]
    fn enum_interval_unbounded_returns_none() {
        let ei = EnumInterval::<i64>::unbounded();
        assert!(ei.bisect(Side::Left).is_none());
    }
}

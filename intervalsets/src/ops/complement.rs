use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

use crate::bound::Side::*;
use crate::factory::UnboundedFactory;
use crate::numeric::Element;
use crate::ops::Intersection;
use crate::{Interval, IntervalSet};

/// Defines the complement of a Set.
///
/// ```text
/// Let A  = { x | P(x) } =>
///     A' = { x | x ∉ A } = { x | !P(x) }
/// ```
pub trait Complement {
    type Output;

    fn complement(self) -> Self::Output;
}

impl<T: Element> Complement for FiniteInterval<T> {
    type Output = IntervalSet<T>;

    fn complement(self) -> Self::Output {
        match self {
            Self::Empty => IntervalSet::new_unchecked(vec![Interval::unbounded()]),
            Self::Bounded(lhs, rhs) => unsafe {
                // SAFETY: Assuming FiniteInterval invariants are satisfied, then lhs <= rhs and
                // new half intervals are properly sorted; bounds are comparable; manually renormalized.
                IntervalSet::new_unchecked(vec![
                    HalfInterval::new_unchecked(Right, lhs.flip().normalized(Right)).into(),
                    HalfInterval::new_unchecked(Left, rhs.flip().normalized(Left)).into(),
                ])
            },
        }
    }
}

impl<T: Element> Complement for HalfInterval<T> {
    type Output = IntervalSet<T>;

    fn complement(self) -> Self::Output {
        let side = self.side.flip();
        // Safety: Assume bound satisfies invariants; manually re-normalize after flip;
        unsafe { HalfInterval::new_unchecked(side, self.bound.flip().normalized(side)).into() }
    }
}

impl<T: Element> Complement for EnumInterval<T> {
    type Output = IntervalSet<T>;

    fn complement(self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.complement(),
            Self::Half(inner) => inner.complement(),
            Self::Unbounded => IntervalSet::empty(),
        }
    }
}

impl<T: Element> Complement for Interval<T> {
    type Output = IntervalSet<T>;

    fn complement(self) -> Self::Output {
        self.0.complement()
    }
}

impl<T: Element + Clone> Complement for IntervalSet<T> {
    type Output = Self;

    fn complement(self) -> Self::Output {
        self.into_iter()
            .map(|x| x.complement())
            .fold(Interval::unbounded().into(), Intersection::intersection)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};
    use crate::ops::{Contains, Union};

    #[quickcheck]
    fn test_finite_complement_i8(a: i8) {
        let baseline = Interval::open_closed(0, 50);
        let complement = baseline.clone().complement();

        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_finite_complement_f32(a: f32) {
        if f32::is_nan(a) {
            return;
        }

        let baseline = Interval::open_closed(0 as f32, 50.0);
        let complement = baseline.clone().complement();
        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_half_complement_i8(a: i8) {
        let baseline = Interval::unbound_closed(50 as i8);
        let complement = baseline.clone().complement();

        assert!(baseline.contains(&a) != complement.contains(&a));
    }

    #[quickcheck]
    fn test_set_complement_i32(a: i32, b: i32, c: i32) {
        let a = Interval::closed(a, a.saturating_add(100));
        let b = Interval::closed(b, b.saturating_add(100));
        let c = Interval::closed(c, c.saturating_add(100));

        let set = IntervalSet::new(vec![a, b, c]);

        assert_eq!(set.clone().complement().complement(), set);
    }

    #[quickcheck]
    fn test_set_complement_f32(a: f32, b: f32, c: f32) {
        if f32::is_nan(a) || f32::is_nan(b) || f32::is_nan(c) {
            return;
        }

        let a = Interval::closed(a, a + 100.0);
        let b = Interval::closed(b, b + 100.0);
        let c = Interval::closed(c, c + 100.0);

        let set = IntervalSet::new(vec![a, b, c]);

        assert_eq!(set.clone().complement().complement(), set);
    }

    #[quickcheck]
    fn check_complement_laws_f32(a: f32, b: f32) -> bool {
        if a.is_nan() || b.is_nan() || a.is_infinite() || b.is_infinite() {
            return true; // skip malformed inputs
        }

        let a = Interval::closed(a, b);
        let c = a.clone().complement();

        // This one we need to fix
        assert_eq!(
            a.clone().union(c.clone()).expect_interval(),
            Interval::unbounded()
        );
        assert_eq!(a.intersection(c).expect_interval(), Interval::empty());

        true
    }
}

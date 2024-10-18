use crate::numeric::Domain;
use crate::{Intersection, Interval, IntervalSet};

/// Defines the complement of a Set.
///
/// Let A  = { x in T | Pred(x) }
///     A' = { x in T | x not in A }
pub trait Complement {
    type Output;

    fn complement(&self) -> Self::Output;
}

impl<T: Domain> Complement for Interval<T> {
    type Output = IntervalSet<T>;

    fn complement(&self) -> Self::Output {
        self.0.complement()
    }
}

impl<T: Domain> Complement for IntervalSet<T> {
    type Output = Self;

    fn complement(&self) -> Self::Output {
        self.intervals()
            .iter()
            .map(|x| x.complement())
            .fold(Interval::unbounded().into(), |l, r| l.intersection(&r))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{Contains, Union};

    #[quickcheck]
    fn test_finite_complement_i8(a: i8) {
        let baseline = Interval::open_closed(0, 50);
        let complement = baseline.complement();

        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_finite_complement_f32(a: f32) {
        if f32::is_nan(a) {
            return;
        }

        let baseline = Interval::open_closed(0 as f32, 50.0);
        let complement = baseline.complement();
        assert!(baseline.contains(&a) != complement.contains(&a))
    }

    #[quickcheck]
    fn test_half_complement_i8(a: i8) {
        let baseline = Interval::unbound_closed(50 as i8);
        let complement = baseline.complement();

        assert!(baseline.contains(&a) != complement.contains(&a));
    }

    #[quickcheck]
    fn test_set_complement_i32(a: i32, b: i32, c: i32) {
        let a = Interval::closed(a, a.saturating_add(100));
        let b = Interval::closed(b, b.saturating_add(100));
        let c = Interval::closed(c, c.saturating_add(100));

        let set = IntervalSet::new(vec![a, b, c]);

        assert_eq!(set.complement().complement(), set);
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

        assert_eq!(set.complement().complement(), set);
    }

    #[quickcheck]
    fn check_complement_laws_f32(a: f32, b: f32) -> bool {
        if a.is_nan() || b.is_nan() || a.is_infinite() || b.is_infinite() {
            return true; // skip malformed inputs
        }

        let a = Interval::closed(a, b);
        let c = a.complement();

        // This one we need to fix
        //assert_eq!(a.union(&c), Interval::unbounded().into());
        assert_eq!(a.intersection(&c), Interval::empty().into());

        true
    }
}

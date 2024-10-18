use crate::numeric::Domain;
use crate::Interval;

/// Defines the union of two intervals if contiguous.
///
/// Disjoint sets return `None` unless one is the `Empty` Set,
/// in which case the other input Set is the result (which could
/// be `Empty`).
pub trait Merged<Rhs = Self> {
    type Output;

    fn merged(&self, rhs: &Rhs) -> Option<Self::Output>;
}

impl<T: Domain> Merged<Self> for Interval<T> {
    type Output = Self;

    fn merged(&self, rhs: &Self) -> Option<Self::Output> {
        self.0.merged(&rhs.0).map(|v| v.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::ops::Complement;

    use super::*;

    #[quickcheck]
    fn check_merge_half_complements_f32(x: f32) {
        if x.is_nan() {
            return;
        }

        let x = Interval::unbound_closed(x);
        let y = x.complement().intervals()[0].clone();

        assert_eq!(x.merged(&y).unwrap(), Interval::unbounded());
    }

    #[quickcheck]
    fn check_merge_half_complements_i32(x: i32) {
        let x = Interval::closed_unbound(x);
        let y = x.complement().intervals()[0].clone();

        assert_eq!(x.merged(&y).unwrap(), Interval::unbounded());
    }

    #[test]
    fn test_merged_max_i32() {
        let x = Interval::closed(0, i32::MAX);
        let y = Interval::closed(-100, -1);

        assert_eq!(x.merged(&y).unwrap(), Interval::closed(-100, i32::MAX));
    }
}

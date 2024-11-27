/// Defines the union of two intervals if [connected](super::Connects).
///
/// # Note
///
/// > For types that do not implement Eq (such as primitive floats),
/// > the adjacency check breaks down. We support floats for their
/// > utility, but leave handling the edge cases to the end user.
/// > If rigorous correctness is required, then a fixed precision
/// > type should be used instead.
///
/// # Example
/// ```
/// use intervalsets::prelude::*;
///
/// let x = Interval::closed(0, 10);
/// let y = Interval::closed(11, 20);
/// assert_eq!(x.try_merge(y).unwrap(), Interval::closed(0, 20));
///
/// let y = Interval::closed(20, 30);
/// assert_eq!(x.try_merge(y), None);
///
/// let y = Interval::<i32>::empty();
/// assert_eq!(x.try_merge(y).unwrap(), x);
///
/// let x = Interval::<i32>::empty();
/// assert_eq!(x.try_merge(y).unwrap(), Interval::empty());
/// ```
pub use intervalsets_core::ops::TryMerge;

use crate::numeric::Element;
use crate::Interval;

impl<T: Element> TryMerge<Self> for Interval<T> {
    type Output = Self;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        self.0.try_merge(rhs.0).map(Interval::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;
    use crate::ops::Complement;

    #[quickcheck]
    fn check_merge_half_complements_f32(x: f32) {
        if x.is_nan() {
            return;
        }

        let x = Interval::unbound_closed(x);
        let y = x.clone().complement().expect_interval();

        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());
    }

    #[quickcheck]
    fn check_merge_half_complements_i32(x: i32) {
        let x = Interval::closed_unbound(x);
        let y = x.clone().complement().expect_interval();

        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());
    }

    #[test]
    fn test_regressions() {
        let x = Interval::closed_unbound(i32::MIN);
        let y = x.clone().complement().expect_interval();
        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());

        let x = Interval::unbound_closed(i32::MAX);
        let y = x.clone().complement().expect_interval();
        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());

        let x = Interval::open_unbound(0.0);
        let y = Interval::unbound_open(0.0);
        assert_eq!(x.try_merge(y), None);

        let x = Interval::closed_unbound(f32::MIN);
        let y = x.clone().complement().expect_interval();
        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());

        let x = Interval::unbound_closed(f32::MAX);
        let y = x.clone().complement().expect_interval();
        assert_eq!(x.try_merge(y).unwrap(), Interval::unbounded());
    }

    #[test]
    fn test_merged_max_i32() {
        let x = Interval::closed(0, i32::MAX);
        let y = Interval::closed(-100, -1);

        assert_eq!(x.try_merge(y).unwrap(), Interval::closed(-100, i32::MAX));
    }
}

/*
#[cfg(feature = "rust_decimal")]
#[cfg(test)]
mod decimal_test {
    use rust_decimal::Decimal;

    use super::*;
    use crate::{Factory, MaybeEmpty};

    #[quickcheck]
    fn check_decimal_merge(a: f32, b: f32, c: f32) {
        let a = Decimal::from_f32_retain(a);
        let b = Decimal::from_f32_retain(b);
        let c = Decimal::from_f32_retain(c);
        if a.is_none() || b.is_none() || c.is_none() {
            return;
        }

        let a = a.unwrap();
        let b = b.unwrap();
        let c = c.unwrap();

        let left = Interval::open(a.clone(), b.clone());
        let right = Interval::closed(b.clone(), c.clone());

        let merged = left.try_merge(&right).unwrap();
        if left.is_empty() {
            assert_eq!(merged, right);
        } else if right.is_empty() {
            assert_eq!(merged, left);
        } else {
            assert_eq!(
                left.try_merge(right).unwrap(),
                Interval::open_closed(a.clone(), c.clone())
            );
        }
    }
}

*/

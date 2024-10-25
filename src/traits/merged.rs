use crate::numeric::Domain;
use crate::Interval;

/// Defines the union of two intervals if contiguous.
///
/// Two intervals are contiguous if they share any elements **or** if
/// they are **adjacent** to each other such that they share bounds
/// with no other elements possible between them.
///
/// Other **disjoint sets** return `None` unless one is the `Empty` Set,
/// in which case the other input Set is the result.
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
/// use intervalsets::Interval;
/// use intervalsets::ops::{Merged, RefMerged};
///
/// let x = Interval::closed(0, 10);
/// let y = Interval::closed(11, 20);
/// assert_eq!(x.ref_merged(&y).unwrap(), Interval::closed(0, 20));
///
/// let y = Interval::closed(20, 30);
/// assert_eq!(x.ref_merged(&y), None);
///
/// let y = Interval::<i32>::empty();
/// assert_eq!(x.ref_merged(&y).unwrap(), x);
///
/// let x = Interval::<i32>::empty();
/// assert_eq!(x.merged(y).unwrap(), Interval::empty());
/// ```
pub trait Merged<Rhs = Self> {
    type Output;

    fn merged(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait RefMerged<Rhs = Self>: Merged<Rhs> + Clone
where
    Rhs: Clone,
{
    fn ref_merged(&self, rhs: &Rhs) -> Option<Self::Output> {
        self.clone().merged(rhs.clone())
    }
}

impl<T: Domain> Merged<Self> for Interval<T> {
    type Output = Self;

    fn merged(self, rhs: Self) -> Option<Self::Output> {
        self.0.merged(rhs.0).map(|v| v.into())
    }
}

impl<T: Domain> RefMerged<Self> for Interval<T> {
    fn ref_merged(&self, rhs: &Self) -> Option<Self::Output> {
        self.0.ref_merged(&rhs.0).map(|v| v.into())
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
        let y = x.clone().complement().expect_interval();

        assert_eq!(x.ref_merged(&y).unwrap(), Interval::unbounded());
    }

    #[quickcheck]
    fn check_merge_half_complements_i32(x: i32) {
        let x = Interval::closed_unbound(x);
        let y = x.clone().complement().expect_interval();

        assert_eq!(x.ref_merged(&y).unwrap(), Interval::unbounded());
    }

    #[test]
    fn test_merged_max_i32() {
        let x = Interval::closed(0, i32::MAX);
        let y = Interval::closed(-100, -1);

        assert_eq!(x.ref_merged(&y).unwrap(), Interval::closed(-100, i32::MAX));
    }
}

#[cfg(feature = "rust_decimal")]
#[cfg(test)]
mod decimal_test {
    use super::*;
    use crate::MaybeEmpty;
    use rust_decimal::Decimal;

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

        let merged = left.ref_merged(&right).unwrap();
        if left.is_empty() {
            assert_eq!(merged, right);
        } else if right.is_empty() {
            assert_eq!(merged, left);
        } else {
            assert_eq!(
                left.merged(right).unwrap(),
                Interval::open_closed(a.clone(), c.clone())
            );
        }
    }
}

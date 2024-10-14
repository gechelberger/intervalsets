use crate::concrete::finite::FiniteInterval;
use crate::ival::IVal;
use crate::numeric::Numeric;
use crate::Interval;

/// Defines the creation of the minimal contiguous Interval/Set
/// which covers all of the provided items.
///
/// # Example
/// ```
/// use intervalsets::Interval;
/// use intervalsets::op::hull::ConvexHull;
///
/// let interval = Interval::convex_hull([5, 3, -120, 44, 100, -100]);
/// assert_eq!(interval, Interval::closed(-120, 100));
/// ```
pub trait ConvexHull<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self;
}

impl<T: Numeric> ConvexHull<T> for FiniteInterval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut iter = iter.into_iter();

        // check our empty case first
        let mut bounds = match iter.next() {
            None => return FiniteInterval::Empty,
            Some(item) => (IVal::closed(item.clone()), IVal::closed(item)),
        };

        for item in iter {
            let item_ival = IVal::closed(item);
            let left = IVal::min_left(&bounds.0, &item_ival);
            let right = IVal::max_right(&bounds.1, &item_ival);
            bounds = (left, right);
        }

        FiniteInterval::new_unchecked(bounds.0, bounds.1)
    }
}

impl<T: Numeric> ConvexHull<T> for Interval<T> {
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        FiniteInterval::convex_hull(iter).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_hull_of_points() {
        let iv = FiniteInterval::convex_hull(vec![5, 3, -1, 30, 2, -22, 100, -100]);
        assert_eq!(iv, FiniteInterval::closed(-100, 100))
    }
}

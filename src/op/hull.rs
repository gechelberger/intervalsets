use crate::concrete::finite::FiniteInterval;
use crate::ival::IVal;
use crate::numeric::Numeric;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_hull_of_points() {
        let iv = FiniteInterval::convex_hull(vec![5, 3, -1, 30, 2, -22, 100, -100]);
        assert_eq!(iv, FiniteInterval::closed(-100, 100))
    }
}

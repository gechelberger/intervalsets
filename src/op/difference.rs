use super::complement::Complement;
use super::intersection::Intersection;
use super::union::Union;
use crate::numeric::Domain;
use crate::{EBounds, FiniteInterval, HalfBounded, Interval, IntervalSet};

/// Defines the difference of sets A - B.
///
/// { x in T | x in A<T> && x not in B<T>}
///
/// Difference is not commutative.
///
/// # Example
///
/// ```
/// use intervalsets::Interval;
/// use intervalsets::op::difference::Difference;
/// use intervalsets::op::union::Union;
///
/// let a = Interval::closed(0.0, 100.0);
/// let b = Interval::closed(50.0, 150.0);
/// assert_eq!(
///     a.difference(&b),
///     Interval::closed_open(0.0, 50.0).into()
/// );
/// assert_eq!(
///     b.difference(&a),
///     Interval::open_closed(100.0, 150.0).into()
/// );
/// ```
pub trait Difference<Rhs = Self> {
    type Output;

    fn difference(&self, rhs: &Rhs) -> Self::Output;
}

macro_rules! difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: Domain> Difference<$t_rhs> for $t_lhs {
            type Output = IntervalSet<T>;

            fn difference(&self, rhs: &$t_rhs) -> Self::Output {
                self.intersection(&rhs.complement()).into()
            }
        }
    };
}

impl<T: Domain> Difference for Interval<T> {
    type Output = IntervalSet<T>;
    
    fn difference(&self, rhs: &Self) -> Self::Output {
        self.0.difference(&rhs.0)
    }
}

difference_impl!(FiniteInterval<T>, FiniteInterval<T>);
difference_impl!(FiniteInterval<T>, HalfBounded<T>);
difference_impl!(HalfBounded<T>, FiniteInterval<T>);
difference_impl!(HalfBounded<T>, HalfBounded<T>);
difference_impl!(EBounds<T>, FiniteInterval<T>);
difference_impl!(EBounds<T>, HalfBounded<T>);
difference_impl!(EBounds<T>, EBounds<T>);
difference_impl!(FiniteInterval<T>, EBounds<T>);
difference_impl!(HalfBounded<T>, EBounds<T>);

//difference_impl!(IntervalSet<T>, FiniteInterval<T>);
//difference_impl!(IntervalSet<T>, HalfBounded<T>);
//difference_impl!(IntervalSet<T>, EBounds<T>);
//difference_impl!(FiniteInterval<T>, IntervalSet<T>);
//difference_impl!(HalfBounded<T>, IntervalSet<T>);
//difference_impl!(EBounds<T>, IntervalSet<T>);
//difference_impl!(IntervalSet<T>, IntervalSet<T>);

/// Defines the symmetric difference for sets A and B.
///
/// {x in T | (x in A || x in B) && (x not in A intersect B)}
///
/// Symmetric difference is commutative.
///
/// Example:
/// ```
/// use intervalsets::Interval;
/// use intervalsets::op::difference::SymmetricDifference;
/// use intervalsets::op::union::Union;
///
/// let a = Interval::closed(0.0, 10.0);
/// let b = Interval::closed(5.0, 15.0);
/// let expected = Interval::closed_open(0.0, 5.0)
///         .union(&Interval::open_closed(10.0, 15.0));
/// assert_eq!(a.sym_difference(&b), expected);
/// assert_eq!(b.sym_difference(&a), expected);
/// assert_eq!(a.sym_difference(&a), Interval::empty().into())
/// ```
pub trait SymmetricDifference<Rhs = Self> {
    type Output;

    fn sym_difference(&self, rhs: &Rhs) -> Self::Output;
}

macro_rules! sym_difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: Domain> SymmetricDifference<$t_rhs> for $t_lhs {
            type Output = IntervalSet<T>;

            fn sym_difference(&self, rhs: &$t_rhs) -> Self::Output {
                self.union(rhs).difference(&self.intersection(rhs))
            }
        }
    };
}

sym_difference_impl!(FiniteInterval<T>, FiniteInterval<T>);
sym_difference_impl!(FiniteInterval<T>, HalfBounded<T>);
sym_difference_impl!(HalfBounded<T>, FiniteInterval<T>);
sym_difference_impl!(HalfBounded<T>, HalfBounded<T>);
sym_difference_impl!(EBounds<T>, FiniteInterval<T>);
sym_difference_impl!(EBounds<T>, HalfBounded<T>);
sym_difference_impl!(EBounds<T>, EBounds<T>);
sym_difference_impl!(FiniteInterval<T>, EBounds<T>);
sym_difference_impl!(HalfBounded<T>, EBounds<T>);
sym_difference_impl!(IntervalSet<T>, FiniteInterval<T>);
sym_difference_impl!(IntervalSet<T>, HalfBounded<T>);
sym_difference_impl!(IntervalSet<T>, EBounds<T>);
sym_difference_impl!(FiniteInterval<T>, IntervalSet<T>);
sym_difference_impl!(HalfBounded<T>, IntervalSet<T>);
sym_difference_impl!(EBounds<T>, IntervalSet<T>);
sym_difference_impl!(IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use crate::EBounds;

    use super::*;

    #[test]
    fn test_finite_difference() {
        assert_eq!(
            FiniteInterval::closed(0.0, 10.0).difference(&FiniteInterval::closed(7.5, 15.0)),
            FiniteInterval::closed_open(0.0, 7.5).into()
        );

        assert_eq!(
            FiniteInterval::closed(7.5, 15.0).difference(&FiniteInterval::closed(0.0, 10.0)),
            FiniteInterval::open_closed(10.0, 15.0).into()
        );

        assert_eq!(
            FiniteInterval::closed(0.0, 10.0).difference(&FiniteInterval::closed(2.5, 7.5)),
            IntervalSet::new_unchecked(vec![
                EBounds::closed_open(0.0, 2.5),
                EBounds::open_closed(7.5, 10.0)
            ])
        );

        assert_eq!(
            FiniteInterval::closed(2.5, 7.5).difference(&FiniteInterval::closed(0.0, 10.0)),
            FiniteInterval::Empty.into()
        )
    }

    #[test]
    fn test_finite_sym_difference() {
        assert_eq!(
            EBounds::closed(0.0, 10.0).sym_difference(&EBounds::closed(5.0, 15.0)),
            EBounds::closed_open(0.0, 5.0).union(&EBounds::open_closed(10.0, 15.0))
        );

        assert_eq!(
            EBounds::closed(5.0, 15.0).sym_difference(&EBounds::closed(0.0, 10.0)),
            EBounds::closed_open(0.0, 5.0).union(&EBounds::open_closed(10.0, 15.0))
        );
    }
}

use crate::ops::{Complement, Intersection, RefIntersection, RefUnion};
use crate::{Interval, IntervalSet};

/// Defines the difference of sets A - B.
///
/// ```text
/// Let A ⊆ T, B ⊆ T:
///
/// { x | x ∈ A && x ∉ B }
/// ```
///
/// Difference is not commutative.
///
/// # Example
///
/// ```
/// use intervalsets::{Interval, Factory};
/// use intervalsets::ops::{Difference, Union};
///
/// let a = Interval::closed(0.0, 100.0);
/// let b = Interval::closed(50.0, 150.0);
/// assert_eq!(
///     a.clone().difference(b.clone()),
///     Interval::closed_open(0.0, 50.0).into()
/// );
/// assert_eq!(
///     b.difference(a),
///     Interval::open_closed(100.0, 150.0).into()
/// );
/// ```
pub trait Difference<Rhs = Self> {
    type Output;

    fn difference(self, rhs: Rhs) -> Self::Output;
}

pub trait RefDifference<Rhs>: Difference<Rhs> + Clone
where
    Rhs: Clone,
{
    fn ref_difference(&self, rhs: &Rhs) -> Self::Output {
        self.clone().difference(rhs.clone())
    }
}

macro_rules! difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Domain> $crate::ops::Difference<$t_rhs> for $t_lhs {
            type Output = $crate::IntervalSet<T>;

            fn difference(self, rhs: $t_rhs) -> Self::Output {
                self.intersection(rhs.complement()).into()
            }
        }
    };
}

difference_impl!(Interval<T>, Interval<T>);
difference_impl!(Interval<T>, IntervalSet<T>);
difference_impl!(IntervalSet<T>, Interval<T>);
difference_impl!(IntervalSet<T>, IntervalSet<T>);

macro_rules! ref_difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Domain> $crate::ops::RefDifference<$t_rhs> for $t_lhs {
            fn ref_difference(&self, rhs: &$t_rhs) -> <Self as $crate::ops::Difference>::Output {
                self.ref_intersection(&rhs.clone().complement()).into()
            }
        }
    };
}

ref_difference_impl!(Interval<T>, Interval<T>);
ref_difference_impl!(Interval<T>, IntervalSet<T>);
ref_difference_impl!(IntervalSet<T>, Interval<T>);
ref_difference_impl!(IntervalSet<T>, IntervalSet<T>);

/// Defines the symmetric difference (A ⊕ B). A and B are consumed.
///
/// ```text
/// Let A ⊆ T, B ⊆ T:
///
/// {x | x ∈ (A ∪ B) && x ∉ (A ∩ B) }
/// ```
///
/// Symmetric difference is commutative.
///
/// Example:
/// ```
/// use intervalsets::{Interval, Factory};
/// use intervalsets::ops::{SymmetricDifference, Union};
///
/// let a = Interval::closed(0.0, 10.0);
/// let b = Interval::closed(5.0, 15.0);
/// let expected = Interval::closed_open(0.0, 5.0)
///     .union(Interval::open_closed(10.0, 15.0));
/// assert_eq!(a.clone().sym_difference(b.clone()), expected);
/// assert_eq!(b.clone().sym_difference(a.clone()), expected);
/// assert_eq!(a.clone().sym_difference(a), Interval::empty().into())
/// ```
pub trait SymmetricDifference<Rhs = Self> {
    type Output;

    fn sym_difference(self, rhs: Rhs) -> Self::Output;
}

pub trait RefSymmetricDifference<Rhs = Self>: SymmetricDifference<Rhs> + Clone
where
    Rhs: Clone,
{
    fn ref_sym_difference(&self, rhs: &Rhs) -> Self::Output {
        self.clone().sym_difference(rhs.clone())
    }
}

macro_rules! sym_difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Domain> $crate::ops::SymmetricDifference<$t_rhs> for $t_lhs {
            type Output = $crate::IntervalSet<T>;

            fn sym_difference(self, rhs: $t_rhs) -> Self::Output {
                self.ref_union(&rhs).difference(self.intersection(rhs))
            }
        }
    };
}

//pub(crate) use sym_difference_impl;

sym_difference_impl!(Interval<T>, Interval<T>);
sym_difference_impl!(Interval<T>, IntervalSet<T>);
sym_difference_impl!(IntervalSet<T>, Interval<T>);
sym_difference_impl!(IntervalSet<T>, IntervalSet<T>);

macro_rules! ref_sym_difference_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Domain> $crate::ops::RefSymmetricDifference<$t_rhs> for $t_lhs {
            fn ref_sym_difference(&self, rhs: &$t_rhs) -> Self::Output {
                self.ref_union(rhs).difference(self.ref_intersection(rhs))
            }
        }
    };
}

ref_sym_difference_impl!(Interval<T>, Interval<T>);
ref_sym_difference_impl!(Interval<T>, IntervalSet<T>);
ref_sym_difference_impl!(IntervalSet<T>, Interval<T>);
ref_sym_difference_impl!(IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ops::Union;
    use crate::Factory;

    #[test]
    fn test_finite_difference() {
        assert_eq!(
            Interval::closed(0.0, 10.0).difference(Interval::closed(7.5, 15.0)),
            Interval::closed_open(0.0, 7.5).into()
        );

        assert_eq!(
            Interval::closed(7.5, 15.0).difference(Interval::closed(0.0, 10.0)),
            Interval::open_closed(10.0, 15.0).into()
        );

        assert_eq!(
            Interval::closed(0.0, 10.0).difference(Interval::closed(2.5, 7.5)),
            IntervalSet::new_unchecked(vec![
                Interval::closed_open(0.0, 2.5),
                Interval::open_closed(7.5, 10.0)
            ])
        );

        assert_eq!(
            Interval::closed(2.5, 7.5).difference(Interval::closed(0.0, 10.0)),
            Interval::empty().into()
        )
    }

    #[test]
    fn test_finite_sym_difference() {
        assert_eq!(
            Interval::closed(0.0, 10.0).sym_difference(Interval::closed(5.0, 15.0)),
            Interval::closed_open(0.0, 5.0).union(Interval::open_closed(10.0, 15.0))
        );

        assert_eq!(
            Interval::closed(5.0, 15.0).sym_difference(Interval::closed(0.0, 10.0)),
            Interval::closed_open(0.0, 5.0).union(Interval::open_closed(10.0, 15.0))
        );
    }
}

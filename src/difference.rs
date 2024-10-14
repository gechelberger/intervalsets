use crate::complement::Complement;
use crate::intersection::Intersection;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

pub trait Difference<Rhs = Self> {
    type Output;

    fn difference(&self, rhs: &Rhs) -> Self::Output;
}

macro_rules! impl_difference {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: Copy + PartialOrd> Difference<$t_rhs> for $t_lhs {
            type Output = IntervalSet<T>;

            fn difference(&self, rhs: &$t_rhs) -> Self::Output {
                self.intersection(&rhs.complement()).into()
            }
        }
    };
}

impl_difference!(FiniteInterval<T>, FiniteInterval<T>);
impl_difference!(FiniteInterval<T>, HalfInterval<T>);
impl_difference!(HalfInterval<T>, FiniteInterval<T>);
impl_difference!(HalfInterval<T>, HalfInterval<T>);
impl_difference!(Interval<T>, FiniteInterval<T>);
impl_difference!(Interval<T>, HalfInterval<T>);
impl_difference!(Interval<T>, Interval<T>);
impl_difference!(FiniteInterval<T>, Interval<T>);
impl_difference!(HalfInterval<T>, Interval<T>);
impl_difference!(IntervalSet<T>, FiniteInterval<T>);
impl_difference!(IntervalSet<T>, HalfInterval<T>);
impl_difference!(IntervalSet<T>, Interval<T>);
impl_difference!(FiniteInterval<T>, IntervalSet<T>);
impl_difference!(HalfInterval<T>, IntervalSet<T>);
impl_difference!(Interval<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use crate::Interval;

    use super::*;

    #[test]
    fn test_finite_difference() {
        assert_eq!(
            FiniteInterval::closed(0.0, 10.0).difference(&FiniteInterval::closed(7.5, 15.0)),
            FiniteInterval::closedopen(0.0, 7.5).into()
        );

        assert_eq!(
            FiniteInterval::closed(7.5, 15.0).difference(&FiniteInterval::closed(0.0, 10.0)),
            FiniteInterval::openclosed(10.0, 15.0).into()
        );

        assert_eq!(
            FiniteInterval::closed(0.0, 10.0).difference(&FiniteInterval::closed(2.5, 7.5)),
            IntervalSet::new_unchecked(vec![
                Interval::closed_open(0.0, 2.5),
                Interval::open_closed(7.5, 10.0)
            ])
        );

        assert_eq!(
            FiniteInterval::closed(2.5, 7.5).difference(&FiniteInterval::closed(0.0, 10.0)),
            FiniteInterval::Empty.into()
        )
    }
}

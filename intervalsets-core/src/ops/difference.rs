use super::{Complement, Difference, Intersection, Union};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};

macro_rules! difference_impl {
    ($t_lhs:ident, $t_rhs:ident) => {
        impl<T: $crate::numeric::Domain + Ord + Clone> $crate::ops::Difference<$t_rhs<T>>
            for $t_lhs<T>
        {
            type Output = $crate::sets::StackSet<T>;

            fn difference(self, rhs: $t_rhs<T>) -> Self::Output {
                self.intersection(rhs.complement()).into()
            }
        }
    };
}

macro_rules! sym_difference_impl {
    ($t_lhs:ident, $t_rhs:ident) => {
        impl<T: $crate::numeric::Domain + Ord + Clone> $crate::ops::SymDifference<$t_rhs<T>>
            for $t_lhs<T>
        {
            type Output = $crate::sets::StackSet<T>;

            fn sym_difference(self, rhs: $t_rhs<T>) -> Self::Output {
                //self.ref_union(&rhs).difference(self.intersection(rhs))
                self.clone()
                    .union(rhs.clone())
                    .difference(self.intersection(rhs))
            }
        }
    };
}

macro_rules! cartesian_impl {
    ($submacro:ident) => {
        $submacro!(FiniteInterval, FiniteInterval);
        $submacro!(FiniteInterval, HalfInterval);
        $submacro!(HalfInterval, FiniteInterval);
        $submacro!(HalfInterval, HalfInterval);
        $submacro!(EnumInterval, FiniteInterval);
        $submacro!(EnumInterval, HalfInterval);
        $submacro!(EnumInterval, EnumInterval);
        $submacro!(FiniteInterval, EnumInterval);
        $submacro!(HalfInterval, EnumInterval);
        $submacro!(StackSet, FiniteInterval);
        $submacro!(StackSet, HalfInterval);
        $submacro!(StackSet, EnumInterval);
        $submacro!(StackSet, StackSet);
        $submacro!(FiniteInterval, StackSet);
        $submacro!(HalfInterval, StackSet);
        $submacro!(EnumInterval, StackSet);
    };
}

cartesian_impl!(difference_impl);
cartesian_impl!(sym_difference_impl);

use crate::numeric::Domain;
use crate::util::commutative_op_move_impl;
use crate::{Interval, IntervalSet, MaybeEmpty};

/// Defines a Set with every element of both input Sets.
///
/// S_out = { x | x in S_left or x in S_right }
pub trait Union<Rhs = Self> {
    type Output;

    fn union(self, rhs: Rhs) -> Self::Output;
}

pub trait RefUnion<Rhs = Self>: Union<Rhs> + Clone
where
    Rhs: Clone,
{
    fn ref_union(&self, rhs: &Rhs) -> Self::Output {
        self.clone().union(rhs.clone())
    }
}

impl<T: Domain> Union<Self> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        self.0.union(rhs.0)
    }
}

impl<T: Domain> RefUnion<Self> for Interval<T> {
    fn ref_union(&self, rhs: &Self) -> Self::Output {
        self.0.ref_union(&rhs.0)
    }
}

impl<T: Domain> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Self) -> Self::Output {
        // need to restore the disjoint invariant
        Self::merge_sorted(itertools::merge(self, rhs))
    }
}

impl<T: Domain> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Interval<T>) -> Self::Output {
        if rhs.is_empty() {
            // IntervalSet::new does take care of this, but it has to check more things
            return self;
        }
        Self::from_iter(self.into_iter().chain(Some(rhs)))
    }
}

commutative_op_move_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);

impl<T: Domain> RefUnion<IntervalSet<T>> for Interval<T> {}
impl<T: Domain> RefUnion<Interval<T>> for IntervalSet<T> {}
impl<T: Domain> RefUnion<IntervalSet<T>> for IntervalSet<T> {}

//op_ref_clone_impl!(Union, union, Interval<T>, Interval<T>, IntervalSet<T>);
//op_ref_clone_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);
//op_ref_clone_impl!(Union, union, IntervalSet<T>, Interval<T>, IntervalSet<T>);
//op_ref_clone_impl!(Union, union, IntervalSet<T>, IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_union_empty() {
        assert_eq!(
            Interval::<i32>::empty().union(Interval::empty()),
            Interval::empty().into()
        )
    }

    #[test]
    fn test_finite_union_full() {
        assert_eq!(
            Interval::<i32>::closed(0, 100).union(Interval::closed(10, 20)),
            Interval::closed(0, 100).into()
        );

        assert_eq!(
            Interval::closed(10, 20).union(Interval::closed(0, 100)),
            Interval::closed(0, 100).into()
        );
    }

    #[test]
    fn test_finite_union_disjoint() {
        assert_eq!(
            Interval::<i32>::closed(0, 10).union(Interval::closed(100, 110)),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::closed(0, 10),
                Interval::closed(100, 110),
            ])
        );

        assert_eq!(
            Interval::<i32>::closed(100, 110).union(Interval::closed(0, 10)),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::closed(0, 10),
                Interval::closed(100, 110),
            ])
        );
    }

    #[test]
    fn test_set_union_infinite() {
        let a = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 10),
            Interval::closed_unbound(100),
        ]);

        let b = IntervalSet::new(vec![
            Interval::closed(-500, -400),
            Interval::closed(-350, -300),
            Interval::closed(-150, 150),
            Interval::closed(300, 500),
        ]);

        assert_eq!(a.clone().union(b.clone()), Interval::unbounded().into());
        assert_eq!(b.union(a), Interval::unbounded().into());
    }

    #[test]
    fn test_set_union() {
        let a = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 10),
            Interval::closed(100, 110),
            Interval::closed(200, 210),
            Interval::closed(300, 310),
        ]);

        let b = IntervalSet::new(vec![
            Interval::closed(400, 410),
            Interval::closed_unbound(1000),
        ]);

        let c = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 10),
            Interval::closed(100, 110),
            Interval::closed(200, 210),
            Interval::closed(300, 310),
            Interval::closed(400, 410),
            Interval::closed_unbound(1000),
        ]);

        assert_eq!(a.clone().union(b.clone()), c);
        assert_eq!(b.union(a), c);
    }

    #[test]
    fn test_set_union_finite() {
        let a = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 10),
            Interval::closed(100, 110),
            Interval::closed(200, 210),
            Interval::closed(300, 310),
        ]);

        let b = Interval::closed(5, 200);

        let c = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 210),
            Interval::closed(300, 310),
        ]);

        assert_eq!(a.clone().union(b.clone()), c);
        assert_eq!(b.union(a), c);
    }

    #[test]
    fn test_set_union_half() {
        let a = IntervalSet::new(vec![
            Interval::unbound_closed(-100),
            Interval::closed(0, 10),
            Interval::closed(100, 110),
            Interval::closed(200, 210),
            Interval::closed(300, 310),
        ]);

        let b = Interval::unbound_closed(150);

        let c = IntervalSet::new(vec![
            Interval::unbound_closed(150),
            Interval::closed(200, 210),
            Interval::closed(300, 310),
        ]);

        assert_eq!(a.clone().union(b.clone()), c);
        assert_eq!(b.union(a), c);
    }

    use crate::detail::test::CloneInt;

    #[test]
    fn test_non_copy_interval_union() {
        let a = CloneInt(0);
        let b = CloneInt(5);
        let c = CloneInt(10);
        let d = CloneInt(15);

        let ac = Interval::closed(a.clone(), c);
        let bd = Interval::closed(b, d.clone());

        assert_eq!(
            ac.clone().union(bd.clone()).expect_interval(),
            Interval::closed(a.clone(), d.clone())
        );

        assert_eq!(ac.union(bd).expect_interval(), Interval::closed(a, d));
    }
}

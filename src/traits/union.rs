use crate::{commutative_op_impl, Domain, Interval, IntervalSet, MaybeEmpty};

pub trait Union<Rhs = Self> {
    type Output;

    fn union(&self, rhs: &Rhs) -> Self::Output;
}

impl<T: Domain> Union<Self> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(&self, rhs: &Self) -> Self::Output {
        self.0.union(&rhs.0)
    }
}

impl<T: Domain> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Self) -> Self::Output {
        let intervals = itertools::merge(self.intervals().iter(), rhs.intervals().iter())
            .cloned()
            .collect();

        /*
        let mut intervals = Vec::with_capacity(self.intervals.len() + rhs.intervals.len());
        let left = itertools::put_back(self.intervals.iter());
        let right = itertools::put_back(rhs.intervals.iter());
        ...
        */

        // need to restore the disjoint invariant
        Self::new_unchecked(Self::merge_sorted(intervals))
    }
}

impl<T: Domain> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(&self, rhs: &Interval<T>) -> Self::Output {
        if rhs.is_empty() {
            // IntervalSet::new does take care of this, but it has to check more things
            return self.clone();
        }

        let mut intervals = self.intervals().clone();
        intervals.push(rhs.clone());
        Self::new(intervals)
    }
}

commutative_op_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_union_empty() {
        assert_eq!(
            Interval::<i32>::empty().union(&Interval::empty()),
            Interval::empty().into()
        )
    }

    #[test]
    fn test_finite_union_full() {
        assert_eq!(
            Interval::<i32>::closed(0, 100).union(&Interval::closed(10, 20)),
            Interval::closed(0, 100).into()
        );

        assert_eq!(
            Interval::closed(10, 20).union(&Interval::closed(0, 100)),
            Interval::closed(0, 100).into()
        );
    }

    #[test]
    fn test_finite_union_disjoint() {
        assert_eq!(
            Interval::<i32>::closed(0, 10).union(&Interval::closed(100, 110)),
            IntervalSet::<i32>::new_unchecked(vec![
                Interval::closed(0, 10),
                Interval::closed(100, 110),
            ])
        );

        assert_eq!(
            Interval::<i32>::closed(100, 110).union(&Interval::closed(0, 10)),
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

        assert_eq!(a.union(&b), Interval::unbounded().into());
        assert_eq!(b.union(&a), Interval::unbounded().into());
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

        assert_eq!(a.union(&b), c);
        assert_eq!(b.union(&a), c);
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

        assert_eq!(a.union(&b), c);
        assert_eq!(b.union(&a), c);
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

        assert_eq!(a.union(&b), c);
        assert_eq!(b.union(&a), c);
    }
}

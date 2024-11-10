use intervalsets_core::ops::merged::MergeSorted;
use intervalsets_core::sets::{FiniteInterval, HalfInterval};
use intervalsets_core::{EnumInterval, MaybeEmpty};
use FiniteInterval::Bounded;

use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::ops::{Adjacent, Contains, Intersects};
use crate::util::commutative_op_move_impl;
use crate::{Factory, Interval, IntervalSet};

fn merge_sorted_intervals<T, I>(iter: I) -> impl Iterator<Item = Interval<T>>
where
    T: Domain,
    I: IntoIterator<Item = Interval<T>>,
{
    MergeSorted::new(iter.into_iter().map(|x| x.0)).map(Interval::from)
}

fn ordered_pair<T: PartialOrd>(a: Interval<T>, b: Interval<T>) -> [Interval<T>; 2] {
    if a <= b {
        [a, b]
    } else {
        [b, a]
    }
}

pub trait Union<Rhs = Self> {
    type Output;

    fn union(self, rhs: Rhs) -> Self::Output;
}

mod icore {
    use super::*;

    impl<T: Domain> Union<Self> for FiniteInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: Self) -> Self::Output {
            if self.intersects(&rhs) || self.is_adjacent_to(&rhs) {
                let Bounded(lhs_min, lhs_max) = self else {
                    unreachable!();
                };

                let Bounded(rhs_min, rhs_max) = rhs else {
                    unreachable!();
                };

                // SAFETY: if self and rhs satisfy invariants then new interval
                // is normalized and min(left, right) <= max(left, right)
                let merged = unsafe {
                    FiniteInterval::new_unchecked(
                        FiniteBound::take_min(Side::Left, lhs_min, rhs_min),
                        FiniteBound::take_max(Side::Right, lhs_max, rhs_max),
                    )
                };

                IntervalSet::new_unchecked([merged.into()])
            } else if self.is_empty() {
                IntervalSet::from(Interval::from(rhs))
            } else if rhs.is_empty() {
                IntervalSet::from(Interval::from(self))
            } else {
                IntervalSet::new_unchecked(ordered_pair(self.into(), rhs.into()))
            }
        }
    }

    impl<T: Domain> Union<Self> for HalfInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: Self) -> Self::Output {
            if self.side == rhs.side {
                if self.contains(rhs.bound.value()) {
                    IntervalSet::new_unchecked([self.into()])
                } else {
                    IntervalSet::new_unchecked([rhs.into()])
                }
            } else if self.contains(rhs.bound.value())
                || rhs.contains(self.bound.value())
                || self.is_adjacent_to(&rhs)
            {
                IntervalSet::unbounded()
            } else {
                IntervalSet::new_unchecked(ordered_pair(self.into(), rhs.into()))
            }
        }
    }

    impl<T: Domain> Union<HalfInterval<T>> for FiniteInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: HalfInterval<T>) -> Self::Output {
            if rhs.contains(&self) {
                IntervalSet::new_unchecked([rhs.into()])
            } else if self.contains(rhs.bound.value()) || self.is_adjacent_to(&rhs) {
                let Bounded(lhs_min, lhs_max) = self else {
                    unreachable!();
                };

                let side = rhs.side;
                let bound = side.select(lhs_min, lhs_max);
                IntervalSet::new_unchecked([HalfInterval { side, bound }.into()])
            } else {
                IntervalSet::new_unchecked(ordered_pair(self.into(), rhs.into()))
            }
        }
    }

    impl<T: Domain> Union<FiniteInterval<T>> for HalfInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: FiniteInterval<T>) -> Self::Output {
            rhs.union(self)
        }
    }

    macro_rules! delegate_enum_impl {
        ($t:ty) => {
            impl<T: Domain> Union<$t> for EnumInterval<T> {
                type Output = IntervalSet<T>;

                fn union(self, rhs: $t) -> Self::Output {
                    match self {
                        Self::Finite(lhs) => lhs.union(rhs),
                        Self::Half(lhs) => lhs.union(rhs),
                        Self::Unbounded => IntervalSet::unbounded(),
                    }
                }
            }
        };
    }

    delegate_enum_impl!(FiniteInterval<T>);
    delegate_enum_impl!(HalfInterval<T>);
    delegate_enum_impl!(EnumInterval<T>);
    commutative_op_move_impl!(
        Union,
        union,
        FiniteInterval<T>,
        EnumInterval<T>,
        IntervalSet<T>
    );
    commutative_op_move_impl!(
        Union,
        union,
        HalfInterval<T>,
        EnumInterval<T>,
        IntervalSet<T>
    );
}

impl<T: Domain> Union<Self> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        self.0.union(rhs.0)
    }
}

impl<T: Domain> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Self) -> Self::Output {
        Self::new_unchecked(merge_sorted_intervals(itertools::merge(self, rhs)))
    }
}

impl<T: Domain> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Interval<T>) -> Self::Output {
        Self::new_unchecked(merge_sorted_intervals(itertools::merge(
            self,
            core::iter::once(rhs),
        )))
    }
}

commutative_op_move_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Factory;

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
}

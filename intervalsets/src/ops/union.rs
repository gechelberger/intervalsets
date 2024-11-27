use core::iter::once;

use intervalsets_core::ops::MergeSortedByValue;
use intervalsets_core::sets::{FiniteInterval, HalfInterval};
use intervalsets_core::EnumInterval;
use num_traits::Zero;

use crate::bound::Side::{Left, Right};
use crate::bound::{FiniteBound, Side};
use crate::factory::UnboundedFactory;
use crate::numeric::Element;
use crate::ops::{Connects, Contains};
use crate::util::commutative_op_move_impl;
use crate::{Interval, IntervalSet};

fn ordered_pair<T: PartialOrd>(a: Interval<T>, b: Interval<T>) -> [Interval<T>; 2] {
    if a <= b {
        [a, b]
    } else {
        [b, a]
    }
}

/// The (possibly disjoint) union of A and B.
///
/// ```text
/// { x | x ∈ A ∨ x ∈ B }
/// ```
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x = Interval::closed(0, 10);
/// let y = Interval::closed(5, 15);
/// assert_eq!(x.union(y).expect_interval(), Interval::closed(0, 15));
///
/// let y = Interval::closed(20, 30);
/// assert_eq!(x.union(y), IntervalSet::new([x, y]));
/// ```
pub trait Union<Rhs = Self> {
    /// The type created by this operation.
    type Output;

    /// Creates a set with every element of self and rhs.
    fn union(self, rhs: Rhs) -> Self::Output;
}

mod icore {
    use super::*;

    impl<T: Element> Union<Self> for FiniteInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: Self) -> Self::Output {
            if self.connects(&rhs) {
                let Some((lhs_min, lhs_max)) = self.into_raw() else {
                    return IntervalSet::from(rhs);
                };

                let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
                    // SAFETY: putting it back together
                    return IntervalSet::from(unsafe { Self::new_unchecked(lhs_min, lhs_max) });
                };

                // SAFETY: if self and rhs satisfy invariants then new interval
                // is normalized and min(left, right) <= max(left, right)
                let merged = unsafe {
                    FiniteInterval::new_unchecked(
                        FiniteBound::take_min_unchecked(Side::Left, lhs_min, rhs_min),
                        FiniteBound::take_max_unchecked(Side::Right, lhs_max, rhs_max),
                    )
                };

                IntervalSet::from(merged)
            } else {
                let ordpair = ordered_pair(self.into(), rhs.into());
                // SAFETY:
                // 2. intervals are sorted here
                // 1+3. Just checked that the two sets are not connected
                unsafe { IntervalSet::new_unchecked(ordpair) }
            }
        }
    }

    impl<T: Element> Union<Self> for HalfInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: Self) -> Self::Output {
            if self.side() == rhs.side() {
                if self.contains(rhs.finite_ord_bound()) {
                    IntervalSet::from(self)
                } else {
                    IntervalSet::from(rhs)
                }
            } else if self.connects(&rhs) {
                IntervalSet::unbounded()
            } else {
                let ordpair = ordered_pair(self.into(), rhs.into());
                // SAFETY:
                // 2: intervals are sorted here
                // 1+3: intervals are not connected (and therefore also non-empty)
                unsafe { IntervalSet::new_unchecked(ordpair) }
            }
        }
    }

    impl<T: Element> Union<HalfInterval<T>> for FiniteInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: HalfInterval<T>) -> Self::Output {
            if self.connects(&rhs) {
                let Some((lhs_min, lhs_max)) = self.into_raw() else {
                    return IntervalSet::from(rhs);
                };

                let left_contained = rhs.side() == Left && rhs.contains(lhs_min.finite_ord(Left));
                let right_contained =
                    rhs.side() == Right && rhs.contains(lhs_max.finite_ord(Right));
                if left_contained || right_contained {
                    IntervalSet::from(rhs)
                } else {
                    let bound = rhs.side().select(lhs_min, lhs_max);
                    // SAFETY: bound stolen from existing FiniteInterval
                    let merged = unsafe { HalfInterval::new_unchecked(rhs.side(), bound) };
                    IntervalSet::from(merged)
                }
            } else {
                let ordpair = ordered_pair(self.into(), rhs.into());
                // SAFETY:
                // 2. intervals are sorted here
                // 1+3. intervals not connected (and therefore non-empty)
                unsafe { IntervalSet::new_unchecked(ordpair) }
            }
        }
    }

    impl<T: Element> Union<FiniteInterval<T>> for HalfInterval<T> {
        type Output = IntervalSet<T>;

        fn union(self, rhs: FiniteInterval<T>) -> Self::Output {
            rhs.union(self)
        }
    }

    macro_rules! delegate_enum_impl {
        ($t:ty) => {
            impl<T> Union<$t> for EnumInterval<T>
            where
                T: $crate::numeric::Element,
                T: $crate::numeric::Zero,
            {
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

impl<T: Element + Zero> Union<Self> for Interval<T> {
    type Output = IntervalSet<T>;

    fn union(self, rhs: Self) -> Self::Output {
        self.0.union(rhs.0)
    }
}

impl<T: Element + Zero> Union<Self> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Self) -> Self::Output {
        let sorted = itertools::merge(self, rhs);
        // SAFETY:
        // 1. Neither operand may produce the empty set per invariants.
        // 2. Operands are sorted per invariants.
        // 3. MergSortedByValue merged connected intervals if properly sorted.
        unsafe { Self::new_unchecked(MergeSortedByValue::new(sorted)) }
    }
}

impl<T: Element + Zero> Union<Interval<T>> for IntervalSet<T> {
    type Output = Self;

    fn union(self, rhs: Interval<T>) -> Self::Output {
        let sorted = itertools::merge(self, once(rhs));
        // SAFETY:
        // 1. MergeSortedByValue strips empty sets from the head of its input.
        // 2. values are sorted if self invariants are satisfied.
        // 3. MergeSortedByValue merges connected intervals if properly sorted.
        unsafe { Self::new_unchecked(MergeSortedByValue::new(sorted)) }
    }
}

commutative_op_move_impl!(Union, union, Interval<T>, IntervalSet<T>, IntervalSet<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

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
            IntervalSet::<i32>::new(vec![Interval::closed(0, 10), Interval::closed(100, 110),])
        );

        assert_eq!(
            Interval::<i32>::closed(100, 110).union(Interval::closed(0, 10)),
            IntervalSet::<i32>::new(vec![Interval::closed(0, 10), Interval::closed(100, 110),])
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

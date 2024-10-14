use crate::ival::Side;
use crate::numeric::Numeric;
use crate::pred::contains::Contains;
use crate::util::commutative_predicate_impl;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

/// Defines whether two sets intersect.
///
/// For these two sets is there at least one
/// element which is contained in each?
///
/// Intersects is commutative.
///
/// # Example
///
/// ```
/// use intervalsets::Interval;
/// use intervalsets::Intersects;
///
/// let interval = Interval::closed(10, 20);
/// if interval.intersects(&Interval::closed_unbound(15)) {
///     // true: do something
/// }
/// ```
pub trait Intersects<Rhs = Self> {
    fn intersects(&self, rhs: &Rhs) -> bool;

    fn is_disjoint_from(&self, rhs: &Rhs) -> bool {
        !self.intersects(rhs)
    }
}

impl<T: Numeric> Intersects<Self> for FiniteInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.map_or::<bool>(false, |l1, r1| {
            rhs.map_or::<bool>(false, |l2, r2| {
                l1.contains(Side::Left, &r2.value)
                    && l2.contains(Side::Left, &r1.value)
                    && r1.contains(Side::Right, &l1.value)
                    && r2.contains(Side::Right, &l1.value)
            })
        })
    }
}

impl<T: Numeric> Intersects<FiniteInterval<T>> for HalfInterval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(&left.value) || self.contains(&right.value)
        })
    }
}

impl<T: Numeric> Intersects<Self> for HalfInterval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        let lhs = self;
        lhs.contains(&rhs.ival.value) || rhs.contains(&lhs.ival.value)
    }
}

impl<T: Numeric> Intersects<FiniteInterval<T>> for Interval<T> {
    fn intersects(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Infinite => *rhs != FiniteInterval::Empty,
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Finite(lhs) => lhs.intersects(rhs),
        }
    }
}

impl<T: Numeric> Intersects<HalfInterval<T>> for Interval<T> {
    fn intersects(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.intersects(rhs),
            Self::Finite(lhs) => rhs.intersects(lhs),
        }
    }
}

impl<T: Numeric> Intersects<Self> for Interval<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        match self {
            Self::Infinite => *rhs != FiniteInterval::Empty.into(),
            Self::Half(lhs) => rhs.intersects(lhs),
            Self::Finite(lhs) => rhs.intersects(lhs),
        }
    }
}

commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, HalfInterval<T>);
commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, Interval<T>);
commutative_predicate_impl!(Intersects, intersects, HalfInterval<T>, Interval<T>);

macro_rules! interval_set_intersects_impl {
    ($t_rhs:ty) => {
        impl<T: Numeric> Intersects<$t_rhs> for IntervalSet<T> {
            fn intersects(&self, rhs: &$t_rhs) -> bool {
                self.intervals.iter().any(|subset| subset.intersects(rhs))
            }
        }
    };
}

interval_set_intersects_impl!(FiniteInterval<T>);
commutative_predicate_impl!(Intersects, intersects, FiniteInterval<T>, IntervalSet<T>);
interval_set_intersects_impl!(HalfInterval<T>);
commutative_predicate_impl!(Intersects, intersects, HalfInterval<T>, IntervalSet<T>);
interval_set_intersects_impl!(Interval<T>);
commutative_predicate_impl!(Intersects, intersects, Interval<T>, IntervalSet<T>);

impl<T: Numeric> Intersects<Self> for IntervalSet<T> {
    fn intersects(&self, rhs: &Self) -> bool {
        self.intervals.iter().any(|lhs| rhs.intersects(lhs))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_finite_intersects() {
        assert!(Interval::open(0, 10).intersects(&Interval::open(5, 15)));

        assert!(!Interval::open(0, 10).intersects(&Interval::closed(10, 20)));
    }

    #[test]
    fn test_set_set_intersects() {
        let a = IntervalSet::new_unchecked(vec![
            Interval::unbound_open(0.0),
            Interval::closed(100.0, 110.0),
            Interval::open(1000.0, 1100.0),
        ]);
        let b = IntervalSet::new_unchecked(vec![
            Interval::open(10.0, 20.0),     // no
            Interval::closed(110.0, 120.0), // [110.0, 110.0]
        ]);

        assert!(a.intersects(&b));
    }
}

use crate::empty::MaybeEmpty;
use crate::ival::Side;
use crate::numeric::Domain;
use crate::{EBounds, FiniteInterval, HalfBounded, Interval, IntervalSet};

/// Defines whether a set fully contains another.
///
/// For our purposes a point is the singleton set [T].
///
/// A contains B if and only if
/// for every element x of B,
/// x is also an element of A.
///
/// Contains is not commutative.
///
/// # Example
/// ```
/// use intervalsets::Interval;
/// use intervalsets::Contains;
///
/// let A = Interval::open(0, 10);
/// if A.contains(&10) {
///     // false: oops
/// }
/// if A.contains(&Interval::open(0, 10)) {
///     // true: do the thing, zhu li!
/// }
/// ```
pub trait Contains<Rhs> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T: Domain> Contains<T> for FiniteInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Empty => false,
            Self::FullyBounded(left, right) => {
                left.contains(Side::Left, rhs) && right.contains(Side::Right, rhs)
            }
        }
    }
}

impl<T: Domain> Contains<Self> for FiniteInterval<T> {
    /// Check if this interval fully contains the other
    fn contains(&self, rhs: &Self) -> bool {
        self.map_or(false, |left_out, right_out| {
            rhs.map_or(false, |left_in, right_in| {
                left_out.contains(Side::Left, &left_in.value)
                    && right_out.contains(Side::Right, &right_in.value)
            })
        })

        /*
        I'm curious to bench mark the two of these and see if there is any difference

        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => match rhs {
                Self::Empty => false,
                Self::NonZero(a, b) => {
                    left.contains(Side::Left, &a.value)
                        && right.contains(Side::Right, &b.value)
                }
            },
        }*/
    }
}

impl<T> Contains<HalfBounded<T>> for FiniteInterval<T> {
    /// A FiniteInterval can never contain a HalfInterval
    fn contains(&self, _: &HalfBounded<T>) -> bool {
        false
    }
}

impl<T: Domain> Contains<EBounds<T>> for FiniteInterval<T> {
    fn contains(&self, rhs: &EBounds<T>) -> bool {
        match rhs {
            EBounds::Unbounded => false,
            EBounds::Half(interval) => self.contains(interval),
            EBounds::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: Domain> Contains<T> for HalfBounded<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.ival.contains(self.side, rhs)
    }
}

impl<T: Domain> Contains<Self> for HalfBounded<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(&rhs.ival.value)
    }
}

impl<T: Domain> Contains<FiniteInterval<T>> for HalfBounded<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(&left.value) && self.contains(&right.value)
        })
    }
}

impl<T: Domain> Contains<EBounds<T>> for HalfBounded<T> {
    fn contains(&self, rhs: &EBounds<T>) -> bool {
        match rhs {
            EBounds::Unbounded => false,
            EBounds::Half(interval) => self.contains(interval),
            EBounds::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: Domain> Contains<T> for EBounds<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Unbounded => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: Domain> Contains<FiniteInterval<T>> for EBounds<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Unbounded => !rhs.is_empty(),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: Domain> Contains<HalfBounded<T>> for EBounds<T> {
    fn contains(&self, rhs: &HalfBounded<T>) -> bool {
        match self {
            Self::Unbounded => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: Domain> Contains<Self> for EBounds<T> {
    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Unbounded => match rhs {
                Self::Unbounded => true, // still not sure?
                Self::Half(interval) => self.contains(interval),
                Self::Finite(interval) => self.contains(interval),
            },
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: Domain> Contains<T> for Interval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.0.contains(rhs)
    }
}

impl<T: Domain> Contains<Self> for Interval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.0.contains(&rhs.0)
    }
}

impl<T: Domain> Contains<IntervalSet<T>> for Interval<T> {
    fn contains(&self, rhs: &IntervalSet<T>) -> bool {
        rhs.intervals.iter().all(|subset| self.contains(&subset))
    }
}

macro_rules! interval_set_contains_impl {
    ($t_rhs:ty) => {
        impl<T: Domain> Contains<$t_rhs> for IntervalSet<T> {
            fn contains(&self, rhs: &$t_rhs) -> bool {
                self.intervals.iter().any(|subset| subset.contains(rhs))
            }
        }
    };
}

interval_set_contains_impl!(T);
interval_set_contains_impl!(Interval<T>);

impl<T: Domain> Contains<Self> for IntervalSet<T> {
    fn contains(&self, rhs: &Self) -> bool {
        rhs.intervals.iter().all(|subset| self.contains(subset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn test_finite_contains_integer(x: i8) {
        let iv = EBounds::open(-100, 100);
        assert_eq!(iv.contains(&x), -100 < x && x < 100);
    }

    #[quickcheck]
    fn test_finite_contains_float(x: f32) {
        let iv = EBounds::closed(-100.0, 100.0);
        assert_eq!(iv.contains(&x), -100.0 < x && x < 100.0);
    }

    #[quickcheck]
    fn test_half_contains_integer(x: i8) {
        let left = EBounds::unbound_closed(0);
        assert_eq!(left.contains(&x), x <= 0);

        let right = EBounds::closed_unbound(0);
        assert_eq!(right.contains(&x), x >= 0);
    }

    #[quickcheck]
    fn test_half_contains_float(x: f32) {
        let left = EBounds::unbound_closed(0.0);
        assert_eq!(left.contains(&x), x <= 0.0);

        let right = EBounds::closed_unbound(0.0);
        assert_eq!(right.contains(&x), x >= 0.0);
    }

    #[quickcheck]
    fn test_infinite_contains_float(x: f32) {
        let iv = EBounds::unbound();
        assert!(iv.contains(&x));
    }

    #[quickcheck]
    fn test_finite_finite_integer_contains(a: i8, b: i8) {
        let interval = EBounds::closed(-50, 50);
        let candidate = EBounds::closed(a, b);

        assert_eq!(interval.contains(&candidate), a <= b && -50 <= a && b <= 50)
    }

    #[quickcheck]
    fn test_finite_finite_float_contains(a: f32, b: f32) {
        let interval = EBounds::open(-100.0, 100.0);
        let candidate = EBounds::open(a, b);

        assert_eq!(
            interval.contains(&candidate),
            a < b && -100.0 < a && b < 100.0
        )
    }

    #[quickcheck]
    fn test_set_contains_f32(a: f32) {
        let interval = IntervalSet::<f32>::new_unchecked(vec![
            EBounds::unbound_open(-100000.0),
            EBounds::open(-100.0, 100.0),
            EBounds::open_unbound(100000.0),
        ]);

        assert_eq!(
            interval.contains(&a),
            a < -100000.0 || (-100.0 < a && a < 100.0) || 100000.0 < a
        );
    }

    #[test]
    fn test_set_contains_set() {
        let a = IntervalSet::new_unchecked(vec![
            EBounds::open(-1000.0, 1000.0),
            EBounds::open(3000.0, 4000.0),
        ]);

        assert!(a.contains(&IntervalSet::new_unchecked(vec![
            EBounds::open(0.0, 100.0),
            EBounds::open(3100.0, 3200.0),
        ])));
    }
}

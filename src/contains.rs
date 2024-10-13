use crate::empty::MaybeEmpty;
use crate::ival::Side;
use crate::{FiniteInterval, HalfInterval, Interval, IntervalSet};

/// A trait to determine whether one item fully contains another.
/// Contains is not associative.
pub trait Contains<Rhs> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T: PartialOrd> Contains<T> for FiniteInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Empty => false,
            Self::NonZero(left, right) => {
                left.contains(Side::Left, rhs) && right.contains(Side::Right, rhs)
            }
        }
    }
}

impl<T: PartialOrd> Contains<Self> for FiniteInterval<T> {
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

impl<T> Contains<HalfInterval<T>> for FiniteInterval<T> {
    /// A FiniteInterval can never contain a HalfInterval
    fn contains(&self, _: &HalfInterval<T>) -> bool {
        false
    }
}

impl<T: PartialOrd> Contains<Interval<T>> for FiniteInterval<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        match rhs {
            Interval::Infinite => false,
            Interval::Half(interval) => self.contains(interval),
            Interval::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: PartialOrd> Contains<T> for HalfInterval<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.ival.contains(self.side, rhs)
    }
}

impl<T: PartialOrd> Contains<Self> for HalfInterval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        self.side == rhs.side && self.contains(&rhs.ival.value)
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        rhs.map_or(false, |left, right| {
            self.contains(&left.value) && self.contains(&right.value)
        })
    }
}

impl<T: PartialOrd> Contains<Interval<T>> for HalfInterval<T> {
    fn contains(&self, rhs: &Interval<T>) -> bool {
        match rhs {
            Interval::Infinite => false,
            Interval::Half(interval) => self.contains(interval),
            Interval::Finite(interval) => self.contains(interval),
        }
    }
}

impl<T: PartialOrd> Contains<T> for Interval<T> {
    fn contains(&self, rhs: &T) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<FiniteInterval<T>> for Interval<T> {
    fn contains(&self, rhs: &FiniteInterval<T>) -> bool {
        match self {
            Self::Infinite => !rhs.is_empty(),
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<HalfInterval<T>> for Interval<T> {
    fn contains(&self, rhs: &HalfInterval<T>) -> bool {
        match self {
            Self::Infinite => true,
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

impl<T: PartialOrd> Contains<Self> for Interval<T> {
    fn contains(&self, rhs: &Self) -> bool {
        match self {
            Self::Infinite => match rhs {
                Self::Infinite => true, // still not sure?
                Self::Half(interval) => self.contains(interval),
                Self::Finite(interval) => self.contains(interval),
            },
            Self::Half(lhs) => lhs.contains(rhs),
            Self::Finite(lhs) => lhs.contains(rhs),
        }
    }
}

////////////////////////////////////////

impl<T: PartialOrd> Contains<T> for IntervalSet<T> {
    fn contains(&self, rhs: &T) -> bool {
        self.intervals.iter().any(|subset| subset.contains(rhs))
    }
}

// todo: other interval set conains

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn test_finite_contains_integer(x: i8) {
        let iv = Interval::open(-100, 100);
        assert_eq!(iv.contains(&x), -100 < x && x < 100);
    }

    #[quickcheck]
    fn test_finite_contains_float(x: f32) {
        let iv = Interval::closed(-100.0, 100.0);
        assert_eq!(iv.contains(&x), -100.0 < x && x < 100.0);
    }

    #[quickcheck]
    fn test_half_contains_integer(x: i8) {
        let left = Interval::unbound_closed(0);
        assert_eq!(left.contains(&x), x <= 0);

        let right = Interval::closed_unbound(0);
        assert_eq!(right.contains(&x), x >= 0);
    }

    #[quickcheck]
    fn test_half_contains_float(x: f32) {
        let left = Interval::unbound_closed(0.0);
        assert_eq!(left.contains(&x), x <= 0.0);

        let right = Interval::closed_unbound(0.0);
        assert_eq!(right.contains(&x), x >= 0.0);
    }

    #[quickcheck]
    fn test_infinite_contains_float(x: f32) {
        let iv = Interval::unbound();
        assert!(iv.contains(&x));
    }

    #[quickcheck]
    fn test_finite_finite_integer_contains(a: i8, b: i8) {
        let interval = Interval::closed(-50, 50);
        let candidate = Interval::closed(a, b);

        assert_eq!(interval.contains(&candidate), a <= b && -50 <= a && b <= 50)
    }

    #[quickcheck]
    fn test_finite_finite_float_contains(a: f32, b: f32) {
        let interval = Interval::open(-100.0, 100.0);
        let candidate = Interval::open(a, b);

        assert_eq!(
            interval.contains(&candidate),
            a < b && -100.0 < a && b < 100.0
        )
    }
    // TODO: plenty of other cases
}

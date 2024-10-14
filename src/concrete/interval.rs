use crate::ival::{Bound, IVal, Side};
use crate::numeric::Numeric;
use crate::{FiniteInterval, HalfInterval};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interval<T> {
    /// (a, a) = (a, a] = [a, a) = Empty { x not in T }
    /// [a, a] = NonZero { x in T |    x = a    }
    /// (a, b) = NonZero { x in T | a <  x <  b }
    /// (a, b] = NonZero { x in T | a <  x <= b }
    /// [a, b) = NonZero { x in T | a <= x <  b }
    /// [a, b] = NonZero { x in T | a <= x <= b }
    Finite(FiniteInterval<T>),

    /// (a, ->) = Left  { x in T | a <  x      }
    /// [a, ->) = Left  { x in T | a <= x      }
    /// (<-, b) = Right { x in T |      x < b  }
    /// (<-, b] = Right { x in T |      x <= b }
    Half(HalfInterval<T>),

    /// {<-, ->) = { x in T }
    Infinite,
}

impl<T: Numeric> Interval<T> {
    /// {} = {x | x not in T }
    pub fn empty() -> Self {
        FiniteInterval::Empty.into()
    }

    /// (a, b) = { x in T | a < x < b }
    pub fn open(left: T, right: T) -> Self {
        FiniteInterval::new(IVal::new(Bound::Open, left), IVal::new(Bound::Open, right)).into()
    }

    /// [a, b] = { x in T | a <= x <= b }
    pub fn closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Closed, right),
        )
        .into()
    }

    /// (a, b] = { x in T | a < x <= b }
    pub fn open_closed(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Open, left),
            IVal::new(Bound::Closed, right),
        )
        .into()
    }

    /// [a, b) = { x in T | a <= x < b }
    pub fn closed_open(left: T, right: T) -> Self {
        FiniteInterval::new(
            IVal::new(Bound::Closed, left),
            IVal::new(Bound::Open, right),
        )
        .into()
    }

    // (<-, b) = { x in T | x < b }
    pub fn unbound_open(right: T) -> Self {
        HalfInterval::new(Side::Right, IVal::new(Bound::Open, right)).into()
    }

    /// (<-, b] = { x in T | x <= b }
    pub fn unbound_closed(right: T) -> Self {
        HalfInterval::new(Side::Right, IVal::new(Bound::Closed, right)).into()
    }

    /// (a, ->) = { x in T | a < x }
    pub fn open_unbound(left: T) -> Self {
        HalfInterval::new(Side::Left, IVal::new(Bound::Open, left)).into()
    }

    /// [a, ->) = {x in T | a <= x }
    pub fn closed_unbound(left: T) -> Self {
        HalfInterval::new(Side::Left, IVal::new(Bound::Closed, left)).into()
    }

    pub fn unbound() -> Self {
        Self::Infinite
    }

    pub fn lval_unchecked(&self) -> &T {
        match self {
            Self::Finite(interval) => interval.lval_unchecked(),
            Self::Half(interval) => interval.lval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }

    pub fn rval_unchecked(&self) -> &T {
        match self {
            Self::Finite(interval) => interval.rval_unchecked(),
            Self::Half(interval) => interval.rval_unchecked(),
            _ => panic!("left bound of interval is not in T"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pred::contains::Contains;

    use super::*;

    #[quickcheck]
    fn test_half_interval_contains_f64(x: f64) {
        let interval: Interval<f64> = Interval::unbound_open(0.0);
        assert_eq!(interval.contains(&x), x < 0.0);

        let interval: Interval<f64> = Interval::unbound_closed(1.0);
        assert_eq!(interval.contains(&x), x <= 1.0);

        let interval: Interval<f64> = Interval::open_unbound(0.0);
        assert_eq!(interval.contains(&x), x > 0.0);

        let interval: Interval<f64> = Interval::closed_unbound(1.0);
        assert_eq!(interval.contains(&x), x >= 1.0);
    }

    #[quickcheck]
    fn test_half_interval_contains_u64(x: u64) {
        let interval: Interval<u64> = Interval::unbound_open(100);
        assert_eq!(interval.contains(&x), x < 100);

        let interval: Interval<u64> = Interval::unbound_closed(100);
        assert_eq!(interval.contains(&x), x <= 100);

        let interval: Interval<u64> = Interval::open_unbound(100);
        assert_eq!(interval.contains(&x), x > 100);
    }

    /*

    #[quickcheck]
    fn test_half_interval_complement_i64(x: i64) {
        let interval: Interval<i64> = Interval::closed_unbound(0);
        let complement = &interval.complement()[0];

        assert_eq!(interval.contains(&x), !complement.contains(&x));
    }

    */
}

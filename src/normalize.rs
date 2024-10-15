use crate::ival::Side;
use crate::numeric::Domain;
use crate::{FiniteInterval, HalfInterval, Interval};

/// Normalize an interval so that there is only one
/// standard representation for a give set.
///
/// (0, 10) represents the same integers as [1, 9]
/// and the standard is to normalize to closed sets.
///
/// The receiver on this trait is NOT a reference type
/// because we want to consume any non-normalized
/// values rather than leaving them lying around.
#[allow(dead_code)]
pub(crate) trait Normalize {
    fn normalized(self) -> Self;
}

impl<T: Domain> Normalize for FiniteInterval<T> {
    fn normalized(self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::NonZero(left, right) => {
                Self::new(left.normalized(Side::Left), right.normalized(Side::Right))
            }
        }
    }
}

impl<T: Domain> Normalize for HalfInterval<T> {
    fn normalized(self) -> Self {
        Self::new(self.side, self.ival.normalized(self.side))
    }
}

impl<T: Domain> Normalize for Interval<T> {
    fn normalized(self) -> Self {
        match self {
            Self::Infinite => Self::Infinite,
            Self::Half(interval) => {
                Self::Half(interval.normalized())
                /*let norm = interval.normalized();
                if T::numeric_set() == DomainSet::Natural && norm.side == Side::Right {
                    Self::Finite(FiniteInterval::new_unchecked(IVal::new(Bound::Closed, T::zero()), norm.ival))
                } else {
                    Self::Half(norm)
                }*/
            }
            Self::Finite(interval) => Self::Finite(interval.normalized()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_integers() {
        //let interval = Interval::open(50.0, 60.0);
        //let foo = interval.normalized();

        assert_eq!(Interval::open(0, 10).normalized(), Interval::closed(1, 9));
        assert_eq!(
            Interval::open_closed(0, 10).normalized(),
            Interval::closed(1, 10)
        );
        assert_eq!(
            Interval::unbound_open(5 as i8).normalized(),
            Interval::unbound_closed(4 as i8)
        );
        assert_eq!(
            Interval::unbound_closed(5 as i8).normalized(),
            Interval::unbound_closed(5 as i8)
        );
        assert_eq!(
            Interval::open_unbound(5 as i8).normalized(),
            Interval::closed_unbound(6 as i8)
        );
        assert_eq!(
            Interval::closed(0, 10).normalized(),
            Interval::closed(0, 10)
        );
    }

    #[test]
    fn test_normalized_reals() {
        let interval = Interval::open(0.0, 50.0);
        assert_eq!(interval.clone().normalized(), interval);
    }
}

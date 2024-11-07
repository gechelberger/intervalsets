use core::ops::{Range, RangeFrom};

use crate::bound::ord::{OrdBound, OrdBoundFinite, OrdBoundPair};
use crate::bound::{BoundType, FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, StackSet};
use crate::Factory;

impl<T: Domain> TryFrom<(T, T)> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: (T, T)) -> Result<Self, Error> {
        FiniteInterval::new(FiniteBound::open(value.0), FiniteBound::open(value.1))
    }
}

impl<T: Domain> TryFrom<[T; 2]> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: [T; 2]) -> Result<Self, Error> {
        let mut iter = value.into_iter();
        FiniteInterval::new(
            FiniteBound::closed(iter.next().unwrap()),
            FiniteBound::closed(iter.next().unwrap()),
        )
    }
}

impl<T: Domain> TryFrom<Range<T>> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: Range<T>) -> Result<Self, Self::Error> {
        FiniteInterval::new(
            FiniteBound::closed(value.start),
            FiniteBound::open(value.end),
        )
    }
}

impl<T> From<FiniteInterval<T>> for EnumInterval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T> TryFrom<EnumInterval<T>> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: EnumInterval<T>) -> Result<Self, Self::Error> {
        match value {
            EnumInterval::Finite(inner) => Ok(inner),
            _ => Err(Error::BoundsMismatchError),
        }
    }
}

impl<T> From<RangeFrom<T>> for HalfInterval<T> {
    fn from(value: RangeFrom<T>) -> Self {
        HalfInterval::left(FiniteBound::closed(value.start))
    }
}

impl<T> From<HalfInterval<T>> for EnumInterval<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::Half(value)
    }
}

impl<T> TryFrom<EnumInterval<T>> for HalfInterval<T> {
    type Error = Error;
    fn try_from(value: EnumInterval<T>) -> Result<Self, Self::Error> {
        match value {
            EnumInterval::Half(inner) => Ok(inner),
            _ => Err(Error::BoundsMismatchError),
        }
    }
}

impl<T: Ord> From<EnumInterval<T>> for StackSet<T> {
    fn from(value: EnumInterval<T>) -> Self {
        Self::new([value])
    }
}

impl<T: Ord> From<FiniteInterval<T>> for StackSet<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        StackSet::from(EnumInterval::from(value))
    }
}

impl<T: Ord> From<HalfInterval<T>> for StackSet<T> {
    fn from(value: HalfInterval<T>) -> Self {
        StackSet::from(EnumInterval::from(value))
    }
}

impl<T> From<FiniteInterval<T>> for OrdBoundPair<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        match value {
            FiniteInterval::Empty => OrdBoundPair::empty(),
            FiniteInterval::Bounded(lhs, rhs) => {
                OrdBoundPair::new(lhs.into_ord(Side::Left), rhs.into_ord(Side::Right))
            }
        }
    }
}

impl<T> From<HalfInterval<T>> for OrdBoundPair<T> {
    fn from(value: HalfInterval<T>) -> Self {
        let ord_bound = value.bound.into_ord(value.side);
        match value.side {
            Side::Left => OrdBoundPair::new(ord_bound, OrdBound::RightUnbounded),
            Side::Right => OrdBoundPair::new(OrdBound::LeftUnbounded, ord_bound),
        }
    }
}

impl<T> From<EnumInterval<T>> for OrdBoundPair<T> {
    fn from(value: EnumInterval<T>) -> Self {
        match value {
            EnumInterval::Finite(inner) => inner.into(),
            EnumInterval::Half(inner) => inner.into(),
            EnumInterval::Unbounded => {
                OrdBoundPair::new(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
            }
        }
    }
}

impl<T> From<StackSet<T>> for OrdBoundPair<T> {
    fn from(value: StackSet<T>) -> Self {
        let mut intervals = value.into_raw();
        match intervals.len() {
            0 => Self::empty(),
            1 => intervals.remove(0).into(),
            _ => {
                let first = intervals.swap_remove(0);
                let last = intervals.swap_remove(0);
                let (left, _) = OrdBoundPair::from(first).into_raw();
                let (_, right) = OrdBoundPair::from(last).into_raw();
                OrdBoundPair::new(left, right)
            }
        }
    }
}

impl From<OrdBoundFinite> for BoundType {
    fn from(value: OrdBoundFinite) -> Self {
        match value {
            OrdBoundFinite::Closed => BoundType::Closed,
            _ => BoundType::Open,
        }
    }
}

impl<T> TryFrom<OrdBound<T>> for FiniteBound<T> {
    type Error = Error;
    fn try_from(value: OrdBound<T>) -> Result<Self, Self::Error> {
        match value {
            OrdBound::Finite(value, case) => {
                let bound_type = BoundType::from(case);
                Ok(FiniteBound::new(bound_type, value))
            }
            _ => Err(Error::BoundsMismatchError),
        }
    }
}

impl<T: Domain> TryFrom<OrdBoundPair<T>> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        let (left, right) = value.into_raw();
        let left = FiniteBound::try_from(left)?;
        let right = FiniteBound::try_from(right)?;
        Self::new(left, right)
    }
}

impl<T: Domain> From<OrdBoundPair<T>> for EnumInterval<T> {
    fn from(value: OrdBoundPair<T>) -> Self {
        let (left, right) = value.into_raw();
        match (left, right) {
            (OrdBound::LeftUnbounded, OrdBound::LeftUnbounded) => Self::empty(),
            (OrdBound::LeftUnbounded, OrdBound::RightUnbounded) => Self::Unbounded,
            (OrdBound::LeftUnbounded, OrdBound::Finite(r_val, r_ord)) => {
                let r_bound = FiniteBound::new(r_ord.into(), r_val);
                Self::half_bounded(Side::Right, r_bound)
            }
            (OrdBound::Finite(l_val, l_ord), OrdBound::RightUnbounded) => {
                let l_bound = FiniteBound::new(l_ord.into(), l_val);
                Self::half_bounded(Side::Left, l_bound)
            }
            (OrdBound::Finite(l_val, l_ord), OrdBound::Finite(r_val, r_ord)) => {
                let l_bound = FiniteBound::new(l_ord.into(), l_val);
                let r_bound = FiniteBound::new(r_ord.into(), r_val);
                // SAFETY: FiniteInterval invariants <=> OrdBoundPair invariants
                unsafe { Self::Finite(FiniteInterval::new_unchecked(l_bound, r_bound)) }
            }
            _ => panic!("OrdBoundPair invariants violated"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::Factory;

    #[test]
    fn test_convert_to_finite() -> Result<(), Error> {
        assert_eq!(FiniteInterval::closed(0, 10), [0, 10].try_into()?);
        assert_eq!(FiniteInterval::open(0, 10), (0, 10).try_into()?);

        Ok(())
    }
}

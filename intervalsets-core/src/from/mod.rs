mod ord;
mod range;
mod try_from;

use crate::bound::ord::{OrdBound, OrdBoundPair};
use crate::bound::{FiniteBound, Side};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T> From<()> for FiniteInterval<T> {
    fn from((): ()) -> Self {
        Self::empty()
    }
}

impl<T: Element> From<(T, T)> for FiniteInterval<T> {
    fn from(value: (T, T)) -> Self {
        Self::new(FiniteBound::open(value.0), FiniteBound::open(value.1))
    }
}

impl<T: Element + Clone> From<&(T, T)> for FiniteInterval<T> {
    fn from(value: &(T, T)) -> Self {
        Self::new(
            FiniteBound::open(value.0.clone()),
            FiniteBound::open(value.1.clone()),
        )
    }
}

impl<T: Element> From<(T, T)> for EnumInterval<T> {
    fn from(value: (T, T)) -> Self {
        EnumInterval::from(FiniteInterval::from(value))
    }
}

impl<T> From<()> for EnumInterval<T> {
    fn from(value: ()) -> Self {
        EnumInterval::from(FiniteInterval::from(value))
    }
}

impl<T: Element> From<[T; 2]> for FiniteInterval<T> {
    fn from(value: [T; 2]) -> Self {
        let mut iter = value.into_iter();
        FiniteInterval::new(
            FiniteBound::closed(iter.next().unwrap()),
            FiniteBound::closed(iter.next().unwrap()),
        )
    }
}

impl<T: Element + Clone> From<&[T; 2]> for FiniteInterval<T> {
    fn from(value: &[T; 2]) -> Self {
        FiniteInterval::from(value.clone())
    }
}

impl<T: Element> From<[T; 2]> for EnumInterval<T> {
    fn from(value: [T; 2]) -> Self {
        EnumInterval::from(FiniteInterval::from(value))
    }
}

impl<T> From<FiniteInterval<T>> for EnumInterval<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::Finite(value)
    }
}

impl<T> From<HalfInterval<T>> for EnumInterval<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::Half(value)
    }
}

impl<T> From<FiniteInterval<T>> for OrdBoundPair<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        match value.into_raw() {
            None => OrdBoundPair::empty(),
            Some((lhs, rhs)) => {
                OrdBoundPair::new(lhs.into_ord(Side::Left), rhs.into_ord(Side::Right))
            }
        }
    }
}

impl<'a, T> From<&'a FiniteInterval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a FiniteInterval<T>) -> Self {
        match value.view_raw() {
            None => OrdBoundPair::empty(),
            Some((lhs, rhs)) => OrdBoundPair::new(lhs.ord(Side::Left), rhs.ord(Side::Right)),
        }
    }
}

impl<T> From<HalfInterval<T>> for OrdBoundPair<T> {
    fn from(value: HalfInterval<T>) -> Self {
        match value.side() {
            Side::Left => OrdBoundPair::new(value.into_ord_bound(), OrdBound::RightUnbounded),
            Side::Right => OrdBoundPair::new(OrdBound::LeftUnbounded, value.into_ord_bound()),
        }
    }
}

impl<'a, T> From<&'a HalfInterval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a HalfInterval<T>) -> Self {
        let ord_bound = value.ord_bound();
        match value.side() {
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

impl<'a, T> From<&'a EnumInterval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a EnumInterval<T>) -> Self {
        match value {
            EnumInterval::Finite(inner) => inner.into(),
            EnumInterval::Half(inner) => inner.into(),
            EnumInterval::Unbounded => {
                OrdBoundPair::new(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
            }
        }
    }
}

/*
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

impl<T: Element> TryFrom<OrdBoundPair<T>> for FiniteInterval<T> {
    type Error = Error;
    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        let (left, right) = value.into_raw();
        let left = FiniteBound::try_from(left)?;
        let right = FiniteBound::try_from(right)?;
        Self::new(left, right)
    }
}*/

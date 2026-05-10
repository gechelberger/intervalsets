use crate::bound::ord::{OrdBound, OrdBoundPair};
use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Element> TryFrom<OrdBoundPair<T>> for EnumInterval<T> {
    type Error = Error;

    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        // OrdBoundPair::try_new enforces the invariants required here, so the
        // _assume_valid calls below are sound for any in-process pair. The
        // catchall arm is kept as defense in depth and to document the
        // boundary cheaply.
        let interval = match value.into_raw() {
            (OrdBound::LeftUnbounded, OrdBound::LeftUnbounded) => Self::empty(),
            (OrdBound::LeftUnbounded, OrdBound::RightUnbounded) => Self::Unbounded,
            (OrdBound::LeftUnbounded, OrdBound::Finite(rhs)) => Self::Half(
                HalfInterval::new_assume_valid(Side::Right, FiniteBound::from(rhs)),
            ),
            (OrdBound::Finite(lhs), OrdBound::RightUnbounded) => Self::Half(
                HalfInterval::new_assume_valid(Side::Left, FiniteBound::from(lhs)),
            ),
            (OrdBound::Finite(lhs), OrdBound::Finite(rhs)) => {
                let lhs = FiniteBound::from(lhs);
                let rhs = FiniteBound::from(rhs);
                Self::Finite(FiniteInterval::new_assume_valid(lhs, rhs))
            }
            _ => return Err(Error::InvalidBoundPair),
        };

        Ok(interval)
    }
}

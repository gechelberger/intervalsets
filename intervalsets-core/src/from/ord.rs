use crate::bound::ord::{OrdBound, OrdBoundPair};
use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl<T: Element> TryFrom<OrdBoundPair<T>> for EnumInterval<T> {
    type Error = Error;

    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        // OrdBoundPair::try_new validates structural shape and PartialOrd
        // ordering, but does not call `Element::validate` on inner values.
        // FiniteBound::try_from on each FiniteOrdBound runs the
        // Element-level chokepoint, so values like `f64::INFINITY` that
        // pass PartialOrd but fail `validate` surface as `Err` here.
        let interval = match value.into_raw() {
            (OrdBound::LeftUnbounded, OrdBound::LeftUnbounded) => Self::empty(),
            (OrdBound::LeftUnbounded, OrdBound::RightUnbounded) => Self::Unbounded,
            (OrdBound::LeftUnbounded, OrdBound::Finite(rhs)) => Self::Half(
                HalfInterval::new_assume_valid(Side::Right, FiniteBound::try_from(rhs)?),
            ),
            (OrdBound::Finite(lhs), OrdBound::RightUnbounded) => Self::Half(
                HalfInterval::new_assume_valid(Side::Left, FiniteBound::try_from(lhs)?),
            ),
            (OrdBound::Finite(lhs), OrdBound::Finite(rhs)) => {
                let lhs = FiniteBound::try_from(lhs)?;
                let rhs = FiniteBound::try_from(rhs)?;
                Self::Finite(FiniteInterval::new_assume_valid(lhs, rhs))
            }
            _ => return Err(Error::InvalidBoundPair),
        };

        Ok(interval)
    }
}

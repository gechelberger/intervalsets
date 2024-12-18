use crate::bound::ord::{FiniteOrdBound, FiniteOrdBoundKind, OrdBound, OrdBoundPair};
use crate::bound::{BoundType, FiniteBound, Side};
use crate::error::Error;
use crate::numeric::Element;
use crate::{EnumInterval, FiniteInterval, HalfInterval};

impl From<FiniteOrdBoundKind> for BoundType {
    fn from(value: FiniteOrdBoundKind) -> Self {
        match value {
            FiniteOrdBoundKind::Closed => BoundType::Closed,
            FiniteOrdBoundKind::LeftOpen | FiniteOrdBoundKind::RightOpen => BoundType::Open,
        }
    }
}

impl<T> From<FiniteOrdBound<T>> for FiniteBound<T> {
    fn from(value: FiniteOrdBound<T>) -> Self {
        Self::new(BoundType::from(value.1), value.0)
    }
}

impl<T: Element> TryFrom<OrdBoundPair<T>> for EnumInterval<T> {
    type Error = Error;

    fn try_from(value: OrdBoundPair<T>) -> Result<Self, Self::Error> {
        let interval = match value.into_raw() {
            (OrdBound::LeftUnbounded, OrdBound::LeftUnbounded) => Self::empty(),
            (OrdBound::LeftUnbounded, OrdBound::RightUnbounded) => Self::Unbounded,
            (OrdBound::LeftUnbounded, OrdBound::Finite(rhs)) => {
                // SAFETY: Interval invariants <=> OrdBoundPair invariants
                unsafe {
                    Self::Half(HalfInterval::new_unchecked(
                        Side::Right,
                        FiniteBound::from(rhs),
                    ))
                }
            }
            (OrdBound::Finite(lhs), OrdBound::RightUnbounded) => {
                // SAFETY: Interval invariants <=> OrdBoundPair invariants
                unsafe {
                    Self::Half(HalfInterval::new_unchecked(
                        Side::Left,
                        FiniteBound::from(lhs),
                    ))
                }
            }
            (OrdBound::Finite(lhs), OrdBound::Finite(rhs)) => {
                let lhs = FiniteBound::from(lhs);
                let rhs = FiniteBound::from(rhs);
                // SAFETY: FiniteInterval invariants <=> OrdBoundPair invariants
                unsafe { Self::Finite(FiniteInterval::new_unchecked(lhs, rhs)) }
            }
            _ => {
                return Err(Error::InvariantError(
                    "EnumInterval::TryFrom<OrdBoundPair> did not match a valid bitpattern",
                ))
            }
        };

        Ok(interval)
    }
}

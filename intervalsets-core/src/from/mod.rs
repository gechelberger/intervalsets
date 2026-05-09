mod ord;
mod range;
mod try_from;

use crate::bound::ord::{OrdBound, OrdBoundPair};
use crate::bound::Side;
use crate::factory::FiniteFactory;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

impl<T> From<()> for FiniteInterval<T> {
    fn from((): ()) -> Self {
        Self::empty()
    }
}

// Tuple and array conversions are **strict**: a typed value pair is
// not a Range, and crossed input is treated as a producer bug. For
// the coercive (`a > b → empty`) semantic, use a Rust `Range` type
// as the source instead — it natively encodes that. Note: Rust's
// blanket `impl<T, U> TryFrom<U> for T where U: Into<T>` precludes
// providing a custom fallible `TryFrom` alongside `From`. Callers
// wanting fallible construction use the strict factory methods
// directly (`FiniteInterval::try_open(start, end)` etc.) instead of
// `.try_into()`.

impl<T: Element> From<(T, T)> for FiniteInterval<T> {
    /// Strict open-open conversion. Panics on crossed bounds
    /// (`start > end`) or invalid limits (NaN / ±INF).
    fn from(value: (T, T)) -> Self {
        FiniteInterval::open(value.0, value.1)
    }
}

impl<T: Element + Clone> From<&(T, T)> for FiniteInterval<T> {
    fn from(value: &(T, T)) -> Self {
        FiniteInterval::open(value.0.clone(), value.1.clone())
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
    /// Strict closed-closed conversion. Panics on crossed bounds
    /// (`start > end`) or invalid limits (NaN / ±INF).
    fn from(value: [T; 2]) -> Self {
        let [start, end] = value;
        FiniteInterval::closed(start, end)
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

// All conversions below take a validated interval as input, so the resulting
// `OrdBoundPair` already satisfies the invariants and can skip re-validation.

impl<T> From<FiniteInterval<T>> for OrdBoundPair<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        match value.into_raw() {
            None => OrdBoundPair::empty(),
            Some((lhs, rhs)) => {
                OrdBoundPair::new_assume_valid(lhs.into_ord(Side::Left), rhs.into_ord(Side::Right))
            }
        }
    }
}

impl<'a, T> From<&'a FiniteInterval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a FiniteInterval<T>) -> Self {
        match value.view_raw() {
            None => OrdBoundPair::empty(),
            Some((lhs, rhs)) => {
                OrdBoundPair::new_assume_valid(lhs.ord(Side::Left), rhs.ord(Side::Right))
            }
        }
    }
}

impl<T> From<HalfInterval<T>> for OrdBoundPair<T> {
    fn from(value: HalfInterval<T>) -> Self {
        match value.side() {
            Side::Left => {
                OrdBoundPair::new_assume_valid(value.into_ord_bound(), OrdBound::RightUnbounded)
            }
            Side::Right => {
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, value.into_ord_bound())
            }
        }
    }
}

impl<'a, T> From<&'a HalfInterval<T>> for OrdBoundPair<&'a T> {
    fn from(value: &'a HalfInterval<T>) -> Self {
        let ord_bound = value.ord_bound();
        match value.side() {
            Side::Left => OrdBoundPair::new_assume_valid(ord_bound, OrdBound::RightUnbounded),
            Side::Right => OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, ord_bound),
        }
    }
}

impl<T> From<EnumInterval<T>> for OrdBoundPair<T> {
    fn from(value: EnumInterval<T>) -> Self {
        match value {
            EnumInterval::Finite(inner) => inner.into(),
            EnumInterval::Half(inner) => inner.into(),
            EnumInterval::Unbounded => {
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
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
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
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

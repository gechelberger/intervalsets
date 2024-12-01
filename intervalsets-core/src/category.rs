use core::cmp::Ordering::{Equal, Greater, Less};

use crate::bound::ord::FiniteOrdBound;
use crate::bound::Side::{Left, Right};
use crate::numeric::Zero;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Marker if a set or bound contains the 0 element.
#[derive(Debug, PartialEq)]
pub enum MaybeZero {
    Zero,
    NonZero,
}

/// Set Elements Category
#[derive(Debug, PartialEq)]
pub enum ECat {
    /// No elements.
    Empty,

    /// Singleton set of 0.
    Zero,

    /// Set contains negative and positive elements.
    /// If the set is an interval then it must contain 0.
    NegPos,

    /// All set elements are non-negative.
    /// If MaybeZero::Zero, then least element **is** 0.
    Pos(MaybeZero),

    /// All set elements are non-positive.
    /// If MaybeZero::Zero, then greatest element **is** 0.
    Neg(MaybeZero),
}

impl<T: Zero + PartialOrd> FiniteInterval<T> {
    pub fn category(&self) -> ECat {
        let Some((lhs, rhs)) = self.view_raw() else {
            return ECat::Empty;
        };

        let t_zero = T::zero();
        let zero = FiniteOrdBound::closed(&t_zero);
        match lhs.finite_ord(Left).partial_cmp(&zero).unwrap() {
            Greater => ECat::Pos(MaybeZero::NonZero),
            Equal => match rhs.finite_ord(Right).partial_cmp(&zero).unwrap() {
                Greater => ECat::Pos(MaybeZero::Zero),
                Equal => ECat::Zero,
                Less => unreachable!(),
            },
            Less => match rhs.finite_ord(Right).partial_cmp(&zero).unwrap() {
                Greater => ECat::NegPos,
                Equal => ECat::Neg(MaybeZero::Zero),
                Less => ECat::Neg(MaybeZero::NonZero),
            },
        }
    }
}

impl<T: Zero + PartialOrd> HalfInterval<T> {
    pub fn category(&self) -> ECat {
        let t_zero = T::zero();
        let zero = FiniteOrdBound::closed(&t_zero);
        match self.side() {
            Left => match self.finite_ord_bound().partial_cmp(&zero).unwrap() {
                Less => ECat::NegPos,
                Equal => ECat::Pos(MaybeZero::Zero),
                Greater => ECat::Pos(MaybeZero::NonZero),
            },
            Right => match self.finite_ord_bound().partial_cmp(&zero).unwrap() {
                Less => ECat::Neg(MaybeZero::NonZero),
                Equal => ECat::Neg(MaybeZero::Zero),
                Greater => ECat::NegPos,
            },
        }
    }
}

impl<T: Zero + PartialOrd> EnumInterval<T> {
    pub fn category(&self) -> ECat {
        match self {
            Self::Finite(inner) => inner.category(),
            Self::Half(inner) => inner.category(),
            Self::Unbounded => ECat::NegPos,
        }
    }
}

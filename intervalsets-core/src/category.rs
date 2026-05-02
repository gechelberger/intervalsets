use core::cmp::Ordering::{Equal, Greater, Less};

use crate::bound::ord::FiniteOrdBound;
use crate::bound::Side::{Left, Right};
use crate::error::TotalOrderError;
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
    /// Classify this interval relative to zero.
    ///
    /// # Panics
    ///
    /// Panics if a bound value is not comparable to zero (e.g. a NaN
    /// `f32`/`f64` bound). For panic-free classification on
    /// `PartialOrd`-only types, use [`try_category`](Self::try_category).
    /// For `Ord` types this method is infallible.
    pub fn category(&self) -> ECat {
        self.try_category().unwrap()
    }

    /// Classify this interval relative to zero, returning `Err` if a
    /// bound is not comparable to zero (e.g. a NaN float bound).
    pub fn try_category(&self) -> Result<ECat, TotalOrderError> {
        let Some((lhs, rhs)) = self.view_raw() else {
            return Ok(ECat::Empty);
        };

        let t_zero = T::zero();
        let zero = FiniteOrdBound::closed(&t_zero);
        let lhs_ord = lhs
            .finite_ord(Left)
            .partial_cmp(&zero)
            .ok_or(TotalOrderError)?;
        Ok(match lhs_ord {
            Greater => ECat::Pos(MaybeZero::NonZero),
            Equal => {
                let rhs_ord = rhs
                    .finite_ord(Right)
                    .partial_cmp(&zero)
                    .ok_or(TotalOrderError)?;
                match rhs_ord {
                    Greater => ECat::Pos(MaybeZero::Zero),
                    Equal => ECat::Zero,
                    Less => unreachable!(),
                }
            }
            Less => {
                let rhs_ord = rhs
                    .finite_ord(Right)
                    .partial_cmp(&zero)
                    .ok_or(TotalOrderError)?;
                match rhs_ord {
                    Greater => ECat::NegPos,
                    Equal => ECat::Neg(MaybeZero::Zero),
                    Less => ECat::Neg(MaybeZero::NonZero),
                }
            }
        })
    }
}

impl<T: Zero + PartialOrd> HalfInterval<T> {
    /// Classify this interval relative to zero.
    ///
    /// # Panics
    ///
    /// Panics if the bound value is not comparable to zero (e.g. a NaN
    /// `f32`/`f64` bound). For panic-free classification on
    /// `PartialOrd`-only types, use [`try_category`](Self::try_category).
    /// For `Ord` types this method is infallible.
    pub fn category(&self) -> ECat {
        self.try_category().unwrap()
    }

    /// Classify this interval relative to zero, returning `Err` if the
    /// bound is not comparable to zero (e.g. a NaN float bound).
    pub fn try_category(&self) -> Result<ECat, TotalOrderError> {
        let t_zero = T::zero();
        let zero = FiniteOrdBound::closed(&t_zero);
        let ord = self
            .finite_ord_bound()
            .partial_cmp(&zero)
            .ok_or(TotalOrderError)?;
        Ok(match self.side() {
            Left => match ord {
                Less => ECat::NegPos,
                Equal => ECat::Pos(MaybeZero::Zero),
                Greater => ECat::Pos(MaybeZero::NonZero),
            },
            Right => match ord {
                Less => ECat::Neg(MaybeZero::NonZero),
                Equal => ECat::Neg(MaybeZero::Zero),
                Greater => ECat::NegPos,
            },
        })
    }
}

impl<T: Zero + PartialOrd> EnumInterval<T> {
    /// Classify this interval relative to zero.
    ///
    /// # Panics
    ///
    /// Panics if a bound value is not comparable to zero (e.g. a NaN
    /// `f32`/`f64` bound). For panic-free classification on
    /// `PartialOrd`-only types, use [`try_category`](Self::try_category).
    /// For `Ord` types this method is infallible.
    pub fn category(&self) -> ECat {
        self.try_category().unwrap()
    }

    /// Classify this interval relative to zero, returning `Err` if a
    /// bound is not comparable to zero (e.g. a NaN float bound).
    pub fn try_category(&self) -> Result<ECat, TotalOrderError> {
        match self {
            Self::Finite(inner) => inner.try_category(),
            Self::Half(inner) => inner.try_category(),
            Self::Unbounded => Ok(ECat::NegPos),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::FiniteBound;
    use crate::factory::FiniteFactory;

    #[test]
    fn test_try_category_nan() {
        // NaN bounds are "logically invalid" but UB-free; construct via
        // new_assume_valid to bypass the factory's NaN check, then verify
        // try_category surfaces the incomparability as Err.
        let bad = FiniteInterval::new_assume_valid(
            FiniteBound::closed(f32::NAN),
            FiniteBound::closed(0.0),
        );
        assert!(bad.try_category().is_err());
    }

    #[test]
    fn test_category_ord() {
        // For Ord types (i32) try_category is provably infallible.
        let x = FiniteInterval::closed(-5i32, 5i32);
        assert_eq!(x.category(), ECat::NegPos);

        let x = FiniteInterval::closed(0i32, 10i32);
        assert_eq!(x.category(), ECat::Pos(MaybeZero::Zero));

        let x = FiniteInterval::closed(1i32, 10i32);
        assert_eq!(x.category(), ECat::Pos(MaybeZero::NonZero));
    }
}

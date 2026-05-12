use core::cmp::Ordering;

use num_traits::One;

use super::{FiniteInterval, HalfInterval};
use crate::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
use crate::bound::{FiniteBound, SetBounds, Side};
use crate::error::Error;
use crate::numeric::{Element, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "RawEnumInterval<T>"))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))
)]
#[allow(missing_docs)]
pub enum EnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

/// Wire-format mirror of [`EnumInterval`]. The variants hold the
/// already-validated public types, so the `TryFrom` is total.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
#[serde(rename = "EnumInterval")]
#[serde(bound(deserialize = "T: Element + serde::Deserialize<'de>"))]
enum RawEnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

#[cfg(feature = "serde")]
impl<T: Element> From<RawEnumInterval<T>> for EnumInterval<T> {
    fn from(raw: RawEnumInterval<T>) -> Self {
        match raw {
            RawEnumInterval::Finite(inner) => Self::Finite(inner),
            RawEnumInterval::Half(inner) => Self::Half(inner),
            RawEnumInterval::Unbounded => Self::Unbounded,
        }
    }
}

impl<T> EnumInterval<T> {
    /// Creates a new empty EnumInterval.
    pub const fn empty() -> Self {
        Self::Finite(FiniteInterval::empty())
    }
}

impl<T> EnumInterval<T> {
    pub fn is_fully_bounded(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_fully_bounded(),
            _ => false,
        }
    }
}

impl<T> OrdBounded<T> for EnumInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Finite(inner) => inner.ord_bound_pair(),
            Self::Half(inner) => inner.ord_bound_pair(),
            Self::Unbounded => {
                OrdBoundPair::new_assume_valid(OrdBound::LeftUnbounded, OrdBound::RightUnbounded)
            }
        }
    }
}

impl<T> SetBounds<T> for EnumInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        match self {
            Self::Finite(inner) => inner.bound(side),
            Self::Half(inner) => inner.bound(side),
            Self::Unbounded => None,
        }
    }
}

// num_traits::Zero requires Self: Add<Self, Output = Self>; the infix
// Add impl on EnumInterval is sugar over try_add, so T must satisfy
// `TryAdd<Output = T>`. Likewise One requires Self: Mul<Self, Output = Self>,
// so T must satisfy `TryMul<Output = T>`.
impl<T> Zero for EnumInterval<T>
where
    T: Element + Zero + crate::ops::math::TryAdd<Output = T>,
    <T as crate::ops::math::TryAdd>::Error: core::fmt::Debug + Into<Error>,
{
    fn zero() -> Self {
        Self::from(FiniteInterval::<T>::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_zero(),
            _ => false,
        }
    }
}

impl<T> One for EnumInterval<T>
where
    T: Element + Clone + Zero + One + crate::ops::math::TryMul<Output = T>,
    <T as crate::ops::math::TryMul>::Error: core::fmt::Debug + Into<Error>,
{
    fn one() -> Self {
        EnumInterval::from(FiniteInterval::one())
    }
}

impl<T> Default for EnumInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: PartialOrd> PartialOrd for EnumInterval<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.partial_cmp(&rhs)
    }
}

impl<T: Ord> Ord for EnumInterval<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let lhs = self.ord_bound_pair();
        let rhs = rhs.ord_bound_pair();
        lhs.cmp(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::FiniteFactory;

    #[test]
    fn test_set_bounds_trait() {
        let x = EnumInterval::closed(0, 10);

        assert_eq!(x.left().unwrap(), &FiniteBound::closed(0));
        assert_eq!(x.right().unwrap(), &FiniteBound::closed(10));
    }

    #[test]
    fn test_ord_bounded_trait() {
        let x = EnumInterval::closed(0, 10);

        fn by_ref(y: &EnumInterval<i32>) {
            let ob = y.ord_bound_pair();
            assert_eq!(
                ob,
                OrdBoundPair::new(OrdBound::closed(&0), OrdBound::closed(&10))
            );
        }

        fn by_val(y: EnumInterval<i32>) {
            let ob = y.ord_bound_pair();
            assert_eq!(
                ob,
                OrdBoundPair::new(OrdBound::closed(&0), OrdBound::closed(&10))
            );
        }

        by_ref(&x);
        by_val(x);
    }
}

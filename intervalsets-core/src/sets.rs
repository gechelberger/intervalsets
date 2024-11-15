use core::cmp::Ordering::{self, *};

use super::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
use super::bound::{FiniteBound, SetBounds, Side};
use crate::numeric::Domain;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FiniteInterval<T> {
    Empty,
    Bounded(FiniteBound<T>, FiniteBound<T>),
}

impl<T: Domain> FiniteInterval<T> {
    pub fn new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        unsafe { Self::new_norm(lhs.normalized(Side::Left), rhs.normalized(Side::Right)) }
    }

    pub fn new_strict(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Option<Self> {
        match lhs.value().partial_cmp(rhs.value())? {
            Less | Equal => Some(Self::new(lhs, rhs)),
            Greater => None,
        }
    }
}

impl<T: PartialOrd> FiniteInterval<T> {
    /// # Safety
    ///
    /// The user must ensure invariants are satisfied:
    /// 1. discrete bounds are normalized to closed form
    pub unsafe fn new_norm(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        match lhs.value().partial_cmp(rhs.value()) {
            Some(Less) => Self::Bounded(lhs, rhs),
            Some(Equal) if lhs.is_closed() && rhs.is_closed() => Self::Bounded(lhs, rhs),
            _ => Self::Empty,
        }
    }
}

impl<T> FiniteInterval<T> {
    pub const fn empty() -> Self {
        Self::Empty
    }

    /// # Safety
    ///
    /// The user must ensure invariants are satisfied:
    /// 1. lhs <= rhs
    /// 2. discrete bounds are normalized to closed form.
    #[inline]
    pub const unsafe fn new_unchecked(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        Self::Bounded(lhs, rhs)
    }

    pub fn into_raw(self) -> Option<(FiniteBound<T>, FiniteBound<T>)> {
        match self {
            Self::Bounded(lhs, rhs) => Some((lhs, rhs)),
            Self::Empty => None,
        }
    }
}

impl<T> OrdBounded<T> for FiniteInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Empty => OrdBoundPair::empty(),
            Self::Bounded(lhs, rhs) => OrdBoundPair::new(lhs.ord(Side::Left), rhs.ord(Side::Right)),
        }
    }
}

impl<T> SetBounds<T> for FiniteInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        match self {
            Self::Bounded(lhs, rhs) => Some(side.select(lhs, rhs)),
            _ => None,
        }
    }

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     self.map(|lhs, rhs| (Some(lhs), Some(rhs))).ok()
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HalfInterval<T> {
    pub side: Side,
    pub bound: FiniteBound<T>,
}

impl<T: Domain> HalfInterval<T> {
    pub fn new(side: Side, bound: FiniteBound<T>) -> Self {
        Self {
            side,
            bound: bound.normalized(side),
        }
    }

    pub fn left(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Left, bound)
    }

    pub fn right(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Right, bound)
    }
}

impl<T> HalfInterval<T> {
    pub fn into_raw(self) -> (Side, FiniteBound<T>) {
        (self.side, self.bound)
    }
}

impl<T> OrdBounded<T> for HalfInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self.side {
            Side::Left => {
                let left = OrdBound::left(&self.bound);
                OrdBoundPair::new(left, OrdBound::RightUnbounded)
            }
            Side::Right => {
                let right = OrdBound::right(&self.bound);
                OrdBoundPair::new(OrdBound::LeftUnbounded, right)
            }
        }
    }
}

impl<T> SetBounds<T> for HalfInterval<T> {
    fn bound(&self, side: Side) -> Option<&FiniteBound<T>> {
        if self.side == side {
            Some(&self.bound)
        } else {
            None
        }
    }

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     let bounds = match self.side {
    //         Side::Left => (Some(self.bound), None),
    //         Side::Right => (None, Some(self.bound)),
    //     };
    //     Some(bounds)
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

impl<T> EnumInterval<T> {
    pub const fn empty() -> Self {
        Self::Finite(FiniteInterval::Empty)
    }
}

impl<T> OrdBounded<T> for EnumInterval<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self {
            Self::Finite(inner) => inner.ord_bound_pair(),
            Self::Half(inner) => inner.ord_bound_pair(),
            Self::Unbounded => OrdBoundPair::new(OrdBound::LeftUnbounded, OrdBound::RightUnbounded),
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

    // fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
    //     match self {
    //         Self::Finite(inner) => inner.into_bounds(),
    //         Self::Half(inner) => inner.into_bounds(),
    //         Self::Unbounded => Some((None, None)),
    //     }
    // }
}

impl<T> Default for FiniteInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Default for EnumInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

macro_rules! impl_interval_cmp {
    ($($t:ident), +) => {
        $(
            impl<T: PartialOrd> PartialOrd for $t<T> {
                fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
                    let lhs = self.ord_bound_pair();
                    let rhs = rhs.ord_bound_pair();
                    lhs.partial_cmp(&rhs)
                }
            }

            impl<T: Ord> Ord for $t<T> {
                fn cmp(&self, rhs: &Self) -> Ordering {
                    let lhs = self.ord_bound_pair();
                    let rhs = rhs.ord_bound_pair();
                    lhs.cmp(&rhs)
                }
            }
        )+
    }
}

impl_interval_cmp!(FiniteInterval, HalfInterval, EnumInterval);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Factory;

    #[test]
    fn test_set_bounds_trait() {
        let x = EnumInterval::closed(0, 10);

        assert_eq!(x.left().unwrap(), &FiniteBound::closed(0));
        assert_eq!(x.right().unwrap(), &FiniteBound::closed(10));
    }
}

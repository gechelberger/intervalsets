use core::cmp::Ordering;

use const_env::from_env;

use super::bound::ord::{OrdBound, OrdBoundPair, OrdBounded};
use super::bound::{FiniteBound, SetBounds, Side};
use crate::empty::MaybeEmpty;
use crate::error::Error;
use crate::numeric::Domain;

#[from_env("INTERVALSETS_CORE_N")]
const ISET_N: usize = 2;

pub type StackSetStorage<T> = heapless::Vec<T, ISET_N>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FiniteInterval<T> {
    Empty,
    Bounded(FiniteBound<T>, FiniteBound<T>),
}

impl<T: Domain> FiniteInterval<T> {
    pub fn new(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Result<Self, Error> {
        if lhs.value() < rhs.value() {
            Ok(Self::Bounded(
                lhs.normalized(Side::Left),
                rhs.normalized(Side::Right),
            ))
        } else if lhs.value() > rhs.value() {
            Err(Error::InvertedBoundsError)
        } else if lhs.is_open() || rhs.is_open() {
            Ok(Self::Empty)
            //Err(Error::InvertedBoundsError)
        } else {
            Ok(Self::Bounded(lhs, rhs)) // singleton
        }
    }

    /// # Safety
    ///
    /// The user must ensure invariants are satisfied:
    /// 1. lhs <= rhs
    /// 2. discrete bounds are normalized to closed form.
    pub unsafe fn new_unchecked(lhs: FiniteBound<T>, rhs: FiniteBound<T>) -> Self {
        Self::Bounded(lhs, rhs)
    }
}

impl<T> FiniteInterval<T> {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn into_raw(self) -> Option<(FiniteBound<T>, FiniteBound<T>)> {
        match self {
            Self::Bounded(lhs, rhs) => Some((lhs, rhs)),
            Self::Empty => None,
        }
    }

    pub fn map<F, U>(self, func: F) -> Result<U, Error>
    where
        F: FnOnce(FiniteBound<T>, FiniteBound<T>) -> U,
    {
        match self {
            Self::Empty => Err(Error::BoundsMismatchError),
            Self::Bounded(lhs, rhs) => Ok(func(lhs, rhs)),
        }
    }

    pub fn ref_map<'a, F, U>(&'a self, func: F) -> Result<U, Error>
    where
        F: FnOnce(&'a FiniteBound<T>, &'a FiniteBound<T>) -> U,
    {
        match self {
            Self::Empty => Err(Error::BoundsMismatchError),
            Self::Bounded(lhs, rhs) => Ok(func(lhs, rhs)),
        }
    }

    pub fn flat_map<F>(self, func: F) -> Self
    where
        F: FnOnce(FiniteBound<T>, FiniteBound<T>) -> Self,
    {
        self.map(func).unwrap_or(Self::Empty)
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
        self.ref_map(|lhs, rhs| side.select(lhs, rhs)).ok()
    }

    fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
        self.map(|lhs, rhs| (Some(lhs), Some(rhs))).ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HalfInterval<T> {
    pub side: Side,
    pub bound: FiniteBound<T>,
}

impl<T> HalfInterval<T> {
    pub fn new(side: Side, bound: FiniteBound<T>) -> Self {
        Self { side, bound }
    }

    pub fn left(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Left, bound)
    }

    pub fn right(bound: FiniteBound<T>) -> Self {
        Self::new(Side::Right, bound)
    }

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

    fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
        let bounds = match self.side {
            Side::Left => (Some(self.bound), None),
            Side::Right => (None, Some(self.bound)),
        };
        Some(bounds)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EnumInterval<T> {
    Finite(FiniteInterval<T>),
    Half(HalfInterval<T>),
    Unbounded,
}

impl<T> EnumInterval<T> {
    pub fn empty() -> Self {
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

    fn into_bounds(self) -> Option<(Option<FiniteBound<T>>, Option<FiniteBound<T>>)> {
        match self {
            Self::Finite(inner) => inner.into_bounds(),
            Self::Half(inner) => inner.into_bounds(),
            Self::Unbounded => Some((None, None)),
        }
    }
}

// A Set allocated on the stack, with a (low) fixed capacity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackSet<T> {
    intervals: heapless::Vec<EnumInterval<T>, ISET_N>,
}

impl<T, I> FromIterator<I> for StackSet<T>
where
    T: Domain + Ord,
    I: Into<EnumInterval<T>>,
{
    fn from_iter<U: IntoIterator<Item = I>>(iter: U) -> Self {
        Self::new(iter.into_iter().map(|x| x.into()))
    }
}

impl<T: Domain> StackSet<T> {
    pub fn new<I: IntoIterator<Item = EnumInterval<T>>>(iter: I) -> Self {
        let mut intervals: StackSetStorage<_> =
            heapless::Vec::from_iter(iter.into_iter().filter(|x| !x.is_empty()));

        intervals
            .as_mut_slice()
            .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        unsafe { Self::new_unchecked(crate::ops::union::MergeSorted::new(intervals)) }
    }
}

impl<T> StackSet<T> {
    /// # Safety
    ///
    /// The user is responsible for enforcing Set invariants
    /// 1. intervals are properly normalized
    /// 2. no empty intervals
    /// 3. intervals are disjoint
    /// 4. intervals are sorted least to greatest
    pub unsafe fn new_unchecked<I: IntoIterator<Item = EnumInterval<T>>>(iter: I) -> Self {
        Self {
            intervals: heapless::Vec::from_iter(iter),
        }
    }

    pub fn empty() -> Self {
        Self {
            intervals: heapless::Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn expect_interval(mut self) -> Result<EnumInterval<T>, Error> {
        match self.intervals.len() {
            0 => Ok(FiniteInterval::<T>::empty().into()),
            1 => Ok(self.intervals.remove(0)),
            _ => Err(crate::error::Error::MultipleIntervalsError),
        }
    }

    pub fn slice(&self) -> &[EnumInterval<T>] {
        &self.intervals
    }

    pub fn iter(&self) -> impl Iterator<Item = &EnumInterval<T>> {
        self.intervals.iter()
    }

    pub fn into_raw(self) -> heapless::Vec<EnumInterval<T>, ISET_N> {
        self.intervals
    }
}

impl<T> OrdBounded<T> for StackSet<T> {
    fn ord_bound_pair(&self) -> OrdBoundPair<&T> {
        match self.intervals.len() {
            0 => OrdBoundPair::empty(),
            1 => self.intervals[0].ord_bound_pair(),
            _ => {
                let lower = self.intervals.first().unwrap();
                let (lower, _) = lower.ord_bound_pair().into_raw();
                let upper = self.intervals.last().unwrap();
                let (_, upper) = upper.ord_bound_pair().into_raw();
                OrdBoundPair::new(lower, upper)
            }
        }
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

    #[test]
    fn new_stack_set() -> Result<(), Error> {
        let a = EnumInterval::closed(0, 10);
        let b = EnumInterval::closed(5, 15);
        let c = StackSet::from_iter([b, a, EnumInterval::empty()]);
        assert_eq!(c.expect_interval()?, EnumInterval::closed(0, 15));

        Ok(())
    }
}

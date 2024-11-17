use super::adjacent::Adjacent;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::ops::{Contains, Intersects, TryMerge};
use crate::sets::EnumInterval::{self, *};
use crate::sets::FiniteInterval::{self, Bounded};
use crate::sets::HalfInterval;

impl<T: Domain> TryMerge<Self> for FiniteInterval<T> {
    type Output = Self;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.intersects(&rhs) || self.is_adjacent_to(&rhs) {
            let Bounded(lhs_min, lhs_max) = self else {
                unreachable!();
            };

            let Bounded(rhs_min, rhs_max) = rhs else {
                unreachable!();
            };

            // SAFETY: if lhs and rhs satisfy invariants, bounds are normalized,
            // and min(left, right) <= max(left, right).
            let merged = unsafe {
                FiniteInterval::new_unchecked(
                    FiniteBound::take_min(Side::Left, lhs_min, rhs_min),
                    FiniteBound::take_max(Side::Right, lhs_max, rhs_max),
                )
            };

            Some(merged)
        } else if self.is_empty() {
            Some(rhs)
        } else if rhs.is_empty() {
            Some(self)
        } else {
            None
        }
    }
}

impl<T: Domain + Clone> TryMerge<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.intersects(rhs) || self.is_adjacent_to(rhs) {
            let Bounded(lhs_min, lhs_max) = self else {
                unreachable!();
            };

            let Bounded(rhs_min, rhs_max) = rhs else {
                unreachable!();
            };

            // SAFETY: if lhs and rhs satisfy invariants, bounds are normalized,
            // and min(left, right) <= max(left, right).
            let merged = unsafe {
                FiniteInterval::new_unchecked(
                    FiniteBound::min(Side::Left, lhs_min, rhs_min).clone(),
                    FiniteBound::max(Side::Right, lhs_max, rhs_max).clone(),
                )
            };

            Some(merged)
        } else if self.is_empty() {
            Some(rhs.clone())
        } else if rhs.is_empty() {
            Some(self.clone())
        } else {
            None
        }
    }
}

impl<T: Domain> TryMerge<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.into())
            } else {
                Some(rhs.into())
            }
        } else if self.contains(rhs.bound.value())
            || rhs.contains(self.bound.value())
            || self.is_adjacent_to(&rhs)
        {
            // <----](---->
            // <----][---->
            // <----)[---->
            // but not <----)(---->
            Some(EnumInterval::Unbounded)
        } else {
            None
        }
    }
}

impl<T: Domain + Clone> TryMerge<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else if self.contains(rhs.bound.value())
            || rhs.contains(self.bound.value())
            || self.is_adjacent_to(rhs)
        {
            Some(EnumInterval::Unbounded)
        } else {
            None
        }
    }
}

impl<T: Domain> TryMerge<FiniteInterval<T>> for HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        let Bounded(rhs_min, rhs_max) = rhs else {
            return Some(self); // identity: merge with empty
        };

        let n = [&rhs_min, &rhs_max]
            .into_iter()
            .filter(|b| self.contains(b.value()))
            .count();

        if n == 2 {
            Some(self) // finite interval is fully contained
        } else if n == 1 {
            let bound = self.side.select(rhs_min, rhs_max);
            Some(HalfInterval::new(self.side, bound))
        } else {
            let maybe_adjacent = self.side.select(&rhs_max, &rhs_min);
            if self.is_adjacent_to(maybe_adjacent) {
                let bound = self.side.select(rhs_min, rhs_max);
                Some(HalfInterval::new(self.side, bound))
            } else {
                None
            }
        }
    }
}

impl<T: Domain + Clone> TryMerge<&FiniteInterval<T>> for &HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        let Bounded(rhs_min, rhs_max) = rhs else {
            return Some(self.clone());
        };

        let n = [rhs_min, rhs_max]
            .into_iter()
            .filter(|b| self.contains(b.value()))
            .count();

        if n == 2 {
            Some(self.clone())
        } else if n == 1 {
            let bound = self.side.select(rhs_min, rhs_max).clone();
            Some(HalfInterval::new(self.side, bound))
        } else {
            let maybe_adj = self.side.select(rhs_max, rhs_min);
            if self.is_adjacent_to(maybe_adj) {
                let bound = self.side.select(rhs_min, rhs_max).clone();
                Some(HalfInterval::new(self.side, bound))
            } else {
                None
            }
        }
    }
}

impl<T: Domain> TryMerge<HalfInterval<T>> for FiniteInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: HalfInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

impl<T: Domain + Clone> TryMerge<&HalfInterval<T>> for &FiniteInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

impl<T: Domain> TryMerge<FiniteInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Half(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain + Clone> TryMerge<&FiniteInterval<T>> for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        match self {
            Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Half(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain> TryMerge<HalfInterval<T>> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Half(lhs) => lhs.try_merge(rhs),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain + Clone> TryMerge<&HalfInterval<T>> for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: &HalfInterval<T>) -> Option<Self::Output> {
        match self {
            Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
            Half(lhs) => lhs.try_merge(rhs),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain> TryMerge<Self> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        match self {
            Finite(lhs) => rhs.try_merge(lhs),
            Half(lhs) => rhs.try_merge(lhs),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain + Clone> TryMerge for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        match self {
            Finite(lhs) => lhs.try_merge(rhs),
            Half(lhs) => lhs.try_merge(rhs),
            Unbounded => Some(Unbounded),
        }
    }
}

impl<T: Domain> TryMerge<EnumInterval<T>> for FiniteInterval<T> {
    type Output = EnumInterval<T>;
    fn try_merge(self, rhs: EnumInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

impl<T: Domain + Clone> TryMerge<&EnumInterval<T>> for &FiniteInterval<T> {
    type Output = EnumInterval<T>;
    fn try_merge(self, rhs: &EnumInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

impl<T: Domain> TryMerge<EnumInterval<T>> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    fn try_merge(self, rhs: EnumInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

impl<T: Domain + Clone> TryMerge<&EnumInterval<T>> for &HalfInterval<T> {
    type Output = EnumInterval<T>;
    fn try_merge(self, rhs: &EnumInterval<T>) -> Option<Self::Output> {
        rhs.try_merge(self)
    }
}

/// MergeSorted merges intersecting intervals and returns disjoint ones.
pub struct MergeSorted<T: Domain, I: Iterator<Item = EnumInterval<T>>> {
    sorted: core::iter::Peekable<I>,
}

impl<T, I> MergeSorted<T, I>
where
    T: Domain,
    I: Iterator<Item = EnumInterval<T>>,
{
    pub fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = EnumInterval<T>, IntoIter = I>,
    {
        Self {
            sorted: sorted.into_iter().peekable(),
        }
    }
}

impl<T, I> Iterator for MergeSorted<T, I>
where
    T: Domain,
    I: Iterator<Item = EnumInterval<T>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?;

        while let Some(peek) = self.sorted.peek() {
            if !super::mergeable(&current, peek) {
                break;
            }

            let candidate = self.sorted.next().unwrap();
            current = match current.try_merge(candidate) {
                Some(merged) => merged,
                None => unreachable!(),
            };
        }

        Some(current)
    }
}

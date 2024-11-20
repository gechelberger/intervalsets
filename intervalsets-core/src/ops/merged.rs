use super::adjacent::Adjacent;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::ops::{Contains, Intersects};
use crate::sets::EnumInterval::{self, *};
use crate::sets::FiniteInterval::{self, Bounded};
use crate::sets::HalfInterval;

/// Test whether two sets are mergable.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// let y = FiniteInterval::closed(11, 20);
/// assert_eq!(mergeable(&x, &y), true);
///
/// let x = FiniteInterval::closed(0.0, 10.0);
/// let y = FiniteInterval::closed(11.0, 20.0);
/// assert_eq!(mergeable(&x, &y), false);
/// ```
#[inline]
pub fn mergeable<'a, A, B>(a: &'a A, b: &'a B) -> bool
where
    A: MaybeEmpty + Intersects<&'a B> + Adjacent<&'a B>,
    B: MaybeEmpty,
{
    a.intersects(b) || a.is_adjacent_to(b) || a.is_empty() || b.is_empty()
}

/// The union of two intervals if and only if connected else `None``.
///
/// Two intervals are connected if they share any elements (ie. [`Intersects`])
/// **or** if they are [`Adjacent`] such that no other elements exist between them.
/// The empty set is considered adjacent and connect to all other sets since
/// no elements exist between the two.
///
/// ```text
/// {x | x ∈ A ∨ x ∈ B } ⇔ {x} is an interval
/// ```
///
/// # Note
///
/// > Types subject to rounding errors (floats) may have unexpected results.
/// > When testing adjacency PartialEq is used directly. Handling
/// > edge cases is left to the end user. A fixed precision decimal
/// > type may be preferred in some cases.
///
/// # Examples
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0.0, 0.3);
/// let y = FiniteInterval::closed(0.1 + 0.2, 1.0);
///
/// assert_eq!(x.try_merge(y), None);
/// ```
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
///
/// let y = FiniteInterval::closed(11, 20);
/// assert_eq!(x.try_merge(y).unwrap(), FiniteInterval::closed(0, 20));
///
/// let y = FiniteInterval::closed(20, 30);
/// assert_eq!(x.try_merge(y), None);
///
/// let y = FiniteInterval::<i32>::empty();
/// assert_eq!(x.try_merge(y).unwrap(), x);
/// assert_eq!(y.try_merge(x).unwrap(), x);
///
/// let x = FiniteInterval::<i32>::empty();
/// assert_eq!(x.try_merge(y).unwrap(), FiniteInterval::empty());
/// ```
pub trait TryMerge<Rhs = Self> {
    /// The type of interval to return when mergeable.
    type Output;

    /// Tries to merge two intervals into a single interval.
    fn try_merge(self, rhs: Rhs) -> Option<Self::Output>;
}

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
            // SAFETY: assume invariants satisfied by FiniteInterval.
            unsafe { Some(HalfInterval::new_unchecked(self.side, bound)) }
        } else {
            let maybe_adjacent = self.side.select(&rhs_max, &rhs_min);
            if self.is_adjacent_to(maybe_adjacent) {
                let bound = self.side.select(rhs_min, rhs_max);
                // SAFETY: assum invariants satisfied by FiniteInterval.
                unsafe { Some(HalfInterval::new_unchecked(self.side, bound)) }
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
            unsafe { Some(HalfInterval::new_unchecked(self.side, bound)) }
        } else {
            let maybe_adj = self.side.select(rhs_max, rhs_min);
            if self.is_adjacent_to(maybe_adj) {
                let bound = self.side.select(rhs_min, rhs_max).clone();
                unsafe { Some(HalfInterval::new_unchecked(self.side, bound)) }
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
    /// Creates a new `MergeSorted` Iterator
    ///
    /// If the input is not sorted, behavior is undefined.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_finite_finite() {
        let a = EnumInterval::closed(0, 10);
        let b = EnumInterval::closed(5, 15);
        let c = EnumInterval::closed(20, 30);

        let expected = Some(EnumInterval::closed(0, 15));
        assert_eq!((&a).try_merge(&b), expected);
        assert_eq!(a.try_merge(b), expected);

        let expected = None;
        assert_eq!((&a).try_merge(&c), expected);
        assert_eq!(a.try_merge(c), expected);

        let empty = EnumInterval::empty();

        assert_eq!((&a).try_merge(&empty), Some(a));
        assert_eq!(a.try_merge(empty), Some(a));

        assert_eq!((&empty).try_merge(&a), Some(a));
        assert_eq!(empty.try_merge(a), Some(a));
    }

    #[test]
    fn test_half_half() {
        let a = EnumInterval::unbound_closed(-10);
        let b = EnumInterval::closed_unbound(10);

        assert_eq!((&a).try_merge(&b), None);
        assert_eq!(a.try_merge(b), None);

        let c = EnumInterval::unbound_closed(20);
        let expected = Some(EnumInterval::unbounded());
        assert_eq!((&b).try_merge(&c), expected);
        assert_eq!(b.try_merge(c), expected);

        assert_eq!((&a).try_merge(&c), Some(c));
        assert_eq!(a.try_merge(c), Some(c));
    }

    #[test]
    fn test_finite_half() {
        let a = EnumInterval::closed(0, 10);

        let b = EnumInterval::unbound_closed(5);
        let expected = Some(EnumInterval::unbound_closed(10));
        assert_eq!((&a).try_merge(&b), expected);
        assert_eq!(a.try_merge(b), expected);

        let b = EnumInterval::closed_unbound(5);
        let expected = Some(EnumInterval::closed_unbound(0));
        assert_eq!((&a).try_merge(&b), expected);
        assert_eq!(a.try_merge(b), expected);

        let b = EnumInterval::closed_unbound(0);
        assert_eq!((&a).try_merge(&b), Some(b));
        assert_eq!(a.try_merge(b), Some(b));

        let b = EnumInterval::closed_unbound(15);
        assert_eq!((&a).try_merge(&b), None);
        assert_eq!(a.try_merge(b), None);
    }
}

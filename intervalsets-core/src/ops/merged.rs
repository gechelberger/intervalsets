use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::numeric::Element;
use crate::ops::{Connects, Contains};
use crate::sets::EnumInterval::{self, *};
use crate::sets::{FiniteInterval, HalfInterval};
use crate::MaybeEmpty;

/// The union of two intervals if and only if [connected](`Connects`) else `None``.
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

impl<T: Element> TryMerge<Self> for FiniteInterval<T> {
    type Output = Self;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.connects(&rhs) {
            let Some((lhs_min, lhs_max)) = self.into_raw() else {
                // intersects is false, but the empty set
                // is trivially connected to all other sets.
                return Some(rhs);
            };

            let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
                // SAFETY: repacking
                let lhs = unsafe { Self::new_unchecked(lhs_min, lhs_max) };
                return Some(lhs);
            };

            // SAFETY: if lhs and rhs satisfy invariants, then bounds are
            // normalized, comparable, and min(left, right) <= max(left, right).
            let merged = unsafe {
                FiniteInterval::new_unchecked(
                    FiniteBound::take_min_unchecked(Side::Left, lhs_min, rhs_min),
                    FiniteBound::take_max_unchecked(Side::Right, lhs_max, rhs_max),
                )
            };

            Some(merged)
        } else {
            None
        }
    }
}

impl<T: Element + Clone> TryMerge<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.connects(rhs) {
            let Some((lhs_min, lhs_max)) = self.view_raw() else {
                return Some(rhs.clone());
            };

            let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
                // SAFETY: just putting it back together
                let lhs =
                    unsafe { FiniteInterval::new_unchecked(lhs_min.clone(), lhs_max.clone()) };
                return Some(lhs);
            };

            // SAFETY: if lhs and rhs satisfy invariants, bounds are normalized,
            // and min(left, right) <= max(left, right).
            let merged = unsafe {
                FiniteInterval::new_unchecked(
                    FiniteBound::min_unchecked(Side::Left, lhs_min, rhs_min).clone(),
                    FiniteBound::max_unchecked(Side::Right, lhs_max, rhs_max).clone(),
                )
            };

            Some(merged)
        } else {
            None
        }
    }
}

impl<T: Element> TryMerge<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                Some(self.into())
            } else {
                Some(rhs.into())
            }
        } else if self.connects(&rhs) {
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

impl<T: Element + Clone> TryMerge<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn try_merge(self, rhs: Self) -> Option<Self::Output> {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                Some(self.clone().into())
            } else {
                Some(rhs.clone().into())
            }
        } else if self.connects(rhs) {
            Some(EnumInterval::Unbounded)
        } else {
            None
        }
    }
}

impl<T: Element> TryMerge<FiniteInterval<T>> for HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: FiniteInterval<T>) -> Option<Self::Output> {
        if self.connects(&rhs) {
            let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
                return Some(self); // identity: merge with empty
            };

            let left_contained = self.side() == Left && self.contains(rhs_min.finite_ord(Left));
            let right_contained = self.side() == Right && self.contains(rhs_max.finite_ord(Right));
            if left_contained || right_contained {
                Some(self)
            } else {
                let bound = self.side().select(rhs_min, rhs_max);
                unsafe { Some(HalfInterval::new_unchecked(self.side(), bound)) }
            }
        } else {
            None
        }
    }
}

impl<T: Element + Clone> TryMerge<&FiniteInterval<T>> for &HalfInterval<T> {
    type Output = HalfInterval<T>;

    fn try_merge(self, rhs: &FiniteInterval<T>) -> Option<Self::Output> {
        if self.connects(rhs) {
            let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
                return Some(self.clone());
            };

            let left_contained = self.side() == Left && self.contains(rhs_min.finite_ord(Left));
            let right_contained = self.side() == Right && self.contains(rhs_max.finite_ord(Right));
            if left_contained || right_contained {
                Some(self.clone())
            } else {
                let bound = self.side().select(rhs_min, rhs_max).clone();
                unsafe { Some(HalfInterval::new_unchecked(self.side(), bound)) }
            }
        } else {
            None
        }
    }
}

macro_rules! dispatch_try_merge_impl {
    ($t_rhs:ty) => {
        impl<T: $crate::numeric::Element> TryMerge<$t_rhs> for EnumInterval<T> {
            type Output = EnumInterval<T>;
            #[inline(always)]
            fn try_merge(self, rhs: $t_rhs) -> Option<Self::Output> {
                match self {
                    Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
                    Unbounded => Some(Unbounded),
                }
            }
        }
        impl<T: $crate::numeric::Element + Clone> TryMerge<&$t_rhs> for &EnumInterval<T> {
            type Output = EnumInterval<T>;
            #[inline(always)]
            fn try_merge(self, rhs: &$t_rhs) -> Option<Self::Output> {
                match self {
                    Finite(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.try_merge(rhs).map(EnumInterval::from),
                    Unbounded => Some(Unbounded),
                }
            }
        }
    };
}

dispatch_try_merge_impl!(FiniteInterval<T>);
dispatch_try_merge_impl!(HalfInterval<T>);
dispatch_try_merge_impl!(EnumInterval<T>);

macro_rules! commutative_try_merge_impl {
    ($t_lhs:ty, $t_rhs:ty, $t_ret:ty) => {
        impl<T: $crate::numeric::Element> TryMerge<$t_rhs> for $t_lhs {
            type Output = $t_ret;
            #[inline(always)]
            fn try_merge(self, rhs: $t_rhs) -> Option<Self::Output> {
                rhs.try_merge(self)
            }
        }

        impl<T: $crate::numeric::Element + Clone> TryMerge<&$t_rhs> for &$t_lhs {
            type Output = $t_ret;
            #[inline(always)]
            fn try_merge(self, rhs: &$t_rhs) -> Option<Self::Output> {
                rhs.try_merge(self)
            }
        }
    };
}

commutative_try_merge_impl!(FiniteInterval<T>, HalfInterval<T>, HalfInterval<T>);
commutative_try_merge_impl!(FiniteInterval<T>, EnumInterval<T>, EnumInterval<T>);
commutative_try_merge_impl!(HalfInterval<T>, EnumInterval<T>, EnumInterval<T>);

/// MergeSorted merges intersecting intervals and returns disjoint ones.
///
/// As an `Iterator` is should return disjoint intervals from the sorted
/// input in order, omitting empty sets.
pub struct MergeSortedByValue<S, I: Iterator<Item = S>> {
    sorted: core::iter::Peekable<I>,
}

impl<S, I> MergeSortedByValue<S, I>
where
    S: MaybeEmpty,
    I: Iterator<Item = S>,
{
    /// Creates a new `MergeSorted` Iterator
    ///
    /// If the input is not sorted, behavior is undefined.
    pub fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = S, IntoIter = I>,
    {
        let mut sorted = sorted.into_iter().peekable();

        // discard all empty sets from the list
        while let Some(head) = sorted.peek() {
            if head.is_empty() {
                sorted.next();
            } else {
                break;
            }
        }

        Self { sorted }
    }
}

impl<S, I> Iterator for MergeSortedByValue<S, I>
where
    S: TryMerge + for<'a> Connects<&'a S>,
    S: From<<S as TryMerge>::Output>,
    I: Iterator<Item = S>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?;

        while let Some(peek) = self.sorted.peek() {
            if !current.connects(peek) {
                break;
            }

            let candidate = self.sorted.next().unwrap();
            current = match current.try_merge(candidate) {
                Some(merged) => S::from(merged),
                None => unreachable!(),
            };
        }

        Some(current)
    }
}

/// MergeSorted merges intersecting intervals and returns disjoint ones.
pub struct MergeSortedByRef<'a, T: 'a, I: Iterator<Item = &'a EnumInterval<T>>> {
    sorted: itertools::PutBack<I>,
}

impl<'a, T, I> MergeSortedByRef<'a, T, I>
where
    I: Iterator<Item = &'a EnumInterval<T>>,
{
    /// Creates a new `MergeSorted` Iterator
    ///
    /// If the input is not sorted, behavior is undefined.
    #[allow(unused)]
    pub fn new<U>(sorted: U) -> Self
    where
        U: IntoIterator<Item = &'a EnumInterval<T>, IntoIter = I>,
    {
        Self {
            sorted: itertools::put_back(sorted),
        }
    }
}

impl<'a, T, I> Iterator for MergeSortedByRef<'a, T, I>
where
    T: Clone + Element,
    I: Iterator<Item = &'a EnumInterval<T>>,
{
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current = self.sorted.next()?.clone();

        while let Some(cand) = self.sorted.next() {
            current = match (&current).try_merge(cand) {
                Some(merged) => merged,
                None => {
                    self.sorted.put_back(cand);
                    break;
                }
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

    extern crate std;

    #[test]
    fn test_merge_sorted_by_value() {
        let mut empty_by_val = MergeSortedByValue::new([FiniteInterval::<u8>::empty()]);
        assert_eq!(empty_by_val.next(), None);

        let finite = [
            FiniteInterval::empty(),
            FiniteInterval::closed(0, 10),
            FiniteInterval::closed(5, 15),
            FiniteInterval::closed(50, 60),
            FiniteInterval::closed(55, 65),
            FiniteInterval::closed(60, 70),
            FiniteInterval::closed(90, 100),
        ];

        let mut finite_by_val = MergeSortedByValue::new(finite);
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(0, 15)));
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(50, 70)));
        assert_eq!(finite_by_val.next(), Some(FiniteInterval::closed(90, 100)));
        assert_eq!(finite_by_val.next(), None);

        let enums = [
            EnumInterval::closed(0, 10),
            EnumInterval::closed(5, 15),
            EnumInterval::closed(50, 60),
            EnumInterval::closed(55, 65),
            EnumInterval::closed(60, 70),
            EnumInterval::closed(90, 100),
        ];

        let mut finite_by_val = MergeSortedByValue::new(enums);
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(0, 15)));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(50, 70)));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::closed(90, 100)));
        assert_eq!(finite_by_val.next(), None);
    }

    #[test]
    fn test_merge_sorted_by_value_merge() {
        let a = std::vec![
            EnumInterval::unbound_closed(-100),
            EnumInterval::closed(0, 10),
            EnumInterval::closed_unbound(100),
        ];

        let b = std::vec![
            EnumInterval::closed(-500, -400),
            EnumInterval::closed(-350, -300),
            EnumInterval::closed(-150, 150),
            EnumInterval::closed(300, 500),
        ];

        let mut finite_by_val = MergeSortedByValue::new(itertools::merge(a, b));
        assert_eq!(finite_by_val.next(), Some(EnumInterval::unbounded()));
        assert_eq!(finite_by_val.next(), None);
    }

    #[test]
    fn test_merge_sorted_by_ref() {
        let enums = [
            EnumInterval::closed(0, 10),
            EnumInterval::closed(5, 15),
            EnumInterval::closed(50, 60),
            EnumInterval::closed(55, 65),
            EnumInterval::closed(60, 70),
            EnumInterval::closed(90, 100),
        ];

        let mut finite_by_ref = MergeSortedByRef::new(enums.iter());
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(0, 15)));
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(50, 70)));
        assert_eq!(finite_by_ref.next(), Some(EnumInterval::closed(90, 100)));
        assert_eq!(finite_by_ref.next(), None);
    }
}

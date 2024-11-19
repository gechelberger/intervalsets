use num_traits::Zero;

use crate::bound::ord::{OrdBoundPair, OrdBounded};
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::factory::FiniteFactory;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval};
use crate::try_cmp::{TryMax, TryMin};

/// Tries to create the smallest interval which fully contains every provided item.
///
/// The set of input elements must have a valid total ordering or `None` will be returned.
/// This is distinct from `Some(empty)` if no elements are provided.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// // from points on the number line
/// let hull = FiniteInterval::convex_hull([5, 3, -120, 44, 100, -100]).unwrap();
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// let items = vec![5, 3, -120, 44, 100, -100];
/// let hull = FiniteInterval::convex_hull(&items).unwrap();
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// // from intervals
/// let intervals = [
///     EnumInterval::open(30, 50),
///     EnumInterval::closed(20, 40),
///     EnumInterval::closed(1000, 2000),
///     EnumInterval::unbound_open(0),
/// ];
/// let hull = EnumInterval::convex_hull(intervals).unwrap();
/// assert_eq!(hull, EnumInterval::unbound_closed(2000));
/// ```
pub trait ConvexHull<T>: Sized {
    /// Creates a convex hull of this `Set`
    fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Option<Self>;
}

macro_rules! convex_hull_t_impl {
    ($($t:ident), +) => {
        $(
            impl<T: Domain + Clone + TryMin + TryMax> ConvexHull<T> for $t<T> {

                fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Option<Self> {
                    let mut iter = iter.into_iter();

                    let (mut left, mut right) = match iter.next() {
                        None => return Some(Self::empty()),
                        Some(item) => (item.clone(), item),
                    };

                    // todo: un-unwrap()
                    for candidate in iter {
                        left = T::try_min(left, candidate.clone())?;
                        right = T::try_max(right, candidate)?;
                    }

                    Some(Self::closed(left, right))
                }
            }
        )+
    };
}

convex_hull_t_impl!(FiniteInterval, EnumInterval);

macro_rules! convex_hull_ref_t_impl {
    ($($t:ident), +) => {
        $(
            impl<'a, T: Domain + Clone + TryMin + TryMax> ConvexHull<&'a T> for $t<T> {
                fn convex_hull<U: IntoIterator<Item = &'a T>>(iter: U) -> Option<Self> {
                    let mut iter = iter.into_iter();
                    let (mut left, mut right) = match iter.next() {
                        None => return Some(Self::empty()),
                        Some(item) => (item.clone(), item.clone())
                    };

                    for candidate in iter {
                        left = T::try_min(left, candidate.clone())?;
                        right = T::try_max(right, candidate.clone())?;
                    }

                    Some(Self::closed(left, right))
                }
            }
        )+
    }
}

convex_hull_ref_t_impl!(FiniteInterval, EnumInterval);

impl<T: Domain + Clone> ConvexHull<FiniteInterval<T>> for FiniteInterval<T> {
    fn convex_hull<U: IntoIterator<Item = FiniteInterval<T>>>(iter: U) -> Option<Self> {
        let mut iter = iter.into_iter();

        let (mut left, mut right) = loop {
            match iter.next() {
                None => return Some(Self::empty()),
                Some(finite) => {
                    if finite.is_empty() {
                        continue;
                    } else {
                        break finite.into_raw().expect("Subset should not be empty");
                    }
                }
            }
        };

        for candidate in iter {
            if candidate.is_empty() {
                continue;
            }

            let (c_left, c_right) = candidate
                .into_raw()
                .expect("Hull subset should not be empty");

            // todo: partial order checks...
            left = FiniteBound::take_min(Side::Left, left, c_left);
            right = FiniteBound::take_max(Side::Right, right, c_right);
        }

        // SAFETY: hull should satisfy invariants (left <= right)
        unsafe { Some(Self::new_unchecked(left, right)) }
    }
}

/// Try to create a hull from elements that can be converted into `OrdBoundPair<T>`.
///
/// Returns `None` if input elements violate ordering requirements.
pub fn convex_hull_into_ord_bound_impl<T, B, I>(iter: I) -> Option<EnumInterval<T>>
where
    T: Domain + Zero,
    B: Into<OrdBoundPair<T>>,
    I: IntoIterator<Item = B>,
{
    let mut iter = iter.into_iter();

    // this is a little wonky:
    // skipping over empty intervals, take from iterator until :
    // 1) it is exhausted -> return Empty
    // 2) we find a non-empty interval and extract it's left and right bounds (or None for +/- inf)
    let (mut left, mut right) = loop {
        match iter.next() {
            None => return Some(EnumInterval::empty()),
            Some(inner) => {
                let pair: OrdBoundPair<T> = inner.into();
                if pair.is_empty() {
                    continue;
                } else {
                    break pair.into_raw();
                }
            }
        }
    };

    for item in iter {
        let pair: OrdBoundPair<T> = item.into();
        if pair.is_empty() {
            continue;
        }

        let (l_candidate, r_candidate) = pair.into_raw();
        left = left.try_min(l_candidate)?;
        right = right.try_max(r_candidate)?;
    }

    Some(OrdBoundPair::new(left, right).into())
}

/// Try to create a hull from `OrdBounded<T>` elements.
///
/// Returns `None` if input elements violate ordering requirements.
pub fn convex_hull_ord_bounded_impl<'a, T, B, I>(iter: I) -> Option<EnumInterval<T>>
where
    T: Domain + Clone + Zero,
    B: 'a + OrdBounded<T>,
    I: IntoIterator<Item = &'a B>,
{
    let mut iter = iter.into_iter();

    // this is a little wonky:
    // skipping over empty intervals, take from iterator until :
    // 1) it is exhausted -> return Empty
    // 2) we find a non-empty interval and extract it's left and right bounds (or None for +/- inf)
    let (mut left, mut right) = loop {
        match iter.next() {
            None => return Some(EnumInterval::empty()),
            Some(inner) => {
                let pair = inner.ord_bound_pair();
                if pair.is_empty() {
                    continue;
                } else {
                    break pair.into_raw();
                }
            }
        }
    };

    for item in iter {
        let pair = item.ord_bound_pair();
        if pair.is_empty() {
            continue;
        }

        let (l_candidate, r_candidate) = pair.into_raw();
        left = left.try_min(l_candidate)?;
        right = right.try_max(r_candidate)?;
    }

    let left = left.cloned();
    let right = right.cloned();
    Some(OrdBoundPair::new(left, right).into())
}

impl<T: Domain + Ord + Zero> ConvexHull<FiniteInterval<T>> for EnumInterval<T> {
    fn convex_hull<U: IntoIterator<Item = FiniteInterval<T>>>(iter: U) -> Option<Self> {
        convex_hull_into_ord_bound_impl(iter)
    }
}

impl<T: Domain + Ord + Zero> ConvexHull<EnumInterval<T>> for EnumInterval<T> {
    fn convex_hull<U: IntoIterator<Item = EnumInterval<T>>>(iter: U) -> Option<Self> {
        convex_hull_into_ord_bound_impl(iter)
    }
}

use super::ConvexHull;
use crate::bound::ord::OrdBoundPair;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::factory::Factory;
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, StackSet};

macro_rules! convex_hull_t_impl {
    ($($t:ident), +) => {
        $(
            impl<T: Domain + Ord + Clone> ConvexHull<T> for $t<T> {

                fn convex_hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
                    let mut iter = iter.into_iter();

                    let (mut left, mut right) = match iter.next() {
                        None => return Self::empty(),
                        Some(item) => (item.clone(), item),
                    };

                    for candidate in iter {
                        left = T::min(left, candidate.clone());
                        right = T::max(right, candidate);
                    }

                    Self::closed(left, right)
                }
            }
        )+
    };
}

convex_hull_t_impl!(FiniteInterval, EnumInterval);

macro_rules! convex_hull_ref_t_impl {
    ($($t:ident), +) => {
        $(
            impl<'a, T: Domain + Ord + Clone> ConvexHull<&'a T> for $t<T> {
                fn convex_hull<U: IntoIterator<Item = &'a T>>(iter: U) -> Self {
                    let mut iter = iter.into_iter();
                    let (mut left, mut right) = match iter.next() {
                        None => return Self::empty(),
                        Some(item) => (item.clone(), item.clone())
                    };

                    for candidate in iter {
                        left = T::min(left, candidate.clone());
                        right = T::max(right, candidate.clone());
                    }

                    Self::closed(left, right)
                }
            }
        )+
    }
}

convex_hull_ref_t_impl!(FiniteInterval, EnumInterval);

impl<T: Domain + Clone + Ord> ConvexHull<FiniteInterval<T>> for FiniteInterval<T> {
    fn convex_hull<U: IntoIterator<Item = FiniteInterval<T>>>(iter: U) -> Self {
        let mut iter = iter.into_iter();

        let (mut left, mut right) = loop {
            match iter.next() {
                None => return Self::empty(),
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
            left = FiniteBound::take_min(Side::Left, left, c_left);
            right = FiniteBound::take_max(Side::Right, right, c_right);
        }

        // SAFETY: hull should satisfy invariants (left <= right)
        unsafe { Self::new_unchecked(left, right) }
    }
}

fn convex_hull_ord_bound_impl<T, B, I>(iter: I) -> EnumInterval<T>
where
    T: Domain + Ord,
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
            None => return EnumInterval::empty(),
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
        left = left.min(l_candidate);
        right = right.max(r_candidate);
    }

    OrdBoundPair::new(left, right).into()
}

impl<T: Domain + Ord> ConvexHull<FiniteInterval<T>> for EnumInterval<T> {
    fn convex_hull<U: IntoIterator<Item = FiniteInterval<T>>>(iter: U) -> Self {
        convex_hull_ord_bound_impl(iter)
    }
}

impl<T: Domain + Ord> ConvexHull<EnumInterval<T>> for EnumInterval<T> {
    fn convex_hull<U: IntoIterator<Item = EnumInterval<T>>>(iter: U) -> Self {
        convex_hull_ord_bound_impl(iter)
    }
}

impl<T: Domain + Ord> ConvexHull<StackSet<T>> for EnumInterval<T> {
    fn convex_hull<U: IntoIterator<Item = StackSet<T>>>(iter: U) -> Self {
        convex_hull_ord_bound_impl(iter)
    }
}

use crate::bound::ord::{OrdBoundPair, OrdBounded};
use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
//use crate::empty::MaybeEmpty;
use crate::error::Error;
use crate::factory::traits::*;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval};
use crate::try_cmp::{try_ord_tuple, TryMax, TryMin};

/// Try to create the smallest interval which fully contains all elements.
///
/// The set of input elements must have a valid total ordering.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// // from points on the number line
/// let hull = FiniteInterval::hull([5, 3, -120, 44, 100, -100]);
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// let items = vec![5, 3, -120, 44, 100, -100];
/// let hull = FiniteInterval::hull(&items);
/// assert_eq!(hull, FiniteInterval::closed(-120, 100));
///
/// // from intervals
/// let intervals = [
///     EnumInterval::open(30, 50),
///     EnumInterval::closed(20, 40),
///     EnumInterval::closed(1000, 2000),
///     EnumInterval::unbound_open(0),
/// ];
/// let hull = EnumInterval::hull(intervals);
/// assert_eq!(hull, EnumInterval::unbound_closed(2000));
/// ```
pub trait ConvexHull<T>: Sized {
    type Error: core::error::Error;

    /// Try to creates a convex hull of this `Set`
    fn strict_hull<U: IntoIterator<Item = T>>(iter: U) -> Result<Self, Self::Error>;

    fn hull<U: IntoIterator<Item = T>>(iter: U) -> Self {
        Self::strict_hull(iter).unwrap()
    }
}

macro_rules! convex_hull_t_impl {
    ($($t:ident), +) => {
        $(
            impl<T: Element + Clone + TryMin + TryMax> ConvexHull<T> for $t<T> {
                type Error = $crate::error::Error;

                fn strict_hull<U: IntoIterator<Item = T>>(iter: U) -> Result<Self, Self::Error> {
                    let mut iter = iter.into_iter();

                    let (mut left, mut right) = match iter.next() {
                        None => return Ok(Self::empty()),
                        Some(item) => (item.clone(), item),
                    };

                    for mut candidate in iter {
                        (left, candidate) = try_ord_tuple(left, candidate)?;
                        (_, right) = try_ord_tuple(right, candidate)?;
                    }

                    Self::strict_closed(left, right)
                }
            }
        )+
    };
}

convex_hull_t_impl!(FiniteInterval, EnumInterval);

macro_rules! convex_hull_ref_t_impl {
    ($($t:ident), +) => {
        $(
            impl<'a, T: Element + Clone + TryMin + TryMax> ConvexHull<&'a T> for $t<T> {
                type Error = $crate::error::Error;

                fn strict_hull<U: IntoIterator<Item = &'a T>>(iter: U) -> Result<Self, Self::Error> {
                    let mut iter = iter.into_iter();

                    let (mut left, mut right) = match iter.next() {
                        None => return Ok(Self::empty()),
                        Some(item) => (item, item)
                    };

                    for candidate in iter {
                        left = <&T>::try_min(left, candidate)?;
                        right = <&T>::try_max(right, candidate)?;
                    }

                    Self::strict_closed(left.clone(), right.clone())
                }
            }
        )+
    }
}

convex_hull_ref_t_impl!(FiniteInterval, EnumInterval);

impl<T: Element> ConvexHull<Self> for FiniteInterval<T> {
    type Error = core::convert::Infallible;

    fn strict_hull<U>(iter: U) -> Result<Self, Self::Error>
    where
        U: IntoIterator<Item = FiniteInterval<T>>,
    {
        let mut iter = iter.into_iter();

        let (mut left, mut right) = loop {
            match iter.next() {
                None => return Ok(Self::empty()),
                Some(finite) => match finite.into_raw() {
                    None => continue,
                    Some(pair) => break pair,
                },
            }
        };

        for candidate in iter {
            let (c_left, c_right) = match candidate.into_raw() {
                None => continue,
                Some(pair) => pair,
            };

            // SAFETY: if input intervals satisfy invariants then this is safe.
            unsafe {
                left = FiniteBound::take_min_unchecked(Side::Left, left, c_left);
                right = FiniteBound::take_max_unchecked(Side::Right, right, c_right);
            }
        }

        // SAFETY: hull should satisfy invariants (left <= right)
        unsafe { Ok(Self::new_unchecked(left, right)) }
    }
}

impl<'a, T: Element + Clone> ConvexHull<&'a Self> for FiniteInterval<T> {
    type Error = core::convert::Infallible;

    fn strict_hull<U: IntoIterator<Item = &'a Self>>(iter: U) -> Result<Self, Self::Error> {
        let mut iter = iter.into_iter();

        let (mut left, mut right) = loop {
            match iter.next() {
                None => return Ok(Self::empty()),
                Some(interval) => match interval.view_raw() {
                    None => continue,
                    Some((lhs, rhs)) => break (lhs, rhs),
                },
            }
        };

        for candidate in iter {
            let Some((c_left, c_right)) = candidate.view_raw() else {
                continue;
            };

            unsafe {
                left = FiniteBound::min_unchecked(Left, left, c_left);
                right = FiniteBound::max_unchecked(Right, right, c_right);
            }
        }

        unsafe { Ok(Self::new_unchecked(left.clone(), right.clone())) }
    }
}

/// Try to create a hull from elements that can be converted into `OrdBoundPair<T>`.
///
/// Returns `None` if input elements violate ordering requirements.
pub fn convex_hull_into_ord_bound_impl<T, B, I>(iter: I) -> Result<EnumInterval<T>, Error>
where
    T: Element,
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
            None => return Ok(EnumInterval::empty()),
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

        let (c_left, c_right) = pair.into_raw();
        left = left.try_min(c_left)?;
        right = right.try_max(c_right)?;
    }

    OrdBoundPair::new(left, right).try_into()
}

/// Try to create a hull from `OrdBounded<T>` elements.
///
/// Returns `None` if input elements violate ordering requirements.
pub fn convex_hull_ord_bounded_impl<'a, T, B, I>(iter: I) -> Result<EnumInterval<T>, Error>
where
    T: Element + Clone,
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
            None => return Ok(EnumInterval::empty()),
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

        let (c_left, c_right) = pair.into_raw();
        left = left.try_min(c_left)?;
        right = right.try_max(c_right)?;
    }

    let left = left.cloned();
    let right = right.cloned();
    OrdBoundPair::new(left, right).try_into()
}

impl<T: Element> ConvexHull<FiniteInterval<T>> for EnumInterval<T> {
    type Error = Error;

    fn strict_hull<U>(iter: U) -> Result<Self, Self::Error>
    where
        U: IntoIterator<Item = FiniteInterval<T>>,
    {
        convex_hull_into_ord_bound_impl(iter)
    }
}

impl<'a, T: Element + Clone> ConvexHull<&'a FiniteInterval<T>> for EnumInterval<T> {
    type Error = core::convert::Infallible;

    fn strict_hull<U: IntoIterator<Item = &'a FiniteInterval<T>>>(
        iter: U,
    ) -> Result<Self, Self::Error> {
        FiniteInterval::strict_hull(iter).map(EnumInterval::from)
    }
}

impl<T: Element> ConvexHull<EnumInterval<T>> for EnumInterval<T> {
    type Error = Error;

    fn strict_hull<U>(iter: U) -> Result<Self, Self::Error>
    where
        U: IntoIterator<Item = EnumInterval<T>>,
    {
        convex_hull_into_ord_bound_impl(iter)
    }
}

impl<'a, T: Element + Clone> ConvexHull<&'a EnumInterval<T>> for EnumInterval<T> {
    type Error = Error;

    fn strict_hull<U: IntoIterator<Item = &'a EnumInterval<T>>>(
        iter: U,
    ) -> Result<Self, Self::Error> {
        convex_hull_ord_bounded_impl(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TotalOrderError;

    #[test]
    fn test_hull_t() {
        let x = FiniteInterval::strict_hull([f32::NAN]);
        assert_eq!(x, Err(Error::TotalOrderError(TotalOrderError)));

        let x = FiniteInterval::strict_hull([&f32::NAN]);
        assert_eq!(x, Err(Error::TotalOrderError(TotalOrderError)));

        let data = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];

        // by ref
        let x = FiniteInterval::strict_hull(data.iter());
        assert_eq!(x.unwrap(), FiniteInterval::closed(0.0, 5.0));

        // by val
        let x = FiniteInterval::strict_hull(data);
        assert_eq!(x.unwrap(), FiniteInterval::closed(0.0, 5.0));
    }

    #[test]
    fn test_hull_of_finite() {
        let intervals = [
            FiniteInterval::open(90, 100),
            FiniteInterval::open(0, 10),
            FiniteInterval::closed(50, 60),
        ];

        assert_eq!(
            FiniteInterval::strict_hull(intervals.iter()).unwrap(),
            FiniteInterval::open(0, 100)
        );

        assert_eq!(
            FiniteInterval::strict_hull(intervals).unwrap(),
            FiniteInterval::open(0, 100)
        );
    }

    #[test]
    fn test_hull_of_enum() {
        let intervals = [
            EnumInterval::closed(0, 10),
            EnumInterval::closed_unbound(100),
            EnumInterval::unbound_closed(-100),
        ];

        assert_eq!(
            EnumInterval::strict_hull(intervals.iter()).unwrap(),
            EnumInterval::unbounded()
        );

        assert_eq!(
            EnumInterval::strict_hull(intervals).unwrap(),
            EnumInterval::unbounded()
        );

        let intervals = [
            FiniteInterval::open(0.0, 10.0),
            FiniteInterval::closed(0.0, 5.0),
        ];

        assert_eq!(
            EnumInterval::strict_hull(intervals.iter()).unwrap(),
            EnumInterval::closed_open(0.0, 10.0)
        );

        assert_eq!(
            EnumInterval::strict_hull(intervals).unwrap(),
            EnumInterval::closed_open(0.0, 10.0)
        );
    }
}

use super::util::commutative_op_move_impl;
use super::{Contains, Intersection};
use crate::bound::ord::OrdBounded;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::sets::*;

impl<T: Domain> Intersection<Self> for FiniteInterval<T> {
    type Output = Self;

    #[inline]
    fn intersection(self, rhs: Self) -> Self::Output {
        let Self::Bounded(lhs_min, lhs_max) = self else {
            return Self::Empty;
        };

        let Self::Bounded(rhs_min, rhs_max) = rhs else {
            return Self::Empty;
        };

        // Safety: self and rhs should already be normalized.
        unsafe {
            Self::new_norm(
                FiniteBound::take_max(Side::Left, lhs_min, rhs_min),
                FiniteBound::take_min(Side::Right, lhs_max, rhs_max),
            )
        }
    }
}

impl<T: Domain> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Self;

    #[inline]
    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        let Self::Bounded(lhs_min, lhs_max) = self else {
            return Self::Empty;
        };

        let n = [&lhs_min, &lhs_max]
            .into_iter()
            .filter(|bound| rhs.contains(bound.value()))
            .count();

        if n == 2 {
            unsafe { Self::new_unchecked(lhs_min, lhs_max) }
        } else if n == 1 {
            match rhs.side {
                Side::Left => unsafe {
                    // SAFETY: bound should already be normalized
                    Self::new_norm(rhs.bound, lhs_max)
                },
                Side::Right => unsafe {
                    // SAFETY: bound should already be normalized
                    Self::new_norm(lhs_min, rhs.bound)
                },
            }
        } else {
            Self::Empty
        }
    }
}

impl<T: Domain> Intersection<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline]
    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.into()
            } else {
                self.into()
            }
        } else {
            unsafe {
                match self.side {
                    Side::Left => FiniteInterval::new_norm(self.bound, rhs.bound),
                    Side::Right => FiniteInterval::new_norm(rhs.bound, self.bound),
                }
            }
            .into()
        }
    }
}

impl<T: Domain> Intersection<FiniteInterval<T>> for EnumInterval<T> {
    type Output = Self;

    fn intersection(self, rhs: FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs).into(),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain> Intersection<HalfInterval<T>> for EnumInterval<T> {
    type Output = Self;

    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain> Intersection<Self> for EnumInterval<T> {
    type Output = Self;

    fn intersection(self, rhs: Self) -> Self::Output {
        match self {
            Self::Finite(lhs) => rhs.intersection(lhs),
            Self::Half(lhs) => rhs.intersection(lhs),
            Self::Unbounded => rhs,
        }
    }
}

commutative_op_move_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    FiniteInterval<T>,
    FiniteInterval<T>
);
commutative_op_move_impl!(
    Intersection,
    intersection,
    FiniteInterval<T>,
    EnumInterval<T>,
    EnumInterval<T>
);
commutative_op_move_impl!(
    Intersection,
    intersection,
    HalfInterval<T>,
    EnumInterval<T>,
    EnumInterval<T>
);

/// Compute the intersection of two iterators of intervals.
///
/// The input iterators are consumed. Each input iterator
/// **must** satisfy the normal Set invariants: (non-empty,
/// disjoint, sorted).
///
/// # Example
///
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::ops::intersection::SetSetIntersection;
/// let a = [
///     EnumInterval::closed(0, 25),
///     EnumInterval::closed(75, 100)
/// ];
/// let b = [
///     EnumInterval::closed(10, 15),
///     EnumInterval::closed(20, 80),
///     EnumInterval::closed(95, 200)
/// ];
///
/// let mut s = SetSetIntersection::new(a, b);
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(10, 15));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(20, 25));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(75, 80));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(95, 100));
/// assert_eq!(s.next(), None);
/// ```
pub struct SetSetIntersection<T, I1, I2>
where
    I1: Iterator<Item = EnumInterval<T>>,
    I2: Iterator<Item = EnumInterval<T>>,
{
    a: itertools::PutBack<I1>,
    b: itertools::PutBack<I2>,
}

impl<T, I1, I2> SetSetIntersection<T, I1, I2>
where
    I1: Iterator<Item = EnumInterval<T>>,
    I2: Iterator<Item = EnumInterval<T>>,
{
    pub fn new<U1, U2>(a: U1, b: U2) -> Self
    where
        U1: IntoIterator<Item = EnumInterval<T>, IntoIter = I1>,
        U2: IntoIterator<Item = EnumInterval<T>, IntoIter = I2>,
    {
        Self {
            a: itertools::put_back(a),
            b: itertools::put_back(b),
        }
    }
}

impl<T, I1, I2> Iterator for SetSetIntersection<T, I1, I2>
where
    T: Domain + Clone,
    I1: Iterator<Item = EnumInterval<T>>,
    I2: Iterator<Item = EnumInterval<T>>,
{
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.a.next()?;
        let b = self.b.next()?;

        let ab = a.clone().intersection(b.clone());
        //let ab = a.ref_intersection(&b);
        if !ab.is_empty() {
            // since `a` and `b` intersect, we want to look at the right hand
            // bounds to decide which one (or both) to discard.
            let (_, a_r) = a.ord_bound_pair().into_raw();
            let (_, b_r) = b.ord_bound_pair().into_raw();
            if a_r > b_r {
                self.a.put_back(a);
            } else if a_r < b_r {
                self.b.put_back(b);
            }
            Some(ab)
        } else {
            // since `a` and `b` are disjoint, discard the one with the
            // smallest left hand bound.
            let (l_a, _) = a.ord_bound_pair().into_raw();
            let (l_b, _) = b.ord_bound_pair().into_raw();
            if l_a > l_b {
                self.a.put_back(a);
            } else {
                self.b.put_back(b);
            }

            self.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Factory;

    #[test]
    fn test_finite_finite() {
        assert_eq!(
            FiniteInterval::closed(0, 100).intersection(FiniteInterval::closed(50, 150)),
            FiniteInterval::closed(50, 100)
        );

        assert_eq!(
            FiniteInterval::closed(0, 100).intersection(FiniteInterval::Empty),
            FiniteInterval::Empty
        );
    }
}

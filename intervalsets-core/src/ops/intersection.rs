use super::util::commutative_op_move_impl;
use super::{Contains, Intersection};
use crate::bound::ord::OrdBounded;
use crate::bound::{FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::sets::*;

impl<T: Domain> Intersection<Self> for FiniteInterval<T> {
    type Output = Self;

    fn intersection(self, rhs: Self) -> Self::Output {
        self.flat_map(|a_lhs, a_rhs| {
            rhs.flat_map(|b_lhs, b_rhs| {
                Self::new(
                    FiniteBound::take_max(Side::Left, a_lhs, b_lhs),
                    FiniteBound::take_min(Side::Right, a_rhs, b_rhs),
                )
                .unwrap_or(Self::Empty)
            })
        })
    }
}

impl<T: Domain> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Self;

    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        self.map(|left, right| {
            let n = [&left, &right]
                .into_iter()
                .filter(|bound| rhs.contains(bound.value()))
                .count();

            if n == 2 {
                Self::Bounded(left, right)
            } else if n == 1 {
                match rhs.side {
                    Side::Left => Self::new(rhs.bound, right),
                    Side::Right => Self::new(left, rhs.bound),
                }
                .unwrap()
            } else {
                Self::Empty
            }
        })
        .unwrap_or(Self::Empty)
    }
}

impl<T: Domain> Intersection<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.into()
            } else {
                self.into()
            }
        } else {
            match self.side {
                Side::Left => FiniteInterval::new(self.bound, rhs.bound),
                Side::Right => FiniteInterval::new(rhs.bound, self.bound),
            }
            .unwrap_or(FiniteInterval::Empty)
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

impl<T: Domain + Clone + Ord> Intersection<EnumInterval<T>> for StackSet<T> {
    type Output = Self;

    fn intersection(self, rhs: EnumInterval<T>) -> Self::Output {
        if self.is_empty() || rhs.is_empty() {
            return Self::empty();
        }

        // invariants:
        // intervals remain sorted; remain disjoint; filter out empty results;
        let intervals: crate::sets::StackSetStorage<_> = self
            .into_raw()
            .into_iter()
            .map(|iv| iv.intersection(rhs.clone()))
            .filter(|iv| !iv.is_empty())
            .collect();

        unsafe { Self::new_unchecked(intervals) }
    }
}

impl<T: Domain + Clone + Ord> Intersection<StackSet<T>> for EnumInterval<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: StackSet<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

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

impl<T: Domain + Clone> Intersection<Self> for StackSet<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: Self) -> Self::Output {
        unsafe { Self::new_unchecked(SetSetIntersection::new(self.into_raw(), rhs.into_raw())) }
    }
}

impl<T: Domain + Clone + Ord> Intersection<HalfInterval<T>> for StackSet<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        self.intersection(EnumInterval::from(rhs))
    }
}

impl<T: Domain + Clone + Ord> Intersection<StackSet<T>> for HalfInterval<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: StackSet<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

impl<T: Domain + Clone + Ord> Intersection<FiniteInterval<T>> for StackSet<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: FiniteInterval<T>) -> Self::Output {
        self.intersection(EnumInterval::from(rhs))
    }
}

impl<T: Domain + Clone + Ord> Intersection<StackSet<T>> for FiniteInterval<T> {
    type Output = StackSet<T>;

    fn intersection(self, rhs: StackSet<T>) -> Self::Output {
        rhs.intersection(self)
    }
}
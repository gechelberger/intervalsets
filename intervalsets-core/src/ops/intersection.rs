use super::util::commutative_op_move_impl;
use super::Contains;
use crate::bound::ord::OrdBounded;
use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::empty::MaybeEmpty;
use crate::numeric::Domain;
use crate::sets::EnumInterval::{self, Finite, Half, Unbounded};
use crate::sets::FiniteInterval::{self, Bounded, Empty};
use crate::sets::HalfInterval;

/// The intersection of two sets.
///
/// ```text
/// {x | x ∈ A ∧ x ∈ B }
/// ```
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// let y = FiniteInterval::closed(5, 15);
/// assert_eq!(x.intersection(y), FiniteInterval::closed(5, 10));
///
/// let y = FiniteInterval::closed(20, 30);
/// assert!(x.intersection(y).is_empty());
/// ```
pub trait Intersection<Rhs = Self> {
    /// The type of `Set` to create.
    type Output;

    /// Creates a new `Set` intersection of A and B.
    fn intersection(self, rhs: Rhs) -> Self::Output;
}

impl<T: Domain> Intersection<Self> for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        let Bounded(lhs_min, lhs_max) = self else {
            return Empty;
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return Empty;
        };

        // Safety: self and rhs should already be normalized.
        unsafe {
            FiniteInterval::new_norm(
                FiniteBound::take_max_unchecked(Left, lhs_min, rhs_min),
                FiniteBound::take_min_unchecked(Right, lhs_max, rhs_max),
            )
        }
    }
}

impl<T: Domain + Clone> Intersection<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        let Bounded(lhs_min, lhs_max) = self else {
            return Empty;
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return Empty;
        };

        // Safety: self and rhs should already be normalized.
        unsafe {
            FiniteInterval::new_norm(
                FiniteBound::max_unchecked(Left, lhs_min, rhs_min).clone(),
                FiniteBound::min_unchecked(Right, lhs_max, rhs_max).clone(),
            )
        }
    }
}

impl<T: Domain> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        /*if self.contains(rhs.bound.value()) {
            let Bounded(lhs_min, lhs_max) = self else {
                unreachable!();
            };
            match rhs.side {
                Left => unsafe { FiniteInterval::new_norm(rhs.bound, lhs_max) },
                Right => unsafe { FiniteInterval::new_norm(lhs_min, rhs.bound) },
            }
        } else if rhs.contains(&self) {
            self
        } else {
            Empty
        }*/

        let Bounded(lhs_min, lhs_max) = self else {
            return Empty;
        };

        let n = [&lhs_min, &lhs_max]
            .into_iter()
            .filter(|bound| rhs.contains(bound.value()))
            .count();

        if n == 2 {
            unsafe { FiniteInterval::new_unchecked(lhs_min, lhs_max) }
        } else if n == 1 {
            // SAFETY: bounds should already be normalized
            match rhs.side {
                Left => unsafe { FiniteInterval::new_norm(rhs.bound, lhs_max) },
                Right => unsafe { FiniteInterval::new_norm(lhs_min, rhs.bound) },
            }
        } else {
            Empty
        }
    }
}

impl<T: Domain + Clone> Intersection<&HalfInterval<T>> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &HalfInterval<T>) -> Self::Output {
        if self.contains(rhs.bound.value()) {
            let Bounded(lhs_min, lhs_max) = self else {
                unreachable!();
            };

            match rhs.side {
                Left => unsafe { FiniteInterval::new_norm(rhs.bound.clone(), lhs_max.clone()) },
                Right => unsafe { FiniteInterval::new_norm(lhs_min.clone(), rhs.bound.clone()) },
            }
        } else if rhs.contains(self) {
            self.clone()
        } else {
            Empty
        }
    }
}

impl<T: Domain + Clone> Intersection<&FiniteInterval<T>> for &HalfInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &FiniteInterval<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

impl<T: Domain> Intersection<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.into()
            } else {
                self.into()
            }
        } else {
            // SAFETY: bounds are already normalized
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

impl<T: Domain + Clone> Intersection<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side == rhs.side {
            if self.contains(rhs.bound.value()) {
                rhs.clone().into()
            } else {
                self.clone().into()
            }
        } else if self.contains(rhs.bound.value()) {
            let lhs = self.bound.clone();
            let rhs = rhs.bound.clone();
            match self.side {
                Left => unsafe { FiniteInterval::new_norm(lhs, rhs).into() },
                Right => unsafe { FiniteInterval::new_norm(rhs, lhs).into() },
            }
        } else {
            Empty.into()
        }
    }
}

impl<T: Domain> Intersection<FiniteInterval<T>> for EnumInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: FiniteInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs).into(),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain + Clone> Intersection<&FiniteInterval<T>> for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &FiniteInterval<T>) -> Self::Output {
        match self {
            Finite(lhs) => lhs.intersection(rhs).into(),
            Half(lhs) => lhs.intersection(rhs).into(),
            Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<HalfInterval<T>> for EnumInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        match self {
            Self::Finite(lhs) => lhs.intersection(rhs).into(),
            Self::Half(lhs) => lhs.intersection(rhs),
            Self::Unbounded => rhs.into(),
        }
    }
}

impl<T: Domain + Clone> Intersection<&HalfInterval<T>> for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &HalfInterval<T>) -> Self::Output {
        match self {
            Finite(lhs) => lhs.intersection(rhs).into(),
            Half(lhs) => lhs.intersection(rhs),
            Unbounded => rhs.clone().into(),
        }
    }
}

impl<T: Domain> Intersection<Self> for EnumInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        match self {
            Self::Finite(lhs) => rhs.intersection(lhs),
            Self::Half(lhs) => rhs.intersection(lhs),
            Self::Unbounded => rhs,
        }
    }
}

impl<T: Domain + Clone> Intersection for &EnumInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        match self {
            Finite(lhs) => lhs.intersection(rhs),
            Half(lhs) => lhs.intersection(rhs),
            Unbounded => rhs.clone(),
        }
    }
}

impl<T: Domain + Clone> Intersection<&EnumInterval<T>> for &FiniteInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &EnumInterval<T>) -> Self::Output {
        rhs.intersection(self)
    }
}

impl<T: Domain + Clone> Intersection<&EnumInterval<T>> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &EnumInterval<T>) -> Self::Output {
        rhs.intersection(self)
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
/// use intervalsets_core::ops::SetSetIntersection;
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
    /// Creates a new SetSetIntersection Iterator
    ///
    /// If the standard `Set` invariants are not satisfied, behavior is undefined.
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

        let ab = (&a).intersection(&b);
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
    use crate::factory::FiniteFactory;

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

    #[test]
    fn test_finite_half() {
        let x = FiniteInterval::closed(0, 100);
        let y = HalfInterval::left(FiniteBound::closed(50));
        let expected = FiniteInterval::closed(50, 100);
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);

        let x = FiniteInterval::closed(0.0, 100.0);
        let y = HalfInterval::right(FiniteBound::open(0.0));
        let expected = FiniteInterval::empty();
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);

        let x = FiniteInterval::closed(0.0, 100.0);
        let y = HalfInterval::right(FiniteBound::closed(0.0));
        let expected = FiniteInterval::closed(0.0, 0.0);
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);
    }

    #[test]
    fn test_half_half() {
        let x = HalfInterval::left(FiniteBound::open(0.0));
        let y = HalfInterval::right(FiniteBound::open(100.0));
        let expected = EnumInterval::open(0.0, 100.0);
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);

        let x = HalfInterval::left(FiniteBound::open(0.0));
        let y = HalfInterval::right(FiniteBound::open(0.0));
        let expected = EnumInterval::empty();
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);

        let x = HalfInterval::left(FiniteBound::closed(0.0));
        let y = HalfInterval::left(FiniteBound::closed(100.0));
        let expected = EnumInterval::from(y.clone());
        assert_eq!((&x).intersection(&y), expected);
        assert_eq!(x.intersection(y), expected);
    }

    fn check_enum_enum<T>(expect: EnumInterval<T>, a: EnumInterval<T>, b: EnumInterval<T>)
    where
        T: PartialEq + Domain + Clone + core::fmt::Debug,
    {
        assert_eq!(expect, (&a).intersection(&b));
        assert_eq!(expect, a.intersection(b));
    }

    #[test]
    fn test_enum_enum() {
        check_enum_enum(
            EnumInterval::empty(),
            EnumInterval::closed(0, 10),
            EnumInterval::closed(20, 30),
        );

        check_enum_enum(
            EnumInterval::open(5.0, 10.0),
            EnumInterval::open(0.0, 10.0),
            EnumInterval::open(5.0, 15.0),
        );
    }
}

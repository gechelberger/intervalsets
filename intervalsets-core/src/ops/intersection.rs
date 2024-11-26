use core::borrow::Borrow;

use super::Contains;
use crate::bound::ord::OrdBounded;
use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
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
pub trait Intersection<Rhs = Self>: Sized {
    /// The type of `Set` to create.
    type Output;
    type Error: core::error::Error;

    /// Creates a new `Set` intersection of A and B.
    #[inline(always)]
    fn intersection(self, rhs: Rhs) -> Self::Output {
        self.strict_intersection(rhs).unwrap()
    }

    fn strict_intersection(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

impl<T: Element> Intersection<Self> for FiniteInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let Bounded(lhs_min, lhs_max) = self else {
            return Ok(Empty);
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return Ok(Empty);
        };

        // SAFETY: self and rhs should already satisfy invariants.
        unsafe {
            FiniteInterval::new_strict_assume_normed(
                FiniteBound::take_max_unchecked(Left, lhs_min, rhs_min),
                FiniteBound::take_min_unchecked(Right, lhs_max, rhs_max),
            )
        }
    }
}

impl<T: Element + Clone> Intersection<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        let Bounded(lhs_min, lhs_max) = self else {
            return Ok(Empty);
        };

        let Bounded(rhs_min, rhs_max) = rhs else {
            return Ok(Empty);
        };

        unsafe {
            FiniteInterval::new_strict_assume_normed(
                FiniteBound::max_unchecked(Left, lhs_min, rhs_min).clone(),
                FiniteBound::min_unchecked(Right, lhs_max, rhs_max).clone(),
            )
        }
    }
}

impl<T: Element> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Ok(Empty);
        };

        let n = [lhs_min.finite_ord(Left), lhs_max.finite_ord(Right)]
            .into_iter()
            .filter(|bound| rhs.contains(*bound))
            .count();

        if n == 2 {
            // SAFETY: just putting it back together
            unsafe { Ok(FiniteInterval::new_unchecked(lhs_min, lhs_max)) }
        } else if n == 1 {
            // SAFETY: if self and rhs already satisfy invariants then ok.
            unsafe {
                match rhs.side {
                    Left => FiniteInterval::new_strict_assume_normed(rhs.bound, lhs_max),
                    Right => FiniteInterval::new_strict_assume_normed(lhs_min, rhs.bound),
                }
            }
        } else {
            Ok(Empty)
        }
    }
}

impl<T: Element + Clone> Intersection<&HalfInterval<T>> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: &HalfInterval<T>) -> Result<Self::Output, Self::Error> {
        let Bounded(lhs_min, lhs_max) = self else {
            return Ok(Empty);
        };

        let n = [lhs_min.finite_ord(Left), lhs_max.finite_ord(Right)]
            .into_iter()
            .filter(|bound| rhs.contains(*bound))
            .count();

        if n == 2 {
            // SAFETY: just putting it back together
            unsafe {
                Ok(FiniteInterval::new_unchecked(
                    lhs_min.clone(),
                    lhs_max.clone(),
                ))
            }
        } else if n == 1 {
            // SAFETY: if self and rhs already satisfy invariants then ok.
            unsafe {
                match rhs.side {
                    Left => {
                        FiniteInterval::new_strict_assume_normed(rhs.bound.clone(), lhs_max.clone())
                    }
                    Right => {
                        FiniteInterval::new_strict_assume_normed(lhs_min.clone(), rhs.bound.clone())
                    }
                }
            }
        } else {
            Ok(Empty)
        }
    }
}

impl<T: Element> Intersection<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        if self.side == rhs.side {
            if self.contains(rhs.finite_ord_bound()) {
                Ok(rhs.into())
            } else {
                Ok(self.into())
            }
        } else {
            // SAFETY: self and rhs should already satifsy invariants
            let result = unsafe {
                match self.side {
                    Side::Left => FiniteInterval::new_strict_assume_normed(self.bound, rhs.bound),
                    Side::Right => FiniteInterval::new_strict_assume_normed(rhs.bound, self.bound),
                }
            };

            result.map(EnumInterval::from)
        }
    }
}

impl<T: Element + Clone> Intersection<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    #[inline(always)]
    fn strict_intersection(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        if self.side == rhs.side {
            if self.contains(rhs.finite_ord_bound()) {
                Ok(rhs.clone().into())
            } else {
                Ok(self.clone().into())
            }
        } else if self.contains(rhs.finite_ord_bound()) {
            let lhs = self.bound.clone();
            let rhs = rhs.bound.clone();

            // SAFETY: self and rhs should satisfy invariants
            let result = unsafe {
                match self.side {
                    Left => FiniteInterval::new_strict_assume_normed(lhs, rhs),
                    Right => FiniteInterval::new_strict_assume_normed(rhs, lhs),
                }
            };
            result.map(EnumInterval::from)
        } else {
            Ok(Empty.into())
        }
    }
}

macro_rules! dispatch_intersection_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Element> Intersection<$t_rhs> for $t_lhs {
            type Output = EnumInterval<T>;
            type Error = $crate::error::Error;

            #[inline(always)]
            fn strict_intersection(self, rhs: $t_rhs) -> Result<Self::Output, Self::Error> {
                match self {
                    Finite(lhs) => lhs.strict_intersection(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.strict_intersection(rhs).map(EnumInterval::from),
                    Unbounded => Ok(rhs.into()),
                }
            }
        }

        impl<T: $crate::numeric::Element + Clone> Intersection<&$t_rhs> for &$t_lhs {
            type Output = EnumInterval<T>;
            type Error = $crate::error::Error;

            #[inline(always)]
            fn strict_intersection(self, rhs: &$t_rhs) -> Result<Self::Output, Self::Error> {
                match self {
                    Finite(lhs) => lhs.strict_intersection(rhs).map(EnumInterval::from),
                    Half(lhs) => lhs.strict_intersection(rhs).map(EnumInterval::from),
                    Unbounded => Ok(rhs.clone().into()),
                }
            }
        }
    };
}

dispatch_intersection_impl!(EnumInterval<T>, EnumInterval<T>);
dispatch_intersection_impl!(EnumInterval<T>, HalfInterval<T>);
dispatch_intersection_impl!(EnumInterval<T>, FiniteInterval<T>);

macro_rules! commutative_intersection_impl {
    ($t_lhs:ty, $t_rhs:ty, $t_ret:ty) => {
        impl<T: $crate::numeric::Element> Intersection<$t_rhs> for $t_lhs {
            type Output = $t_ret;
            type Error = $crate::error::Error;

            #[inline(always)]
            fn strict_intersection(self, rhs: $t_rhs) -> Result<Self::Output, Self::Error> {
                rhs.strict_intersection(self)
            }
        }

        impl<T: $crate::numeric::Element + Clone> Intersection<&$t_rhs> for &$t_lhs {
            type Output = $t_ret;
            type Error = $crate::error::Error;

            #[inline(always)]
            fn strict_intersection(self, rhs: &$t_rhs) -> Result<Self::Output, Self::Error> {
                rhs.strict_intersection(self)
            }
        }
    };
}

commutative_intersection_impl!(HalfInterval<T>, FiniteInterval<T>, FiniteInterval<T>);
commutative_intersection_impl!(FiniteInterval<T>, EnumInterval<T>, EnumInterval<T>);
commutative_intersection_impl!(HalfInterval<T>, EnumInterval<T>, EnumInterval<T>);

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
/// assert_eq!(s.next().unwrap(), Ok(EnumInterval::closed(10, 15)));
/// assert_eq!(s.next().unwrap(), Ok(EnumInterval::closed(20, 25)));
/// assert_eq!(s.next().unwrap(), Ok(EnumInterval::closed(75, 80)));
/// assert_eq!(s.next().unwrap(), Ok(EnumInterval::closed(95, 100)));
/// assert_eq!(s.next(), None);
/// ```
pub struct SetSetIntersection<T, S, I1, I2>
where
    S: Borrow<EnumInterval<T>>,
    I1: Iterator<Item = S>,
    I2: Iterator<Item = S>,
{
    a: itertools::PutBack<I1>,
    b: itertools::PutBack<I2>,
    t: core::marker::PhantomData<T>,
}

impl<T, S, I1, I2> SetSetIntersection<T, S, I1, I2>
where
    S: Borrow<EnumInterval<T>>,
    I1: Iterator<Item = S>,
    I2: Iterator<Item = S>,
{
    /// Creates a new SetSetIntersection Iterator
    ///
    /// If the standard `Set` invariants are not satisfied, behavior is undefined.
    pub fn new<U1, U2>(a: U1, b: U2) -> Self
    where
        S: Borrow<EnumInterval<T>>,
        U1: IntoIterator<Item = S, IntoIter = I1>,
        U2: IntoIterator<Item = S, IntoIter = I2>,
    {
        Self {
            a: itertools::put_back(a),
            b: itertools::put_back(b),
            t: core::marker::PhantomData,
        }
    }
}

impl<T, S, I1, I2> Iterator for SetSetIntersection<T, S, I1, I2>
where
    T: Element + Clone,
    S: Borrow<EnumInterval<T>>,
    I1: Iterator<Item = S>,
    I2: Iterator<Item = S>,
{
    type Item = Result<EnumInterval<T>, crate::error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.a.next()?;
        let b = self.b.next()?;

        let ab = match a.borrow().strict_intersection(b.borrow()) {
            Ok(inner) => inner,
            Err(e) => return Some(Err(e)),
        };

        if !ab.is_empty() {
            // since `a` and `b` intersect, we want to look at the right hand
            // bounds to decide which one (or both) to discard.
            let (_, a_r) = a.borrow().ord_bound_pair().into_raw();
            let (_, b_r) = b.borrow().ord_bound_pair().into_raw();
            if a_r > b_r {
                self.a.put_back(a);
            } else if a_r < b_r {
                self.b.put_back(b);
            }
            Some(Ok(ab))
        } else {
            // since `a` and `b` are disjoint, discard the one with the
            // smallest left hand bound.
            let (l_a, _) = a.borrow().ord_bound_pair().into_raw();
            let (l_b, _) = b.borrow().ord_bound_pair().into_raw();
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
        T: PartialEq + Element + Clone + core::fmt::Debug,
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

    extern crate std;

    #[test]
    fn test_set_set_iter() {
        let a = std::vec![EnumInterval::closed(0, 10), EnumInterval::closed(100, 150)];

        let b = std::vec![
            EnumInterval::closed(5, 15),
            EnumInterval::closed(90, 95),
            EnumInterval::closed(140, 160),
        ];

        let mut it = SetSetIntersection::new(a.iter(), b.iter());

        assert_eq!(it.next(), Some(Ok(EnumInterval::closed(5, 10))));
        assert_eq!(it.next(), Some(Ok(EnumInterval::closed(140, 150))));
        assert_eq!(it.next(), None);

        let mut it = SetSetIntersection::new(a.into_iter(), b.into_iter());

        assert_eq!(it.next(), Some(Ok(EnumInterval::closed(5, 10))));
        assert_eq!(it.next(), Some(Ok(EnumInterval::closed(140, 150))));
        assert_eq!(it.next(), None);
    }
}

use core::borrow::Borrow;

use super::Contains;
use crate::bound::ord::OrdBounded;
use crate::bound::FiniteBound;
use crate::bound::Side::{self, Left, Right};
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::sets::EnumInterval::{self, Finite, Half, Unbounded};
use crate::sets::{FiniteInterval, HalfInterval};

/// Tier-3 helper: build a `FiniteInterval` from a candidate bound
/// pair whose per-bound invariants are already satisfied (taken from
/// validated intervals), evaluating the pair-level satisfiability
/// question. If `lhs` and `rhs` describe a non-empty set, return the
/// corresponding `Bounded`; otherwise return `Empty`.
///
/// # Preconditions
///
/// - **I2 (limit-valid):** each bound's value lies in a totally-
///   ordered subdomain (no NaN). Tripped by `debug_assert!`.
/// - **I4 (discrete-normalized):** for discrete `T`, each bound is
///   already in closed form. Tripped by `debug_assert!` per bound.
///
/// I5 (ordered) is the *question* this helper answers, not a
/// precondition.
///
/// Violating the preconditions yields incorrect results but no
/// undefined behavior; release builds do not check them.
///
/// # Visibility
///
/// `pub(super)` so siblings under [`crate::ops`] (currently
/// `intersection` and `finite::IntoFinite`) can reach it without
/// re-exposing a Tier-3 inherent on `FiniteInterval`. A public
/// inherent proved to be a misuse temptation for callers who weren't
/// actually working with bounds from validated intervals.
#[inline]
pub(super) fn from_normed_pair<T: Element>(
    lhs: FiniteBound<T>,
    rhs: FiniteBound<T>,
) -> FiniteInterval<T> {
    debug_assert!(
        lhs.value().partial_cmp(rhs.value()).is_some(),
        "from_normed_pair: bounds must be comparable (NaN check)"
    );
    debug_assert!(
        lhs.is_closed() || lhs.value().try_adjacent(Side::Right).is_none(),
        "from_normed_pair: lhs must be discrete-normalized to closed"
    );
    debug_assert!(
        rhs.is_closed() || rhs.value().try_adjacent(Side::Left).is_none(),
        "from_normed_pair: rhs must be discrete-normalized to closed"
    );
    if lhs.value() < rhs.value()
        || (lhs.value() == rhs.value() && lhs.is_closed() && rhs.is_closed())
    {
        FiniteInterval::new_assume_valid(lhs, rhs)
    } else {
        FiniteInterval::empty()
    }
}

/// The intersection of two sets.
///
/// ```text
/// {x | x ∈ A ∧ x ∈ B }
/// ```
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). Cannot panic
/// or error given inputs satisfying their type invariants; no
/// `try_*` variant because the operation introduces no logical
/// violation of its own. See [`crate::ops`] for the full tier model.
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

    /// Creates a new `Set` intersection of A and B.
    fn intersection(self, rhs: Rhs) -> Self::Output;
}

impl<T: Element> Intersection<Self> for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Self::empty();
        };

        let Some((rhs_min, rhs_max)) = rhs.into_raw() else {
            return Self::empty();
        };

        // self and rhs already satisfy invariants -> bounds are normalized & comparable
        from_normed_pair(
            FiniteBound::take_max_assume_valid(Left, lhs_min, rhs_min),
            FiniteBound::take_min_assume_valid(Right, lhs_max, rhs_max),
        )
    }
}

impl<T: Element + Clone> Intersection<Self> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return Self::Output::empty();
        };

        let Some((rhs_min, rhs_max)) = rhs.view_raw() else {
            return Self::Output::empty();
        };

        // self and rhs already satisfy invariants -> bounds are normalized & comparable
        from_normed_pair(
            FiniteBound::max_assume_valid(Left, lhs_min, rhs_min).clone(),
            FiniteBound::min_assume_valid(Right, lhs_max, rhs_max).clone(),
        )
    }
}

impl<T: Element> Intersection<HalfInterval<T>> for FiniteInterval<T> {
    type Output = Self;

    #[inline(always)]
    fn intersection(self, rhs: HalfInterval<T>) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.into_raw() else {
            return Self::Output::empty();
        };

        let n = [lhs_min.finite_ord(Left), lhs_max.finite_ord(Right)]
            .into_iter()
            .filter(|bound| rhs.contains(*bound))
            .count();

        if n == 2 {
            // both self bounds are inside rhs -> self is the intersection unchanged
            FiniteInterval::new_assume_valid(lhs_min, lhs_max)
        } else if n == 1 {
            let (rhs_side, rhs_bound) = rhs.into_raw();
            // self and rhs already satisfy invariants
            match rhs_side {
                Left => from_normed_pair(rhs_bound, lhs_max),
                Right => from_normed_pair(lhs_min, rhs_bound),
            }
        } else {
            Self::Output::empty()
        }
    }
}

impl<T: Element + Clone> Intersection<&HalfInterval<T>> for &FiniteInterval<T> {
    type Output = FiniteInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: &HalfInterval<T>) -> Self::Output {
        let Some((lhs_min, lhs_max)) = self.view_raw() else {
            return FiniteInterval::empty();
        };

        let n = [lhs_min.finite_ord(Left), lhs_max.finite_ord(Right)]
            .into_iter()
            .filter(|bound| rhs.contains(*bound))
            .count();

        if n == 2 {
            // both self bounds are inside rhs -> self is the intersection unchanged
            FiniteInterval::new_assume_valid(lhs_min.clone(), lhs_max.clone())
        } else if n == 1 {
            // self and rhs already satisfy invariants
            match rhs.side() {
                Left => from_normed_pair(rhs.finite_bound().clone(), lhs_max.clone()),
                Right => from_normed_pair(lhs_min.clone(), rhs.finite_bound().clone()),
            }
        } else {
            Self::Output::empty()
        }
    }
}

impl<T: Element> Intersection<Self> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                rhs.into()
            } else {
                self.into()
            }
        } else {
            let (lhs_side, lhs_bound) = self.into_raw();
            let (_, rhs_bound) = rhs.into_raw();

            // self and rhs already satisfy invariants
            let finite = match lhs_side {
                Side::Left => from_normed_pair(lhs_bound, rhs_bound),
                Side::Right => from_normed_pair(rhs_bound, lhs_bound),
            };

            EnumInterval::from(finite)
        }
    }
}

impl<T: Element + Clone> Intersection<Self> for &HalfInterval<T> {
    type Output = EnumInterval<T>;

    #[inline(always)]
    fn intersection(self, rhs: Self) -> Self::Output {
        if self.side() == rhs.side() {
            if self.contains(rhs.finite_ord_bound()) {
                rhs.clone().into()
            } else {
                self.clone().into()
            }
        } else if self.contains(rhs.finite_ord_bound()) {
            let lhs = self.finite_bound().clone();
            let rhs = rhs.finite_bound().clone();

            // self and rhs already satisfy invariants
            let result = match self.side() {
                Left => from_normed_pair(lhs, rhs),
                Right => from_normed_pair(rhs, lhs),
            };
            Self::Output::from(result)
        } else {
            Self::Output::empty()
        }
    }
}

macro_rules! dispatch_intersection_impl {
    ($t_lhs:ty, $t_rhs:ty) => {
        impl<T: $crate::numeric::Element> Intersection<$t_rhs> for $t_lhs {
            type Output = EnumInterval<T>;

            #[inline(always)]
            fn intersection(self, rhs: $t_rhs) -> Self::Output {
                match self {
                    Finite(lhs) => lhs.intersection(rhs).into(),
                    Half(lhs) => lhs.intersection(rhs).into(),
                    Unbounded => rhs.into(),
                }
            }
        }

        impl<T: $crate::numeric::Element + Clone> Intersection<&$t_rhs> for &$t_lhs {
            type Output = EnumInterval<T>;

            #[inline(always)]
            fn intersection(self, rhs: &$t_rhs) -> Self::Output {
                match self {
                    Finite(lhs) => lhs.intersection(rhs).into(),
                    Half(lhs) => lhs.intersection(rhs).into(),
                    Unbounded => rhs.clone().into(),
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

            #[inline(always)]
            fn intersection(self, rhs: $t_rhs) -> Self::Output {
                rhs.intersection(self)
            }
        }

        impl<T: $crate::numeric::Element + Clone> Intersection<&$t_rhs> for &$t_lhs {
            type Output = $t_ret;

            #[inline(always)]
            fn intersection(self, rhs: &$t_rhs) -> Self::Output {
                rhs.intersection(self)
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
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(10, 15));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(20, 25));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(75, 80));
/// assert_eq!(s.next().unwrap(), EnumInterval::closed(95, 100));
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
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.a.next()?;
        let b = self.b.next()?;

        let ab = a.borrow().intersection(b.borrow());

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
            Some(ab)
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
            FiniteInterval::closed(0, 100).intersection(FiniteInterval::empty()),
            FiniteInterval::empty()
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
        let expected = EnumInterval::from(y);
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

        assert_eq!(it.next(), Some(EnumInterval::closed(5, 10)));
        assert_eq!(it.next(), Some(EnumInterval::closed(140, 150)));
        assert_eq!(it.next(), None);

        let mut it = SetSetIntersection::new(a, b);

        assert_eq!(it.next(), Some(EnumInterval::closed(5, 10)));
        assert_eq!(it.next(), Some(EnumInterval::closed(140, 150)));
        assert_eq!(it.next(), None);
    }
}

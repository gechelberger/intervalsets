use crate::bound::{FiniteBound, Side};
use crate::numeric::Domain;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};
use crate::Factory;

/// Create a new interval, replacing a bound.
///
/// # Examples
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// assert_eq!(x.with_left_closed(5), [5, 10].into());
/// ```
pub trait Rebound<T>: Sized {
    /// The concrete type of `Set`` to create when replacing bounds.
    type Output;

    /// Creates a `Set`, replacing the left/lower bound.
    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output;

    /// Creates a `Set`, replacing the right/upper bound.
    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output;

    /// Creates a `Set` with a new finite left/lower bound.
    #[inline]
    fn with_left_finite(self, bound: FiniteBound<T>) -> Self::Output {
        self.with_left(Some(bound))
    }

    /// Creates a `Set` with a new finite right/upper bound.
    #[inline]
    fn with_right_finite(self, bound: FiniteBound<T>) -> Self::Output {
        self.with_right(Some(bound))
    }

    /// Creates a `Set` with a new closed left/lower bound.
    #[inline]
    fn with_left_closed(self, bound: T) -> Self::Output {
        self.with_left_finite(FiniteBound::closed(bound))
    }

    /// Creates a `Set` with a new closed right/upper bound.
    #[inline]
    fn with_right_closed(self, bound: T) -> Self::Output {
        self.with_right_finite(FiniteBound::closed(bound))
    }

    /// Creates a `Set` with a new open left/lower bound.
    #[inline]
    fn with_left_open(self, bound: T) -> Self::Output {
        self.with_left_finite(FiniteBound::open(bound))
    }

    /// Creates a `Set` with a new open right/upper bound.
    #[inline]
    fn with_right_open(self, bound: T) -> Self::Output {
        self.with_right_finite(FiniteBound::open(bound))
    }
}

impl<T: Domain> Rebound<T> for FiniteInterval<T> {
    type Output = EnumInterval<T>;

    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        let Self::Bounded(_, rhs) = self else {
            return EnumInterval::Finite(self); // empty
        };

        match bound {
            None => EnumInterval::right_bounded(rhs),
            Some(inner) => EnumInterval::finite(inner, rhs),
        }
    }

    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        let Self::Bounded(lhs, _) = self else {
            return EnumInterval::Finite(self); // empty
        };

        match bound {
            None => EnumInterval::left_bounded(lhs),
            Some(inner) => EnumInterval::finite(lhs, inner),
        }
    }
}

impl<T: Domain> Rebound<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;

    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        match self.side {
            Side::Left => match bound {
                None => EnumInterval::unbounded(),
                Some(inner) => EnumInterval::left_bounded(inner),
            },
            Side::Right => match bound {
                None => EnumInterval::from(self),
                Some(inner) => EnumInterval::finite(inner, self.bound),
            },
        }
    }

    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        match self.side {
            Side::Right => match bound {
                None => EnumInterval::unbounded(),
                Some(inner) => EnumInterval::right_bounded(inner),
            },
            Side::Left => match bound {
                None => EnumInterval::from(self),
                Some(inner) => EnumInterval::finite(self.bound, inner),
            },
        }
    }
}

impl<T: Domain> Rebound<T> for EnumInterval<T> {
    type Output = EnumInterval<T>;

    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.with_left(bound),
            Self::Half(inner) => inner.with_left(bound),
            Self::Unbounded => match bound {
                None => Self::Unbounded,
                Some(inner) => Self::left_bounded(inner),
            },
        }
    }

    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.with_right(bound),
            Self::Half(inner) => inner.with_right(bound),
            Self::Unbounded => match bound {
                None => Self::Unbounded,
                Some(inner) => Self::right_bounded(inner),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_left() {
        let x = FiniteInterval::closed(0, 100);
        assert_eq!(x.clone().with_left(None), EnumInterval::unbound_closed(100));
        assert_eq!(x.clone().with_left_closed(-100), [-100, 100].into());
        assert_eq!(x.clone().with_left_closed(200), ().into());

        let x = HalfInterval::left(FiniteBound::closed(0));
        assert_eq!(x.clone().with_left(None), EnumInterval::Unbounded);
        assert_eq!(
            x.clone().with_left_closed(100),
            EnumInterval::closed_unbound(100)
        );

        let x = HalfInterval::right(FiniteBound::closed(0));
        assert_eq!(x.clone().with_left(None), x.into());
        assert_eq!(x.clone().with_left_closed(0), EnumInterval::closed(0, 0));
        assert_eq!(x.clone().with_left_closed(100), EnumInterval::empty());

        let x = EnumInterval::<i32>::Unbounded;
        assert_eq!(x.clone().with_left(None), EnumInterval::Unbounded);
        assert_eq!(
            x.clone().with_left_closed(0),
            EnumInterval::closed_unbound(0)
        );
    }

    #[test]
    fn test_with_right() {
        let x = FiniteInterval::closed(0, 100);
        assert_eq!(x.clone().with_right(None), EnumInterval::closed_unbound(0));
        assert_eq!(x.clone().with_right_closed(-100), EnumInterval::empty());
        assert_eq!(x.clone().with_right_closed(200), [0, 200].into());

        let x = HalfInterval::left(FiniteBound::closed(0));
        assert_eq!(x.clone().with_right(None), x.into());
        assert_eq!(
            x.clone().with_right_closed(100),
            EnumInterval::closed(0, 100)
        );
        assert_eq!(x.clone().with_right_closed(-100), EnumInterval::empty());

        let x = HalfInterval::right(FiniteBound::closed(0));
        assert_eq!(x.clone().with_right(None), EnumInterval::unbounded());
        assert_eq!(
            x.clone().with_right_closed(100),
            EnumInterval::unbound_closed(100)
        );
        assert_eq!(
            x.clone().with_right_closed(-100),
            EnumInterval::unbound_closed(-100)
        );

        let x = EnumInterval::unbounded();
        assert_eq!(x.clone().with_right(None), x.clone());
        assert_eq!(
            x.clone().with_right_closed(0),
            EnumInterval::unbound_closed(0)
        );
    }
}

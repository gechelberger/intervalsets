use num_traits::Zero;

use crate::bound::{FiniteBound, Side};
use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

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
    type Error: core::error::Error;

    fn with_left_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error>;
    fn with_right_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error>;

    /// Creates a `Set`, replacing the left/lower bound.
    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        self.with_left_strict(bound).unwrap()
    }

    /// Creates a `Set`, replacing the right/upper bound.
    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        self.with_right_strict(bound).unwrap()
    }

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

impl<T: Element + Zero> Rebound<T> for FiniteInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn with_left_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        let Some((_, rhs)) = self.into_raw() else {
            return Ok(Self::Output::empty());
        };

        match bound {
            None => EnumInterval::strict_right_bounded(rhs),
            Some(inner) => EnumInterval::strict_finite(inner, rhs),
        }
    }

    fn with_right_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        let Some((lhs, _)) = self.into_raw() else {
            return Ok(Self::Output::empty()); // empty
        };

        match bound {
            None => EnumInterval::strict_left_bounded(lhs),
            Some(inner) => EnumInterval::strict_finite(lhs, inner),
        }
    }
}

impl<T: Element + Zero> Rebound<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn with_left_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        let (side, current_bound) = self.into_raw();
        match side {
            Side::Left => match bound {
                None => Ok(EnumInterval::unbounded()),
                Some(inner) => EnumInterval::strict_left_bounded(inner),
            },
            Side::Right => match bound {
                // SAFETY: just repacking
                None => unsafe { Ok(EnumInterval::from(Self::new_unchecked(side, current_bound))) },
                Some(inner) => EnumInterval::strict_finite(inner, current_bound),
            },
        }
    }

    fn with_right_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        let (side, current_bound) = self.into_raw();
        match side {
            Side::Right => match bound {
                None => Ok(EnumInterval::unbounded()),
                Some(inner) => EnumInterval::strict_right_bounded(inner),
            },
            Side::Left => match bound {
                None => unsafe {
                    // SAFETY: just putting it back together
                    Ok(EnumInterval::from(Self::new_unchecked(side, current_bound)))
                },
                Some(inner) => EnumInterval::strict_finite(current_bound, inner),
            },
        }
    }
}

impl<T: Element + Zero> Rebound<T> for EnumInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn with_left_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(inner) => inner.with_left_strict(bound),
            Self::Half(inner) => inner.with_left_strict(bound),
            Self::Unbounded => match bound {
                None => Ok(Self::Unbounded),
                Some(inner) => Self::strict_left_bounded(inner),
            },
        }
    }

    fn with_right_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Finite(inner) => inner.with_right_strict(bound),
            Self::Half(inner) => inner.with_right_strict(bound),
            Self::Unbounded => match bound {
                None => Ok(Self::Unbounded),
                Some(inner) => Self::strict_right_bounded(inner),
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

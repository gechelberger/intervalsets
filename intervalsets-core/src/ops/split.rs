use super::Contains;
use crate::bound::{FiniteBound, Side};
use crate::error::Error;
use crate::factory::TrySatisfyFiniteInterval;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Split a Set into two disjoint subsets, fully covering the original.
///
/// `at` provides the new bounds where the set should be split.
///
/// # Contract
///
/// Tier 3 (`try_*` + panicking sugar).
/// [`try_split`](Self::try_split) returns `Err(Self::Error)` on
/// logical violation (typically: a non-comparable user-supplied
/// `at`, e.g. NaN); it never panics. [`split`](Self::split) is the
/// panicking unwrap of `try_split`. See [`crate::ops`] for the full
/// tier model.
///
/// # Example
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x = FiniteInterval::closed(0, 10);
/// let (left, right) = x.split(5, Side::Left);
/// assert_eq!(left, FiniteInterval::closed(0, 5));
/// assert_eq!(right, FiniteInterval::closed(6, 10));
/// ```
pub trait Split<T>: Sized {
    /// The type of `Set` to create when split.
    type Output;
    type Error: core::error::Error;

    /// Creates two disjoint subsets with elements partitioned by `at`.
    ///
    /// # Panics
    ///
    /// Panic if `at` is not comparable.
    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        self.try_split(at, closed).unwrap()
    }

    fn try_split(self, at: T, closed: Side) -> Result<(Self::Output, Self::Output), Self::Error>;
}

fn split_bounds_at<T: Element + Clone>(
    at: T,
    closed: Side,
) -> Result<(FiniteBound<T>, FiniteBound<T>), Error> {
    Ok(match closed {
        Side::Left => (
            FiniteBound::try_closed(at.clone())?,
            FiniteBound::try_open(at)?,
        ),
        Side::Right => (
            FiniteBound::try_open(at.clone())?,
            FiniteBound::try_closed(at)?,
        ),
    })
}

impl<T: Element + Clone> Split<T> for FiniteInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn try_split(self, at: T, closed: Side) -> Result<(Self::Output, Self::Output), Self::Error> {
        let Some((min, max)) = self.into_raw() else {
            return Ok((Self::empty(), Self::empty()));
        };

        if !min.try_contains(Side::Left, &at)? {
            let repacked = Self::new_assume_valid(min, max);
            return Ok((Self::empty(), repacked));
        }

        if !max.try_contains(Side::Right, &at)? {
            let repacked = Self::new_assume_valid(min, max);
            return Ok((repacked, Self::empty()));
        }

        let (lhs_max, rhs_min) = split_bounds_at(at, closed)?;
        // try_satisfy_bounds: splitting at a boundary value with the
        // boundary kind on one side produces a degenerate empty side
        // (e.g. [min, min) when closed = Right and at = min). That's
        // the correct answer, not an error.
        let split_left = Self::try_satisfy_bounds(min, lhs_max)?;
        let split_right = Self::try_satisfy_bounds(rhs_min, max)?;
        Ok((split_left, split_right))
    }
}

impl<T: Element + Clone> Split<T> for HalfInterval<T> {
    type Output = EnumInterval<T>;
    type Error = crate::error::Error;

    fn try_split(self, at: T, closed: Side) -> Result<(Self::Output, Self::Output), Self::Error> {
        if !self.contains(&at) {
            return match self.side() {
                Side::Left => Ok((Self::Output::empty(), self.into())),
                Side::Right => Ok((self.into(), Self::Output::empty())),
            };
        }

        let (lhs_max, rhs_min) = split_bounds_at(at, closed)?;
        let (side, bound) = self.into_raw();
        // try_satisfy_bounds: a split exactly at the half-bounded interval's
        // own boundary produces a degenerate empty side, which is the
        // correct answer (not an error).
        match side {
            Side::Left => {
                let left = FiniteInterval::try_satisfy_bounds(bound, lhs_max)?;
                let right = HalfInterval::try_new(side, rhs_min)?;
                Ok((left.into(), right.into()))
            }
            Side::Right => {
                let left = HalfInterval::try_new(side, lhs_max)?;
                let right = FiniteInterval::try_satisfy_bounds(rhs_min, bound)?;
                Ok((left.into(), right.into()))
            }
        }
    }
}

impl<T: Element + Clone> Split<T> for EnumInterval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn try_split(self, at: T, closed: Side) -> Result<(Self::Output, Self::Output), Self::Error> {
        match self {
            Self::Finite(inner) => inner
                .try_split(at, closed)
                .map(|(l, r)| (l.into(), r.into())),
            Self::Half(inner) => inner.try_split(at, closed),
            Self::Unbounded => {
                let (lhs_max, rhs_min) = split_bounds_at(at, closed)?;
                let left = HalfInterval::try_new(Side::Right, lhs_max)?;
                let right = HalfInterval::try_new(Side::Left, rhs_min)?;
                Ok((left.into(), right.into()))
            }
        }
    }

    fn split(self, at: T, closed: Side) -> (Self::Output, Self::Output) {
        match self {
            Self::Finite(inner) => {
                let (left, right) = inner.split(at, closed);
                (left.into(), right.into())
            }
            Self::Half(inner) => inner.split(at, closed),
            Self::Unbounded => {
                // `split` is the panicking sibling of `try_split`; an
                // invalid `at` is documented to panic. `.unwrap()` is
                // the design, not an "should never fail" claim.
                let (l_max, r_min) = split_bounds_at(at, closed).unwrap();
                (
                    HalfInterval::right(l_max).into(),
                    HalfInterval::left(r_min).into(),
                )
            }
        }
    }
}

impl<T: Element + Clone> Split<T> for MaybeDisjoint<T> {
    type Output = MaybeDisjoint<T>;
    type Error = Error;

    fn try_split(self, at: T, closed: Side) -> Result<(Self::Output, Self::Output), Self::Error> {
        match self {
            Self::Connected(iv) => {
                let (l, r) = iv.try_split(at, closed)?;
                Ok((
                    MaybeDisjoint::from_interval(l),
                    MaybeDisjoint::from_interval(r),
                ))
            }
            Self::Disjoint(a, b) => {
                let (a_l, a_r) = a.try_split(at.clone(), closed)?;
                let (b_l, b_r) = b.try_split(at, closed)?;
                Ok((
                    MaybeDisjoint::from_pair(a_l, b_l),
                    MaybeDisjoint::from_pair(a_r, b_r),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    fn md_pair(a: EnumInterval<i32>, b: EnumInterval<i32>) -> MaybeDisjoint<i32> {
        MaybeDisjoint::from_pair(a, b)
    }

    // ---- MaybeDisjoint::Connected delegates to the inner split ----

    #[test]
    fn md_connected_split_middle() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0_i32, 10));
        let (left, right) = md.split(5, Side::Left);
        assert_eq!(
            left,
            MaybeDisjoint::from_interval(EnumInterval::closed(0, 5))
        );
        assert_eq!(
            right,
            MaybeDisjoint::from_interval(EnumInterval::closed(6, 10))
        );
    }

    #[test]
    fn md_empty_split_yields_two_empties() {
        let md = MaybeDisjoint::<i32>::empty();
        let (left, right) = md.split(5, Side::Left);
        assert_eq!(left, MaybeDisjoint::empty());
        assert_eq!(right, MaybeDisjoint::empty());
    }

    // ---- MaybeDisjoint::Disjoint, split position cases ----

    #[test]
    fn md_disjoint_split_before_first_piece() {
        // at < a → left empty, right keeps both pieces
        let md = md_pair(EnumInterval::closed(10, 20), EnumInterval::closed(30, 40));
        let (left, right) = md.clone().split(5, Side::Left);
        assert_eq!(left, MaybeDisjoint::empty());
        assert_eq!(right, md);
    }

    #[test]
    fn md_disjoint_split_inside_first_piece() {
        // at inside a → left has partial-a, right has rest-of-a + b
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(5, Side::Left);
        assert_eq!(
            left,
            MaybeDisjoint::from_interval(EnumInterval::closed(0, 5))
        );
        assert_eq!(
            right,
            md_pair(EnumInterval::closed(6, 10), EnumInterval::closed(20, 30))
        );
    }

    #[test]
    fn md_disjoint_split_in_gap() {
        // at in the gap → left = a, right = b, no piece splitting
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(15, Side::Left);
        assert_eq!(
            left,
            MaybeDisjoint::from_interval(EnumInterval::closed(0, 10))
        );
        assert_eq!(
            right,
            MaybeDisjoint::from_interval(EnumInterval::closed(20, 30))
        );
    }

    #[test]
    fn md_disjoint_split_inside_second_piece() {
        // at inside b → left has a + partial-b, right has rest-of-b
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(25, Side::Left);
        assert_eq!(
            left,
            md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 25))
        );
        assert_eq!(
            right,
            MaybeDisjoint::from_interval(EnumInterval::closed(26, 30))
        );
    }

    #[test]
    fn md_disjoint_split_after_second_piece() {
        // at > b → left keeps both pieces, right empty
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.clone().split(35, Side::Left);
        assert_eq!(left, md);
        assert_eq!(right, MaybeDisjoint::empty());
    }

    // ---- closed-Right boundary semantics ----

    #[test]
    fn md_disjoint_split_at_first_piece_right_boundary_closed_left() {
        // closed=Left → `at` belongs to the left side. Split [0,10]∪[20,30] at 10:
        // a_left = [0,10], a_right = empty (after 10 is empty in a's domain),
        // b_left = empty, b_right = [20, 30].
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(10, Side::Left);
        assert_eq!(
            left,
            MaybeDisjoint::from_interval(EnumInterval::closed(0, 10))
        );
        assert_eq!(
            right,
            MaybeDisjoint::from_interval(EnumInterval::closed(20, 30))
        );
    }

    #[test]
    fn md_disjoint_split_at_first_piece_right_boundary_closed_right() {
        // closed=Right → `at` belongs to the right side. Split [0,10]∪[20,30] at 10:
        // a_left = [0, 9], a_right = [10, 10], plus b on the right side.
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(10, Side::Right);
        assert_eq!(
            left,
            MaybeDisjoint::from_interval(EnumInterval::closed(0, 9))
        );
        assert_eq!(
            right,
            md_pair(EnumInterval::closed(10, 10), EnumInterval::closed(20, 30))
        );
    }

    // ---- closed under MaybeDisjoint: both sides always fit ----

    #[test]
    fn md_disjoint_split_inside_each_piece_distributes_correctly() {
        // Catches the case where `at` is between the pieces' inner edges,
        // ensuring the algorithm doesn't accidentally merge or split wrong.
        // At at=5 (inside first piece), expected: left=[0,5], right=[6,10]∪[20,30].
        let md = md_pair(EnumInterval::closed(0, 10), EnumInterval::closed(20, 30));
        let (left, right) = md.split(5, Side::Left);
        // left has 1 piece, right has 2 — both fit in MaybeDisjoint.
        assert_eq!(left.into_iter().count(), 1);
        let right_pieces: std::vec::Vec<_> = right.into_iter().collect();
        assert_eq!(right_pieces.len(), 2);
    }

    // ---- try_split error propagation ----

    #[test]
    fn md_try_split_propagates_nan_error() {
        // NaN is not Element-valid for f64; the inner try_split surfaces
        // the error and MD's impl propagates it.
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0.0_f64, 10.0));
        let result = md.try_split(f64::NAN, Side::Left);
        assert!(result.is_err());
    }

    extern crate std;
}

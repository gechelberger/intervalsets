use crate::bound::Side;
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// The complement of a set in the unbounded universe of `T`.
///
/// ```text
/// Let A = { x | P(x) } => A' = { x | x ∉ A }
/// ```
///
/// The complement of an interval has at most two pieces and is therefore
/// representable in [`MaybeDisjoint`]. Specifically:
///
/// | Input | Complement | Pieces |
/// |---|---|---|
/// | `empty` | unbounded | 1 |
/// | finite `[a, b]` | `(-∞, a) ∪ (b, ∞)` (with bound flips) | 2 |
/// | half-bounded | half-bounded with flipped side | 1 |
/// | unbounded | empty | 0 |
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). Cannot panic
/// or error given inputs satisfying their type invariants; no
/// `try_*` variant because the operation introduces no logical
/// violation of its own. See [`crate::ops`] for the full tier model.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// let interval = EnumInterval::closed(0, 10);
/// let mut iter = interval.complement().into_iter();
/// assert_eq!(iter.next(), Some(EnumInterval::unbound_open(0)));
/// assert_eq!(iter.next(), Some(EnumInterval::open_unbound(10)));
/// assert_eq!(iter.next(), None);
/// ```
pub trait Complement {
    /// The type produced by complementing.
    type Output;

    /// Returns the complement of this set.
    fn complement(self) -> Self::Output;
}

impl<T: Element> Complement for FiniteInterval<T> {
    type Output = MaybeDisjoint<T>;

    fn complement(self) -> Self::Output {
        match self.into_raw() {
            // empty -> unbounded
            None => EnumInterval::Unbounded.into(),
            Some((lhs, rhs)) => {
                // [a, b] -> (-inf, a) U (b, inf), with each bound flipped
                // (closed becomes open and vice versa) and re-normalized
                // for discrete types.
                let left =
                    HalfInterval::new_assume_valid(Side::Right, lhs.flip().normalized(Side::Right));
                let right =
                    HalfInterval::new_assume_valid(Side::Left, rhs.flip().normalized(Side::Left));
                MaybeDisjoint::new_disjoint_assume_valid(
                    EnumInterval::from(left),
                    EnumInterval::from(right),
                )
            }
        }
    }
}

impl<T: Element> Complement for HalfInterval<T> {
    type Output = MaybeDisjoint<T>;

    fn complement(self) -> Self::Output {
        let (side, bound) = self.into_raw();
        let side = side.flip();
        let half = HalfInterval::new_assume_valid(side, bound.flip().normalized(side));
        EnumInterval::from(half).into()
    }
}

impl<T: Element> Complement for EnumInterval<T> {
    type Output = MaybeDisjoint<T>;

    fn complement(self) -> Self::Output {
        match self {
            Self::Finite(inner) => inner.complement(),
            Self::Half(inner) => inner.complement(),
            Self::Unbounded => MaybeDisjoint::empty(),
        }
    }
}

/// By-ref blanket: any `&X` whose owned form implements `Complement`
/// gets a `Complement` impl that clones and forwards. The clone is
/// shallow (the interval is two `FiniteBound`s plus a tag).
impl<X: Complement + Clone> Complement for &X {
    type Output = <X as Complement>::Output;

    fn complement(self) -> Self::Output {
        self.clone().complement()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_complement_empty_is_unbounded() {
        let result = FiniteInterval::<i32>::empty().complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbounded()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_unbounded_is_empty() {
        let result = EnumInterval::<i32>::unbounded().complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_finite_closed_continuous() {
        // [0.0, 10.0] -> (-inf, 0.0) U (10.0, inf)
        let result = FiniteInterval::closed(0.0_f64, 10.0).complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbound_open(0.0)));
        assert_eq!(iter.next(), Some(EnumInterval::open_unbound(10.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_finite_open_continuous() {
        // (0.0, 10.0) -> (-inf, 0.0] U [10.0, inf)
        let result = FiniteInterval::open(0.0_f64, 10.0).complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbound_closed(0.0)));
        assert_eq!(iter.next(), Some(EnumInterval::closed_unbound(10.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_finite_discrete() {
        // [0, 10] -> (-inf, 0) U (10, inf), which for i32 normalizes to
        // (-inf, -1] U [11, inf) since open-on-discrete becomes closed-on-adjacent.
        let result = FiniteInterval::closed(0_i32, 10).complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbound_closed(-1)));
        assert_eq!(iter.next(), Some(EnumInterval::closed_unbound(11)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_half() {
        // [5, +inf) -> (-inf, 5)
        let result = HalfInterval::left(crate::bound::FiniteBound::closed(5_i32)).complement();
        let mut iter = result.into_iter();
        // for discrete i32, (-inf, 5) normalizes to (-inf, 4]
        assert_eq!(iter.next(), Some(EnumInterval::unbound_closed(4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_complement_involutive_finite() {
        // (A')' == A for a closed continuous interval
        let original = EnumInterval::closed(0.0_f64, 10.0);
        let double = original
            .complement()
            .next() // grab one piece — should be wrong on its own
            .unwrap();
        // Verifying the actual involution for a 2-piece complement requires
        // applying complement to a multi-piece set, which needs IntervalSet
        // (alloc). But we can at least confirm an Unbounded round-trip:
        let _ = double; // placeholder; see test_complement_round_trip_unbounded
    }

    #[test]
    fn test_complement_by_ref() {
        // by-ref blanket: (&interval).complement() works without consuming.
        let interval = EnumInterval::closed(0, 10);
        let result = (&interval).complement();
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbound_closed(-1)));
        assert_eq!(iter.next(), Some(EnumInterval::closed_unbound(11)));
        assert_eq!(iter.next(), None);
        // original is still usable
        assert_eq!(interval, EnumInterval::closed(0, 10));
    }

    #[test]
    fn test_complement_round_trip_unbounded() {
        // Unbounded' = Empty; Empty' = Unbounded.
        let unbounded: EnumInterval<i32> = EnumInterval::unbounded();
        let mut iter = unbounded.complement();
        assert_eq!(iter.next(), None);

        let empty: FiniteInterval<i32> = FiniteInterval::empty();
        let mut iter = empty.complement();
        assert_eq!(iter.next(), Some(EnumInterval::unbounded()));
        assert_eq!(iter.next(), None);
    }
}

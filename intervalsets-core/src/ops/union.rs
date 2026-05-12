use crate::numeric::Element;
use crate::ops::MergeConnected;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// The (possibly disjoint) union of two intervals.
///
/// ```text
/// { x | x ∈ A ∨ x ∈ B }
/// ```
///
/// The union of two intervals has at most two pieces (connected →
/// 1 piece, disjoint → 2 pieces) and is therefore representable in
/// [`MaybeDisjoint`] without allocation.
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). Cannot panic
/// or error given inputs satisfying their type invariants; no
/// `try_*` variant because the operation introduces no logical
/// violation of its own. The `T: Ord` bound on impls is a
/// stronger-guarantee policy choice (see `numeric.rs`), independent
/// of fallibility. See [`crate::ops`] for the full tier model.
///
/// # Example
/// ```
/// use intervalsets_core::prelude::*;
///
/// // connected -> single interval
/// let connected = EnumInterval::closed(0, 10).union(EnumInterval::closed(5, 15));
/// assert_eq!(connected.into_interval(), Some(EnumInterval::closed(0, 15)));
///
/// // disjoint -> two pieces, ordered ascending
/// let disjoint = EnumInterval::closed(0, 5).union(EnumInterval::closed(10, 15));
/// let mut pieces = disjoint.into_iter();
/// assert_eq!(pieces.next(), Some(EnumInterval::closed(0, 5)));
/// assert_eq!(pieces.next(), Some(EnumInterval::closed(10, 15)));
/// assert_eq!(pieces.next(), None);
/// ```
pub trait Union<Rhs = Self> {
    /// The type produced by the union operation.
    type Output;

    /// Returns the union of `self` and `rhs`.
    fn union(self, rhs: Rhs) -> Self::Output;
}

// All Union impls take T: Element + Ord + Clone. Ord is needed to
// order the pieces when the inputs are disjoint; Clone is needed
// because we use the by-ref MergeConnected to avoid consuming the operands
// until we know whether they merge.
macro_rules! union_via_merge {
    ($lhs:ty, $rhs:ty) => {
        impl<T> Union<$rhs> for $lhs
        where
            T: Element + Ord + Clone,
        {
            type Output = MaybeDisjoint<T>;

            fn union(self, rhs: $rhs) -> Self::Output {
                // Try to merge first; if connected, return a single piece.
                // Otherwise, order the operands and return a disjoint pair.
                match (&self).merge_connected(&rhs) {
                    Some(merged) => EnumInterval::from(merged).into(),
                    None => {
                        let lhs: EnumInterval<T> = self.into();
                        let rhs: EnumInterval<T> = rhs.into();
                        if lhs <= rhs {
                            MaybeDisjoint::new_disjoint_assume_valid(lhs, rhs)
                        } else {
                            MaybeDisjoint::new_disjoint_assume_valid(rhs, lhs)
                        }
                    }
                }
            }
        }
    };
}

union_via_merge!(FiniteInterval<T>, FiniteInterval<T>);
union_via_merge!(HalfInterval<T>, HalfInterval<T>);
union_via_merge!(FiniteInterval<T>, HalfInterval<T>);
union_via_merge!(HalfInterval<T>, FiniteInterval<T>);
union_via_merge!(EnumInterval<T>, FiniteInterval<T>);
union_via_merge!(EnumInterval<T>, HalfInterval<T>);
union_via_merge!(EnumInterval<T>, EnumInterval<T>);
union_via_merge!(FiniteInterval<T>, EnumInterval<T>);
union_via_merge!(HalfInterval<T>, EnumInterval<T>);

// By-ref Union: specialized per LHS x RHS pair so the connected case
// avoids cloning entirely (the by-ref MergeConnected returns an owned merged
// value). Clones only happen when falling into the disjoint branch,
// which needs owned EnumInterval values for MaybeDisjoint::Disjoint.
macro_rules! union_via_merge_ref {
    ($lhs:ty, $rhs:ty) => {
        impl<T> Union<&$rhs> for &$lhs
        where
            T: Element + Ord + Clone,
        {
            type Output = MaybeDisjoint<T>;

            fn union(self, rhs: &$rhs) -> Self::Output {
                match self.merge_connected(rhs) {
                    Some(merged) => EnumInterval::from(merged).into(),
                    None => {
                        let lhs: EnumInterval<T> = self.clone().into();
                        let rhs: EnumInterval<T> = rhs.clone().into();
                        if lhs <= rhs {
                            MaybeDisjoint::new_disjoint_assume_valid(lhs, rhs)
                        } else {
                            MaybeDisjoint::new_disjoint_assume_valid(rhs, lhs)
                        }
                    }
                }
            }
        }
    };
}

union_via_merge_ref!(FiniteInterval<T>, FiniteInterval<T>);
union_via_merge_ref!(HalfInterval<T>, HalfInterval<T>);
union_via_merge_ref!(FiniteInterval<T>, HalfInterval<T>);
union_via_merge_ref!(HalfInterval<T>, FiniteInterval<T>);
union_via_merge_ref!(EnumInterval<T>, FiniteInterval<T>);
union_via_merge_ref!(EnumInterval<T>, HalfInterval<T>);
union_via_merge_ref!(EnumInterval<T>, EnumInterval<T>);
union_via_merge_ref!(FiniteInterval<T>, EnumInterval<T>);
union_via_merge_ref!(HalfInterval<T>, EnumInterval<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_union_connected_finite() {
        let result = FiniteInterval::closed(0, 10).union(FiniteInterval::closed(5, 15));
        assert_eq!(result.into_interval(), Some(EnumInterval::closed(0, 15)));
    }

    #[test]
    fn test_union_disjoint_finite_ordered() {
        let result = FiniteInterval::closed(0, 5).union(FiniteInterval::closed(10, 15));
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::closed(0, 5)));
        assert_eq!(iter.next(), Some(EnumInterval::closed(10, 15)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_union_disjoint_finite_reversed_inputs() {
        // Same disjoint pieces, lhs > rhs in input order — output still ascending.
        let result = FiniteInterval::closed(10, 15).union(FiniteInterval::closed(0, 5));
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::closed(0, 5)));
        assert_eq!(iter.next(), Some(EnumInterval::closed(10, 15)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_union_with_empty() {
        // empty ∪ A = A, A ∪ empty = A
        let a = FiniteInterval::closed(0, 10);
        let e = FiniteInterval::<i32>::empty();
        assert_eq!(
            a.union(e).into_interval(),
            Some(EnumInterval::closed(0, 10))
        );
        assert_eq!(
            e.union(a).into_interval(),
            Some(EnumInterval::closed(0, 10))
        );
    }

    #[test]
    fn test_union_half_with_finite_connecting() {
        // [0, 10] ∪ [5, ∞) = [0, ∞)
        let f = FiniteInterval::closed(0, 10);
        let h = HalfInterval::closed_unbound(5);
        assert_eq!(
            f.union(h).into_interval(),
            Some(EnumInterval::closed_unbound(0))
        );
    }

    #[test]
    fn test_union_two_halves_opposite_connecting() {
        // (-∞, 5] ∪ [3, ∞) = (-∞, ∞)
        let a = HalfInterval::unbound_closed(5);
        let b = HalfInterval::closed_unbound(3);
        assert_eq!(a.union(b).into_interval(), Some(EnumInterval::unbounded()));
    }

    #[test]
    fn test_union_unbounded_absorbs() {
        let u = EnumInterval::<i32>::unbounded();
        let a = EnumInterval::closed(0, 10);
        assert_eq!(u.union(a).into_interval(), Some(EnumInterval::unbounded()));
    }

    #[test]
    fn test_union_by_ref() {
        let a = EnumInterval::closed(0, 10);
        let b = EnumInterval::closed(5, 15);
        let result = (&a).union(&b);
        assert_eq!(result.into_interval(), Some(EnumInterval::closed(0, 15)));
        // originals still usable
        assert_eq!(a, EnumInterval::closed(0, 10));
        assert_eq!(b, EnumInterval::closed(5, 15));
    }

    #[test]
    fn test_union_disjoint_half_and_finite() {
        // (-∞, 0] ∪ [10, 20] -> two disjoint pieces, ordered ascending
        let h = EnumInterval::unbound_closed(0);
        let f = EnumInterval::closed(10, 20);
        let result = h.union(f);
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::unbound_closed(0)));
        assert_eq!(iter.next(), Some(EnumInterval::closed(10, 20)));
        assert_eq!(iter.next(), None);
    }
}

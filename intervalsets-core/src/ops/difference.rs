use crate::disjoint::MaybeDisjoint;
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::ops::{Complement, Intersection};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// The set difference `A \ B` — elements in `A` but not in `B`.
///
/// ```text
/// { x | x ∈ A ∧ x ∉ B }
/// ```
///
/// `A \ B` of two intervals has at most two pieces (when `B` lies in
/// the interior of `A`, splitting `A` in two) and is therefore
/// representable in [`MaybeDisjoint`] without allocation.
///
/// Difference is **not** commutative.
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
/// // B in interior of A -> 2 pieces
/// let split = EnumInterval::closed(0, 100).difference(EnumInterval::closed(40, 60));
/// let mut pieces = split.into_iter();
/// assert_eq!(pieces.next(), Some(EnumInterval::closed_open(0, 40)));
/// assert_eq!(pieces.next(), Some(EnumInterval::open_closed(60, 100)));
/// assert_eq!(pieces.next(), None);
///
/// // B overlaps one side of A -> 1 piece
/// let trimmed = EnumInterval::closed(0, 100).difference(EnumInterval::closed(50, 200));
/// assert_eq!(
///     trimmed.into_interval(),
///     Some(EnumInterval::closed_open(0, 50))
/// );
/// ```
pub trait Difference<Rhs = Self> {
    /// The type produced by the difference operation.
    type Output;

    /// Returns the elements of `self` that are not in `rhs`.
    fn difference(self, rhs: Rhs) -> Self::Output;
}

// All Difference impls compute A \ B = A ∩ B'. B' has at most two
// pieces (via Complement); intersecting each with A gives at most one
// piece, so the result has at most two pieces total. Bound: T must be
// Element + Clone since we intersect lhs with each complement piece.
macro_rules! difference_via_complement {
    ($lhs:ty, $rhs:ty) => {
        impl<T> Difference<$rhs> for $lhs
        where
            T: Element + Clone,
        {
            type Output = MaybeDisjoint<T>;

            #[cfg_attr(all(feature = "panic-free-check", not(debug_assertions)), no_panic::no_panic)]
            fn difference(self, rhs: $rhs) -> Self::Output {
                let lhs: EnumInterval<T> = self.into();
                let mut pieces = rhs.complement().into_iter();
                let first = match pieces.next() {
                    // B is unbounded -> complement is empty -> A \ B is empty
                    None => return MaybeDisjoint::empty(),
                    Some(p) => lhs.clone().intersection(p),
                };
                let second = match pieces.next() {
                    // B's complement was a single piece (B was empty
                    // or half-bounded); the result is the single
                    // intersection.
                    None => return MaybeDisjoint::from(first),
                    Some(p) => lhs.intersection(p),
                };

                // B's complement was two pieces; combine the two
                // intersections, dropping any empty results.
                match (first.is_empty(), second.is_empty()) {
                    (true, true) => MaybeDisjoint::empty(),
                    (false, true) => MaybeDisjoint::from(first),
                    (true, false) => MaybeDisjoint::from(second),
                    (false, false) => (first, second).into(),
                }
            }
        }
    };
}

difference_via_complement!(FiniteInterval<T>, FiniteInterval<T>);
difference_via_complement!(HalfInterval<T>, HalfInterval<T>);
difference_via_complement!(FiniteInterval<T>, HalfInterval<T>);
difference_via_complement!(HalfInterval<T>, FiniteInterval<T>);
difference_via_complement!(EnumInterval<T>, FiniteInterval<T>);
difference_via_complement!(EnumInterval<T>, HalfInterval<T>);
difference_via_complement!(EnumInterval<T>, EnumInterval<T>);
difference_via_complement!(FiniteInterval<T>, EnumInterval<T>);
difference_via_complement!(HalfInterval<T>, EnumInterval<T>);

/// By-ref blanket: any `(&X, &Y)` whose owned forms have a
/// `Difference<Y>` impl on `X` gets a `Difference<&Y>` impl on
/// `&X` that clones and forwards.
impl<X, Y> Difference<&Y> for &X
where
    X: Difference<Y> + Clone,
    Y: Clone,
{
    type Output = <X as Difference<Y>>::Output;

    #[cfg_attr(all(feature = "panic-free-check", not(debug_assertions)), no_panic::no_panic)]
    fn difference(self, rhs: &Y) -> Self::Output {
        self.clone().difference(rhs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_difference_b_in_interior_splits_a() {
        // [0, 100] \ [40, 60] = [0, 40) ∪ (60, 100]
        let result =
            FiniteInterval::closed(0.0_f64, 100.0).difference(FiniteInterval::closed(40.0, 60.0));
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::closed_open(0.0, 40.0)));
        assert_eq!(iter.next(), Some(EnumInterval::open_closed(60.0, 100.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_difference_b_overlaps_one_side() {
        // [0, 100] \ [50, 200] = [0, 50)
        let result = FiniteInterval::closed(0.0_f64, 100.0)
            .difference(FiniteInterval::closed(50.0, 200.0));
        assert_eq!(
            result.into_interval(),
            Some(EnumInterval::closed_open(0.0, 50.0))
        );
    }

    #[test]
    fn test_difference_b_contains_a_is_empty() {
        // [10, 20] \ [0, 100] = ∅
        let result =
            FiniteInterval::closed(10.0_f64, 20.0).difference(FiniteInterval::closed(0.0, 100.0));
        assert_eq!(result.into_interval(), Some(EnumInterval::empty()));
    }

    #[test]
    fn test_difference_b_disjoint_returns_a() {
        // [0, 10] \ [50, 60] = [0, 10]
        let result =
            FiniteInterval::closed(0_i32, 10).difference(FiniteInterval::closed(50, 60));
        assert_eq!(result.into_interval(), Some(EnumInterval::closed(0, 10)));
    }

    #[test]
    fn test_difference_a_minus_empty_is_a() {
        // [0, 10] \ ∅ = [0, 10]
        let result =
            FiniteInterval::closed(0_i32, 10).difference(FiniteInterval::empty());
        assert_eq!(result.into_interval(), Some(EnumInterval::closed(0, 10)));
    }

    #[test]
    fn test_difference_empty_minus_anything_is_empty() {
        let result =
            FiniteInterval::<i32>::empty().difference(FiniteInterval::closed(0, 10));
        assert_eq!(result.into_interval(), Some(EnumInterval::empty()));
    }

    #[test]
    fn test_difference_a_minus_unbounded_is_empty() {
        // A \ (-∞, ∞) = ∅
        let result = EnumInterval::closed(0_i32, 10).difference(EnumInterval::unbounded());
        assert_eq!(result.into_interval(), Some(EnumInterval::empty()));
    }

    #[test]
    fn test_difference_half_minus_finite() {
        // [0, ∞) \ [3, 7] = [0, 3) ∪ (7, ∞)
        let result =
            HalfInterval::closed_unbound(0.0_f64).difference(FiniteInterval::closed(3.0, 7.0));
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::closed_open(0.0, 3.0)));
        assert_eq!(iter.next(), Some(EnumInterval::open_unbound(7.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_difference_by_ref() {
        let a = EnumInterval::closed(0, 100);
        let b = EnumInterval::closed(40, 60);
        let result = (&a).difference(&b);
        let mut iter = result.into_iter();
        assert_eq!(iter.next(), Some(EnumInterval::closed(0, 39)));
        assert_eq!(iter.next(), Some(EnumInterval::closed(61, 100)));
        assert_eq!(iter.next(), None);
        // originals still usable
        assert_eq!(a, EnumInterval::closed(0, 100));
        assert_eq!(b, EnumInterval::closed(40, 60));
    }

    #[test]
    fn test_difference_finite_minus_half() {
        // [0, 100] \ [50, ∞) = [0, 50)
        let result =
            FiniteInterval::closed(0.0_f64, 100.0).difference(HalfInterval::closed_unbound(50.0));
        assert_eq!(
            result.into_interval(),
            Some(EnumInterval::closed_open(0.0, 50.0))
        );
    }
}

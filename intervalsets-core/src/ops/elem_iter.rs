//! Iterate the discrete element values of an interval or disjoint pair.
//!
//! [`IntoElementIterator`] is the trait entry point: any owned set type
//! that knows how to enumerate its elements implements it. [`Elements`]
//! does the per-step walk over a single interval via
//! [`Element::try_adjacent`](crate::numeric::Element::try_adjacent), and
//! [`DisjointElements`] composes it across the at-most-two pieces of a
//! [`MaybeDisjoint`](crate::sets::MaybeDisjoint).
//!
//! # Why not [`IntoIterator`]?
//!
//! [`IntervalSet`](https://docs.rs/intervalsets/latest/intervalsets/struct.IntervalSet.html)
//! has two natural iteration semantics — interval-wise (yielding pieces)
//! and element-wise (yielding `T`). [`IntoIterator`] is reserved for the
//! interval-wise reading; element-wise iteration uses
//! [`IntoElementIterator`] so the two don't compete.
//!
//! # Why `T: DiscreteElement`?
//!
//! `Element::try_adjacent` is what advances the cursor. On continuous
//! types it returns `None` unconditionally, so iterating over a
//! continuous interval would produce an always-empty walk —
//! structurally ill-defined. The `DiscreteElement` bound
//! (`Element<Kind = DiscreteKind>`) catches this at compile time
//! rather than letting users construct iterators that yield nothing.
//!
//! The bound is **just** `DiscreteElement` — `PartialOrd` (carried by
//! `Element` itself) suffices for the `==` / `<` / `>` cursor
//! comparisons. A discrete user type with a `PartialOrd`-only order
//! (e.g. an ordered chain of values plus an error variant rejected
//! by `Element::validate`) is fully iterable; the construction
//! chokepoint guarantees stored bounds are comparable to each other.
//!
//! # Half-bounded and unbounded shapes
//!
//! Each cursor (`front`, `back`) is independently `Option<T>`. A `None`
//! cursor *is* "this direction yields nothing" — no special-case
//! branches, no panics, no errors:
//!
//! - `[a, +∞)` → `front = Some(a)`, `back = None`. Forward walks until
//!   `try_adjacent` returns `None` at the type's MAX; reverse yields
//!   nothing.
//! - `(-∞, b]` → `front = None`, `back = Some(b)`. Mirror.
//! - `(-∞, +∞)` → both `None`, both directions yield nothing.
//! - empty → both `None`.
//!
//! # Contract
//!
//! Tier 2 (infallible when closed over the invariants). Iteration cannot
//! panic given a well-formed interval. See [`crate::ops`] for the full
//! tier model.

use crate::bound::{BoundType, FiniteBound, Side};
use crate::empty::MaybeEmpty;
use crate::numeric::DiscreteElement;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

/// Convert a set value into an iterator that yields its discrete elements.
///
/// Distinct from [`IntoIterator`] so set types can keep `IntoIterator` for
/// interval-wise iteration without conflict. Implemented only on owned set
/// types — for borrowed iteration, use the inherent `.elements()` method
/// on the set type, which mirrors std's `iter()` / `into_iter()`
/// convention.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let xs: Vec<i32> = EnumInterval::closed(0, 4).into_elements().collect();
/// assert_eq!(xs, vec![0, 1, 2, 3, 4]);
///
/// let xs: Vec<i32> = EnumInterval::closed(0, 4).into_elements().rev().collect();
/// assert_eq!(xs, vec![4, 3, 2, 1, 0]);
/// ```
///
/// # Contract
///
/// Tier 2 (infallible when closed over the invariants). See
/// [`crate::ops`] for the full tier model.
pub trait IntoElementIterator {
    /// The element type produced by the iterator.
    type Item;
    /// The iterator type.
    type IntoIter: Iterator<Item = Self::Item>;
    /// Consume `self` and return an iterator over its elements.
    fn into_elements(self) -> Self::IntoIter;
}

/// Iterator over the discrete element values of an interval.
///
/// Yields `T` by value via [`Element::try_adjacent`]. Implements
/// [`Iterator`], [`DoubleEndedIterator`], and [`core::iter::FusedIterator`],
/// so combinators like `.rev()`, `.collect()`, `.take_while()`, and
/// `.fuse()` all work as expected.
///
/// Constructed via [`IntoElementIterator::into_elements`] (consume) or the
/// inherent `.elements()` methods on the interval types (borrow).
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// // Forward walk over a closed interval.
/// let xs: Vec<u8> = FiniteInterval::closed(2u8, 5).into_elements().collect();
/// assert_eq!(xs, vec![2, 3, 4, 5]);
///
/// // Open bounds step in for discrete types.
/// let xs: Vec<i32> = EnumInterval::open(0, 5).into_elements().collect();
/// assert_eq!(xs, vec![1, 2, 3, 4]);
///
/// // Reaching the type's MAX terminates iteration without panicking.
/// let xs: Vec<u8> = FiniteInterval::closed(u8::MAX - 2, u8::MAX)
///     .into_elements()
///     .collect();
/// assert_eq!(xs, vec![253, 254, 255]);
/// ```
pub struct Elements<T> {
    front: Option<T>,
    back: Option<T>,
}

impl<T> Elements<T> {
    const fn empty() -> Self {
        Self {
            front: None,
            back: None,
        }
    }
}

impl<T: DiscreteElement> Elements<T> {
    /// Build a fresh iterator from the bounds of an interval. Open bounds
    /// step in via `try_adjacent` to land on the first/last included
    /// element; an `Open` whose value has no neighbor in the stepping
    /// direction (e.g. `Open(T::MAX)` on the right) collapses that
    /// cursor to `None`.
    fn from_bounds(lower: Option<FiniteBound<T>>, upper: Option<FiniteBound<T>>) -> Self {
        let front = lower.and_then(|b| match b.into_raw() {
            (BoundType::Closed, v) => Some(v),
            (BoundType::Open, v) => v.try_adjacent(Side::Right),
        });
        let back = upper.and_then(|b| match b.into_raw() {
            (BoundType::Closed, v) => Some(v),
            (BoundType::Open, v) => v.try_adjacent(Side::Left),
        });
        // If front > back after open-bound stepping (e.g. `(5, 6)` discrete
        // → no representable interior element), the next() / next_back()
        // guards collapse to empty on first call.
        Self { front, back }
    }
}

impl<T: DiscreteElement> Iterator for Elements<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let current = self.front.take()?;
        match self.back.as_ref() {
            Some(b) if &current == b => {
                // meeting point — yield once, exhaust both ends
                self.back = None;
            }
            Some(b) if &current > b => {
                // next_back already advanced past front; iterator is done
                self.back = None;
                return None;
            }
            _ => {
                // advance; None propagates exhaustion at type MAX
                self.front = current.try_adjacent(Side::Right);
            }
        }
        Some(current)
    }

    // Default size_hint of (0, None) is intentional: a tighter bound
    // requires T::Measure, which we keep off the type bounds. Revisit
    // as a separate `impl<T: Measure> Elements<T>` adding both
    // size_hint and ExactSizeIterator.
}

impl<T: DiscreteElement> DoubleEndedIterator for Elements<T> {
    fn next_back(&mut self) -> Option<T> {
        let current = self.back.take()?;
        match self.front.as_ref() {
            Some(f) if &current == f => {
                self.front = None;
            }
            Some(f) if &current < f => {
                self.front = None;
                return None;
            }
            _ => {
                self.back = current.try_adjacent(Side::Left);
            }
        }
        Some(current)
    }
}

impl<T: DiscreteElement> core::iter::FusedIterator for Elements<T> {}

// ---- FiniteInterval ----

impl<T: DiscreteElement> IntoElementIterator for FiniteInterval<T> {
    type Item = T;
    type IntoIter = Elements<T>;

    fn into_elements(self) -> Elements<T> {
        match self.into_raw() {
            None => Elements::empty(),
            Some((lhs, rhs)) => Elements::from_bounds(Some(lhs), Some(rhs)),
        }
    }
}

impl<T: DiscreteElement + Clone> FiniteInterval<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    pub fn elements(&self) -> Elements<T> {
        match self.view_raw() {
            None => Elements::empty(),
            Some((lhs, rhs)) => Elements::from_bounds(Some(lhs.clone()), Some(rhs.clone())),
        }
    }
}

// ---- HalfInterval ----

impl<T: DiscreteElement> IntoElementIterator for HalfInterval<T> {
    type Item = T;
    type IntoIter = Elements<T>;

    fn into_elements(self) -> Elements<T> {
        let (side, bound) = self.into_raw();
        match side {
            Side::Left => Elements::from_bounds(Some(bound), None),
            Side::Right => Elements::from_bounds(None, Some(bound)),
        }
    }
}

impl<T: DiscreteElement + Clone> HalfInterval<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    pub fn elements(&self) -> Elements<T> {
        let bound = self.finite_bound().clone();
        match self.side() {
            Side::Left => Elements::from_bounds(Some(bound), None),
            Side::Right => Elements::from_bounds(None, Some(bound)),
        }
    }
}

// ---- EnumInterval ----

impl<T: DiscreteElement> IntoElementIterator for EnumInterval<T> {
    type Item = T;
    type IntoIter = Elements<T>;

    fn into_elements(self) -> Elements<T> {
        match self {
            Self::Finite(inner) => inner.into_elements(),
            Self::Half(inner) => inner.into_elements(),
            Self::Unbounded => Elements::empty(),
        }
    }
}

impl<T: DiscreteElement + Clone> EnumInterval<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    pub fn elements(&self) -> Elements<T> {
        match self {
            Self::Finite(inner) => inner.elements(),
            Self::Half(inner) => inner.elements(),
            Self::Unbounded => Elements::empty(),
        }
    }
}

// ---- MaybeDisjoint ----

/// Iterator over the discrete element values of a [`MaybeDisjoint`].
///
/// `MaybeDisjoint` is at most two pieces, so this iterator holds at most
/// two `Elements<T>` walkers — one per cursor. Implements [`Iterator`],
/// [`DoubleEndedIterator`], and [`core::iter::FusedIterator`].
///
/// Constructed via [`IntoElementIterator::into_elements`] (consume) or
/// [`MaybeDisjoint::elements`] (borrow).
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
/// use intervalsets_core::sets::MaybeDisjoint;
///
/// let lhs = EnumInterval::closed(0, 2);
/// let rhs = EnumInterval::closed(10, 12);
/// let pair = MaybeDisjoint::from_pair(lhs, rhs);
///
/// let xs: Vec<i32> = pair.into_elements().collect();
/// assert_eq!(xs, vec![0, 1, 2, 10, 11, 12]);
/// ```
pub struct DisjointElements<T> {
    front: Option<Elements<T>>,
    back: Option<Elements<T>>,
}

impl<T: DiscreteElement> Iterator for DisjointElements<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if let Some(it) = self.front.as_mut() {
            if let Some(v) = it.next() {
                return Some(v);
            }
            self.front = None;
        }
        // Front exhausted; back walker now owns whatever's left.
        // Elements<T> is itself DoubleEndedIterator, so meeting in
        // the middle is handled there.
        if let Some(it) = self.back.as_mut() {
            if let Some(v) = it.next() {
                return Some(v);
            }
            self.back = None;
        }
        None
    }

    // Delegate to the at-most-two children. Today both contribute
    // (0, None), so this matches the default — but when Elements gains
    // a tighter hint (via Measure, deferred), we pick it up for free.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (fl, fu) = self
            .front
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let (bl, bu) = self.back.as_ref().map_or((0, Some(0)), Iterator::size_hint);
        let lower = fl.saturating_add(bl);
        let upper = match (fu, bu) {
            (Some(a), Some(b)) => a.checked_add(b),
            _ => None,
        };
        (lower, upper)
    }
}

impl<T: DiscreteElement> DoubleEndedIterator for DisjointElements<T> {
    fn next_back(&mut self) -> Option<T> {
        if let Some(it) = self.back.as_mut() {
            if let Some(v) = it.next_back() {
                return Some(v);
            }
            self.back = None;
        }
        if let Some(it) = self.front.as_mut() {
            if let Some(v) = it.next_back() {
                return Some(v);
            }
            self.front = None;
        }
        None
    }
}

impl<T: DiscreteElement> core::iter::FusedIterator for DisjointElements<T> {}

impl<T: DiscreteElement> IntoElementIterator for MaybeDisjoint<T> {
    type Item = T;
    type IntoIter = DisjointElements<T>;

    fn into_elements(self) -> DisjointElements<T> {
        match self {
            // Skip wrapping when the piece is empty so `size_hint` can
            // report an exact `(0, Some(0))` for the empty MaybeDisjoint
            // instead of inheriting the inner walker's coarser hint.
            MaybeDisjoint::Connected(iv) => DisjointElements {
                front: (!iv.is_empty()).then(|| iv.into_elements()),
                back: None,
            },
            MaybeDisjoint::Disjoint(lhs, rhs) => DisjointElements {
                front: Some(lhs.into_elements()),
                back: Some(rhs.into_elements()),
            },
        }
    }
}

impl<T: DiscreteElement + Clone> MaybeDisjoint<T> {
    /// Borrow `self` and produce an iterator over its discrete elements.
    ///
    /// Walks each piece in order, yielding every element along the way.
    /// The returned iterator is double-ended.
    pub fn elements(&self) -> DisjointElements<T> {
        match self {
            MaybeDisjoint::Connected(iv) => DisjointElements {
                front: (!iv.is_empty()).then(|| iv.elements()),
                back: None,
            },
            MaybeDisjoint::Disjoint(lhs, rhs) => DisjointElements {
                front: Some(lhs.elements()),
                back: Some(rhs.elements()),
            },
        }
    }
}

/// Element iteration is gated on `DiscreteElement` — continuous
/// types have no adjacency relation (`try_adjacent` returns `None`),
/// so an "iterator over the elements" is ill-defined regardless of
/// whether the type happens to be `Ord`. Floats are excluded twice
/// over (not `Ord`, also `ContinuousKind`); `Decimal` / `BigDecimal`
/// / `OrderedFloat<T>` / `NotNan<T>` are excluded once (`Ord` but
/// `ContinuousKind`). Users wanting an element iterator on a
/// continuous T should pick a discrete grid (e.g. `Fixed*`) instead.
///
/// ```compile_fail
/// use intervalsets_core::prelude::*;
/// let _ = FiniteInterval::closed(0.0_f64, 1.0).into_elements();
/// ```
#[allow(dead_code)]
struct ContinuousCompileFailDoctest;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory};

    #[test]
    fn finite_closed_walks_forward() {
        assert!(FiniteInterval::closed(0, 4)
            .into_elements()
            .eq([0, 1, 2, 3, 4]));
    }

    #[test]
    fn finite_closed_walks_backward() {
        assert!(FiniteInterval::closed(0, 4)
            .into_elements()
            .rev()
            .eq([4, 3, 2, 1, 0]));
    }

    #[test]
    fn finite_open_steps_in_for_discrete() {
        assert!(EnumInterval::open(0, 5).into_elements().eq([1, 2, 3, 4]));
    }

    #[test]
    fn singleton_yields_once_forward() {
        assert!(FiniteInterval::closed(7, 7).into_elements().eq([7]));
    }

    #[test]
    fn singleton_yields_once_backward() {
        assert!(FiniteInterval::closed(7, 7).into_elements().rev().eq([7]));
    }

    #[test]
    fn empty_yields_nothing() {
        let mut it = FiniteInterval::<i32>::empty().into_elements();
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn unbounded_yields_nothing_either_direction() {
        let mut fwd = EnumInterval::<i32>::Unbounded.into_elements();
        assert_eq!(fwd.next(), None);

        let mut back = EnumInterval::<i32>::Unbounded.into_elements();
        assert_eq!(back.next_back(), None);
    }

    #[test]
    fn left_half_bounded_walks_forward_and_terminates_at_max() {
        // u8 max is 255; start near it so the walk terminates.
        let iset = HalfInterval::left(FiniteBound::closed(253u8));
        assert!(iset.into_elements().eq([253u8, 254, 255]));
    }

    #[test]
    fn left_half_bounded_yields_nothing_in_reverse() {
        let iset = HalfInterval::left(FiniteBound::closed(0u8));
        assert_eq!(iset.into_elements().next_back(), None);
    }

    #[test]
    fn right_half_bounded_walks_backward_and_terminates_at_min() {
        let iset = HalfInterval::right(FiniteBound::closed(2u8));
        assert!(iset.into_elements().rev().eq([2u8, 1, 0]));
    }

    #[test]
    fn right_half_bounded_yields_nothing_forward() {
        let iset = HalfInterval::right(FiniteBound::closed(10u8));
        assert_eq!(iset.into_elements().next(), None);
    }

    #[test]
    fn meeting_in_the_middle() {
        let mut it = FiniteInterval::closed(0, 5).into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next_back(), Some(5));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next_back(), Some(4));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next_back(), Some(3));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn meeting_at_a_single_remaining_element() {
        // Odd length — the meeting yields the lone middle element.
        let mut it = FiniteInterval::closed(0, 4).into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next_back(), Some(4));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next_back(), Some(3));
        assert_eq!(it.next(), Some(2));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn type_max_terminates_forward_walk() {
        assert!(FiniteInterval::closed(u8::MAX - 2, u8::MAX)
            .into_elements()
            .eq([253u8, 254, 255]));
    }

    #[test]
    fn type_min_terminates_backward_walk() {
        assert!(FiniteInterval::closed(0u8, 2)
            .into_elements()
            .rev()
            .eq([2u8, 1, 0]));
    }

    #[test]
    fn elements_borrowed_does_not_consume() {
        let interval = FiniteInterval::closed(0, 3);
        assert!(interval.elements().eq([0, 1, 2, 3]));
        // Still usable.
        assert_eq!(interval.elements().count(), 4);
    }

    #[test]
    fn elements_borrowed_on_enum() {
        let iv = EnumInterval::closed(10, 13);
        assert!(iv.elements().eq([10, 11, 12, 13]));
    }

    #[test]
    fn elements_borrowed_on_half() {
        let iv = HalfInterval::left(FiniteBound::closed(253u8));
        assert!(iv.elements().eq([253u8, 254, 255]));
    }

    #[test]
    fn fused_after_exhaustion() {
        let mut it = FiniteInterval::closed(0, 1).into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next(), None);
        // Fused — repeated calls keep returning None.
        assert_eq!(it.next(), None);
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    // ---- MaybeDisjoint coverage ----

    #[test]
    fn maybe_disjoint_empty_yields_nothing() {
        let mut it = MaybeDisjoint::<i32>::empty().into_elements();
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn maybe_disjoint_connected_walks_forward() {
        let pair = MaybeDisjoint::Connected(EnumInterval::closed(2, 5));
        assert!(pair.into_elements().eq([2, 3, 4, 5]));
    }

    #[test]
    fn maybe_disjoint_connected_walks_backward() {
        let pair = MaybeDisjoint::Connected(EnumInterval::closed(2, 5));
        assert!(pair.into_elements().rev().eq([5, 4, 3, 2]));
    }

    #[test]
    fn maybe_disjoint_two_pieces_walks_forward() {
        let pair =
            MaybeDisjoint::from_pair(EnumInterval::closed(0, 2), EnumInterval::closed(10, 12));
        assert!(pair.into_elements().eq([0, 1, 2, 10, 11, 12]));
    }

    #[test]
    fn maybe_disjoint_two_pieces_walks_backward() {
        let pair =
            MaybeDisjoint::from_pair(EnumInterval::closed(0, 2), EnumInterval::closed(10, 12));
        assert!(pair.into_elements().rev().eq([12, 11, 10, 2, 1, 0]));
    }

    #[test]
    fn maybe_disjoint_mixed_walk_meets_inside_one_piece() {
        let pair =
            MaybeDisjoint::from_pair(EnumInterval::closed(0, 1), EnumInterval::closed(10, 14));
        let mut it = pair.into_elements();
        assert_eq!(it.next(), Some(0));
        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next_back(), Some(14));
        assert_eq!(it.next(), Some(10));
        assert_eq!(it.next_back(), Some(13));
        assert_eq!(it.next(), Some(11));
        assert_eq!(it.next_back(), Some(12));
        assert_eq!(it.next(), None);
        assert_eq!(it.next_back(), None);
    }

    #[test]
    fn maybe_disjoint_borrowed_does_not_consume() {
        let pair =
            MaybeDisjoint::from_pair(EnumInterval::closed(0u8, 1), EnumInterval::closed(10, 11));
        assert!(pair.elements().eq([0u8, 1, 10, 11]));
        // Still usable.
        assert_eq!(pair.elements().count(), 4);
    }

    #[test]
    fn maybe_disjoint_with_half_bounded_piece() {
        // Right piece is `[254, +∞)` over u8 → walks to MAX = 255.
        let pair = MaybeDisjoint::from_pair(
            EnumInterval::closed(0u8, 1),
            EnumInterval::closed_unbound(254u8),
        );
        assert!(pair.into_elements().eq([0u8, 1, 254, 255]));
    }

    #[test]
    fn disjoint_elements_size_hint_empty_is_exact_zero() {
        // With both cursors None, the delegated hint reports the
        // tightest possible (0, Some(0)) rather than (0, None).
        let it = MaybeDisjoint::<i32>::empty().into_elements();
        assert_eq!(it.size_hint(), (0, Some(0)));
    }

    /// Iteration on a discrete type that is `PartialOrd` (not `Ord`):
    /// a chain of values with an error variant that the default
    /// `Element::validate` rejects (incomparable to itself). Stored
    /// bounds are pairwise comparable by construction, so iteration
    /// is well-defined.
    #[test]
    fn discrete_partial_ord_only_chain_iterates() {
        use crate::bound::Side;
        use crate::numeric::{DiscreteKind, Element};

        #[derive(Debug, Clone, PartialEq)]
        #[allow(dead_code)] // `Err` is referenced through `PartialOrd::partial_cmp` matches only.
        enum ChainItem {
            Val(u8),
            Err,
        }

        impl PartialOrd for ChainItem {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                match (self, other) {
                    (Self::Val(a), Self::Val(b)) => a.partial_cmp(b),
                    // Err is incomparable to anything — including itself,
                    // so the default `Element::validate` rejects it at
                    // construction.
                    _ => None,
                }
            }
        }

        impl Element for ChainItem {
            type Kind = DiscreteKind;
            type Measure = u16;

            fn try_adjacent(&self, side: Side) -> Option<Self> {
                match self {
                    Self::Val(v) => match side {
                        Side::Left => v.checked_sub(1).map(Self::Val),
                        Side::Right => v.checked_add(1).map(Self::Val),
                    },
                    Self::Err => None,
                }
            }

            fn try_measure_finite(left: &Self, right: &Self) -> Option<Self::Measure> {
                match (left, right) {
                    (Self::Val(l), Self::Val(r)) => u8::try_measure_finite(l, r),
                    _ => None,
                }
            }
        }

        // ChainItem is NOT `Ord`, but `[Val(2), Val(5)]` has pairwise
        // comparable bounds — accepted at construction, iterable.
        let iv = FiniteInterval::closed(ChainItem::Val(2), ChainItem::Val(5));
        assert!(iv.into_elements().eq([
            ChainItem::Val(2),
            ChainItem::Val(3),
            ChainItem::Val(4),
            ChainItem::Val(5),
        ]));
    }
}

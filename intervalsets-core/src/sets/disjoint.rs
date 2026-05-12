use super::{EnumInterval, FiniteInterval, HalfInterval};
use crate::empty::MaybeEmpty;
use crate::numeric::Element;
use crate::ops::{Connects, MergeConnected};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaybeDisjoint<T> {
    #[non_exhaustive]
    Connected(EnumInterval<T>),
    #[non_exhaustive]
    Disjoint(EnumInterval<T>, EnumInterval<T>),
}

impl<T> Iterator for MaybeDisjoint<T> {
    type Item = EnumInterval<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // `Connected(EnumInterval::empty())` doubles as the drained
        // sentinel: it denotes ∅ as a set and yields `None` here, so the
        // iterator's exhausted state coincides with the canonical empty
        // value. `EnumInterval::empty()` is a tag-only variant — no `T`
        // is constructed — so the swap is just a discriminant write.
        let mut inst = Self::Connected(EnumInterval::empty());
        core::mem::swap(self, &mut inst);
        match inst {
            Self::Connected(interval) => interval.is_inhabited().then_some(interval),
            Self::Disjoint(lhs, rhs) => {
                *self = Self::Connected(rhs);
                Some(lhs)
            }
        }
    }
}

impl<T: Element> MaybeDisjoint<T> {
    /// Create a new MaybeDisjoint from two optional EnumIntervals.
    ///
    /// Invariants are applied.
    pub fn new(a: Option<EnumInterval<T>>, b: Option<EnumInterval<T>>) -> Self {
        match (a, b) {
            (None, None) => Self::empty(),
            (Some(interval), None) | (None, Some(interval)) => Self::from_interval(interval),
            (Some(a), Some(b)) => Self::from_pair(a, b),
        }
    }

    /// Create a new `MaybeDisjoint` from two EnumIntervals and repairs invariants.
    pub fn from_pair(a: EnumInterval<T>, b: EnumInterval<T>) -> Self {
        if a.connects(&b) {
            match a.merge_connected(b) {
                Some(merged) => Self::from_interval(merged),
                None => unreachable!("connects() implies merge_connected returns Some"),
            }
        } else {
            // the empty set connects trivially with all other sets,
            // so both a, and b must be inhabited, disjoint intervals.
            if a < b {
                MaybeDisjoint::Disjoint(a, b)
            } else {
                MaybeDisjoint::Disjoint(b, a)
            }
        }
    }

    pub(crate) fn new_disjoint_assume_valid(left: EnumInterval<T>, right: EnumInterval<T>) -> Self {
        debug_assert!(Self::satisfies_invariants(&left, &right));
        Self::Disjoint(left, right)
    }

    /// Returns `true` if `(left, right)` is a well-formed `Disjoint`
    /// pair: both non-empty, sorted, and non-connecting.
    pub(crate) fn satisfies_invariants(left: &EnumInterval<T>, right: &EnumInterval<T>) -> bool {
        !left.is_empty() && !right.is_empty() && left < right && !left.connects(right)
    }
}

impl<T> MaybeDisjoint<T> {
    pub fn empty() -> Self {
        Self::Connected(EnumInterval::empty())
    }

    pub fn from_interval(interval: EnumInterval<T>) -> Self {
        Self::Connected(interval)
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; returns `None` if this is two disjoint intervals.
    pub fn into_interval(self) -> Option<EnumInterval<T>> {
        match self {
            Self::Connected(interval) => Some(interval),
            Self::Disjoint(_, _) => None,
        }
    }

    /// Returns the interval if this is empty or a single connected
    /// interval; panics otherwise.
    ///
    /// # Panics
    ///
    /// Panics if this is two disjoint intervals. Use
    /// [`into_interval`](Self::into_interval) for a panic-free alternative.
    pub fn expect_interval(self) -> EnumInterval<T> {
        self.into_interval()
            .expect("expected a single connected interval")
    }
}

impl<T> From<FiniteInterval<T>> for MaybeDisjoint<T> {
    fn from(value: FiniteInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<HalfInterval<T>> for MaybeDisjoint<T> {
    fn from(value: HalfInterval<T>) -> Self {
        Self::from(EnumInterval::from(value))
    }
}

impl<T> From<EnumInterval<T>> for MaybeDisjoint<T> {
    fn from(interval: EnumInterval<T>) -> Self {
        Self::from_interval(interval)
    }
}

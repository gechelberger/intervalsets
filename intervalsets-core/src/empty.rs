use crate::bound::ord::OrdBoundPair;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Test a `Set` for emptiness.
pub trait MaybeEmpty {
    /// Is self the empty set
    fn is_empty(&self) -> bool;

    /// Does this set contain at least one element
    fn is_inhabited(&self) -> bool {
        !self.is_empty()
    }
}

impl<T> MaybeEmpty for FiniteInterval<T> {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl<T> MaybeEmpty for HalfInterval<T> {
    fn is_empty(&self) -> bool {
        false
    }
}

impl<T> MaybeEmpty for EnumInterval<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Finite(inner) => inner.is_empty(),
            Self::Half(_) | Self::Unbounded => false,
        }
    }
}

impl<T: PartialEq> MaybeEmpty for OrdBoundPair<T> {
    fn is_empty(&self) -> bool {
        self.is_empty() // forwards to the concrete impl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bound::FiniteBound;

    #[test]
    fn test_empty() {
        let empty = FiniteInterval::<u32>::empty();
        assert!(empty.is_empty());

        let not_empty = FiniteInterval::<u32>::new(FiniteBound::closed(0), FiniteBound::closed(10));
        assert!(!not_empty.is_empty());

        let empty = EnumInterval::Finite(empty);
        assert!(empty.is_empty());

        let not_empty = EnumInterval::Finite(not_empty);
        assert!(!not_empty.is_empty());
    }
}

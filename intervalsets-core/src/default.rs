// there is no reasonable default value for HalfInterval
use crate::sets::{EnumInterval, FiniteInterval, StackSet};

impl<T> Default for FiniteInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Default for EnumInterval<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> Default for StackSet<T> {
    fn default() -> Self {
        Self::empty()
    }
}

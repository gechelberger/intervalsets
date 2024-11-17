pub use intervalsets_core::ops::Adjacent;

use crate::numeric::Domain;
use crate::sets::Interval;

impl<T: Domain> Adjacent<&Interval<T>> for Interval<T> {
    fn is_adjacent_to(&self, rhs: &Interval<T>) -> bool {
        self.0.is_adjacent_to(&rhs.0)
    }
}

pub use intervalsets_core::ops::Adjacent;

use crate::numeric::Element;
use crate::sets::Interval;

impl<T: Element> Adjacent<&Interval<T>> for Interval<T> {
    fn is_adjacent_to(&self, rhs: &Interval<T>) -> bool {
        self.0.is_adjacent_to(&rhs.0)
    }
}

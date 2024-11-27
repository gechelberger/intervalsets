pub use intervalsets_core::ops::Connects;

use crate::numeric::Element;
use crate::sets::Interval;

impl<T: Element> Connects<&Interval<T>> for Interval<T> {
    fn connects(&self, rhs: &Interval<T>) -> bool {
        self.0.connects(&rhs.0)
    }
}

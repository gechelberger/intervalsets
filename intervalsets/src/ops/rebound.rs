pub use intervalsets_core::ops::Rebound;

use crate::bound::FiniteBound;
use crate::numeric::{Element, Zero};
use crate::sets::Interval;

impl<T: Element + Zero> Rebound<T> for Interval<T> {
    type Output = Self;
    type Error = crate::error::Error;

    fn with_left_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        self.0.with_left_strict(bound).map(Interval::from)
    }

    fn with_right_strict(self, bound: Option<FiniteBound<T>>) -> Result<Self::Output, Self::Error> {
        self.0.with_right_strict(bound).map(Interval::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::traits::*;

    #[test]
    fn test_interval_rebound() {
        let x = Interval::closed(0, 10);
        assert_eq!(x.with_left_closed(5), [5, 10].into());
        assert_eq!(x.with_right_closed(5), [0, 5].into());

        assert_eq!(x.with_left(None), Interval::unbound_closed(10));
        assert_eq!(x.with_right(None), Interval::closed_unbound(0));

        assert_eq!(x.with_left(None).with_right(None), Interval::unbounded());

        assert_eq!(x.with_left_closed(20), Interval::empty());
        assert_eq!(x.with_right_closed(-20), Interval::empty());
    }
}

pub use intervalsets_core::ops::Rebound;

use crate::bound::FiniteBound;
use crate::numeric::{Domain, Zero};
use crate::sets::Interval;

impl<T: Domain + Zero> Rebound<T> for Interval<T> {
    type Output = Self;

    fn with_left(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        self.0.with_left(bound).into()
    }

    fn with_right(self, bound: Option<FiniteBound<T>>) -> Self::Output {
        self.0.with_right(bound).into()
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

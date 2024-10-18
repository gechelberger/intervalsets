use crate::continuous_domain_impl;
use chrono::{DateTime, TimeDelta, TimeZone};

impl<T: TimeZone> crate::numeric::Domain for DateTime<T> {
    #[inline]
    fn try_adjacent(&self, side: crate::Side) -> Option<Self> {
        None
    }
}

continuous_domain_impl!(TimeDelta);

impl crate::numeric::LibZero for TimeDelta {
    fn new_zero() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::measure::Width;
    use crate::Interval;

    #[test]
    fn test_chrono_datetime() {
        let a = Utc::now();
        let delta = TimeDelta::new(1000, 0).unwrap();
        let b = a + delta;

        let interval = Interval::open(a, b);
        assert_eq!(interval.width().finite(), delta);
    }

    #[test]
    fn test_chrono_timedelta() {
        let a = TimeDelta::new(100, 0).unwrap();
        let b = a + a;

        let interval = Interval::open(a, b);
        assert_eq!(interval.width().finite(), a);
    }
}

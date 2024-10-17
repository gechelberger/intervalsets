use chrono::{DateTime, TimeDelta, TimeZone};
use crate::continuous_domain_impl;

impl<T: TimeZone> crate::Domain for DateTime<T> {
    #[inline]
    fn try_adjacent(&self, side: crate::Side) -> Option<Self> {
        None
    }
}

continuous_domain_impl!(TimeDelta);

impl crate::LibZero for TimeDelta {
    fn new_zero() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::Interval;
    use crate::measure::Width;

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

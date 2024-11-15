use core::fmt;

use itertools::Itertools;

use crate::{Interval, IntervalSet, MaybeEmpty};

impl<T: fmt::Display> fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display> fmt::Display for IntervalSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            Interval::<i32>::empty().fmt(f)
        } else {
            write!(f, "{{{}}}", self.iter().join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::Union;
    use crate::Factory;

    #[test]
    fn test_display_finite() {
        assert_eq!(format!("{}", Interval::<i8>::empty()), "{}");

        assert_eq!(format!("{}", Interval::closed(0, 5)), "[0, 5]");

        assert_eq!(format!("{}", Interval::open(0.1, 5.1)), "(0.1, 5.1)");

        assert_eq!(format!("{}", Interval::open_closed(0.1, 5.1)), "(0.1, 5.1]");

        assert_eq!(format!("{}", Interval::closed_open(0.1, 5.1)), "[0.1, 5.1)");
    }

    #[test]
    fn test_display_half() {
        assert_eq!(format!("{}", Interval::unbound_closed(0.5)), "(<-, 0.5]");

        assert_eq!(format!("{}", Interval::unbound_open(0.5)), "(<-, 0.5)");

        assert_eq!(format!("{}", Interval::closed_unbound(0.5)), "[0.5, ->)");

        assert_eq!(format!("{}", Interval::open_unbound(0.5)), "(0.5, ->)")
    }

    #[test]
    fn test_display_interval() {
        assert_eq!(format!("{}", Interval::<i8>::empty()), "{}");

        assert_eq!(format!("{}", Interval::<i8>::unbounded()), "(<-, ->)");
    }

    #[test]
    fn test_display_set() {
        assert_eq!(
            format!(
                "{}",
                Interval::unbound_closed(-9.9)
                    .union(Interval::open(5.5, 9.9))
                    .union(Interval::closed_open(11.1, 22.2))
                    .union(Interval::open_unbound(33.3))
            ),
            "{(<-, -9.9], (5.5, 9.9), [11.1, 22.2), (33.3, ->)}"
        )
    }
}

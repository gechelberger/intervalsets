use itertools::Itertools;

use crate::bound::BoundType;
use crate::numeric::Domain;
use crate::{Bound, Bounding, Interval, Side};

use crate::detail::{BoundCase, Finite, HalfBounded};
use crate::IntervalSet;

use core::fmt;

fn bound_symbol(side: Side, bound_type: BoundType) -> char {
    match bound_type {
        BoundType::Open => match side {
            Side::Left => '(',
            Side::Right => ')',
        },
        BoundType::Closed => match side {
            Side::Left => '[',
            Side::Right => ']',
        },
    }
}

fn format_bound<T: fmt::Display>(side: Side, bound: Option<&Bound<T>>) -> String {
    match bound {
        None => match side {
            Side::Left => "(<-".to_string(),
            Side::Right => "->)".to_string(),
        },
        Some(bound) => match side {
            Side::Left => format!(
                "{}{}",
                bound_symbol(side, bound.bound_type()),
                bound.value()
            ),
            Side::Right => format!(
                "{}{}",
                bound.value(),
                bound_symbol(side, bound.bound_type())
            ),
        },
    }
}

impl<T: fmt::Display + Clone> fmt::Display for Finite<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "{{}}"),
            Self::FullyBounded(left, right) => {
                write!(
                    f,
                    "{}, {}",
                    format_bound(Side::Left, Some(left)),
                    format_bound(Side::Right, Some(right)),
                )
            }
        }
    }
}

impl<T: fmt::Display + Domain> fmt::Display for HalfBounded<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}",
            format_bound(Side::Left, self.left()),
            format_bound(Side::Right, self.right())
        )
    }
}

impl<T: fmt::Display + Domain> fmt::Display for BoundCase<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unbounded => write!(f, "(<-, ->)"),
            Self::Finite(inner) => inner.fmt(f),
            Self::Half(inner) => inner.fmt(f),
        }
    }
}

impl<T: fmt::Display + Domain> fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Display + Domain> fmt::Display for IntervalSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.intervals().is_empty() {
            Finite::<i32>::Empty.fmt(f)
        } else {
            write!(f, "{{{}}}", self.intervals().iter().join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ops::Union;

    use super::*;

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
                    .union(&Interval::open(5.5, 9.9))
                    .union(&Interval::closed_open(11.1, 22.2))
                    .union(&Interval::open_unbound(33.3))
            ),
            "{(<-, -9.9], (5.5, 9.9), [11.1, 22.2), (33.3, ->)}"
        )
    }
}

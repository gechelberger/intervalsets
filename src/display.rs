use itertools::Itertools;

use crate::bounds::Bounds;
use crate::ival::{Bound, IVal, Side};
use crate::{Domain, EBounds, FiniteInterval, HalfBounded, Interval, IntervalSet};

fn bound_symbol(side: Side, bound: Bound) -> char {
    match bound {
        Bound::Open => match side {
            Side::Left => '(',
            Side::Right => ')',
        },
        Bound::Closed => match side {
            Side::Left => '[',
            Side::Right => ']',
        },
    }
}

fn format_ival<T: std::fmt::Display>(side: Side, ival: Option<IVal<T>>) -> String {
    match ival {
        None => match side {
            Side::Left => "(<-".to_string(),
            Side::Right => "->)".to_string(),
        },
        Some(ival) => match side {
            Side::Left => format!("{}{}", bound_symbol(side, ival.bound), ival.value),
            Side::Right => format!("{}{}", ival.value, bound_symbol(side, ival.bound)),
        },
    }
}

impl<T: std::fmt::Display + Clone> std::fmt::Display for FiniteInterval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "{{}}"),
            Self::FullyBounded(left, right) => {
                write!(
                    f,
                    "{}, {}",
                    format_ival(Side::Left, Some(left.clone())),
                    format_ival(Side::Right, Some(right.clone())),
                )
            }
        }
    }
}

impl<T: std::fmt::Display + Clone> std::fmt::Display for HalfBounded<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}",
            format_ival(Side::Left, self.left()),
            format_ival(Side::Right, self.right())
        )
    }
}

impl<T: std::fmt::Display + Clone> std::fmt::Display for EBounds<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unbounded => write!(f, "(<-, ->)"),
            Self::Finite(inner) => inner.fmt(f),
            Self::Half(inner) => inner.fmt(f),
        }
    }
}

impl<T: std::fmt::Display + Domain> std::fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: std::fmt::Display + Domain> std::fmt::Display for IntervalSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.intervals.is_empty() {
            FiniteInterval::<i32>::Empty.fmt(f)
        } else {
            write!(f, "{{{}}}", self.intervals.iter().join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::op::union::Union;

    use super::*;

    #[test]
    fn test_display_finite() {
        assert_eq!(format!("{}", FiniteInterval::<i8>::Empty), "{}");

        assert_eq!(format!("{}", FiniteInterval::closed(0, 5)), "[0, 5]");

        assert_eq!(format!("{}", FiniteInterval::open(0.1, 5.1)), "(0.1, 5.1)");

        assert_eq!(
            format!("{}", FiniteInterval::open_closed(0.1, 5.1)),
            "(0.1, 5.1]"
        );

        assert_eq!(
            format!("{}", FiniteInterval::closed_open(0.1, 5.1)),
            "[0.1, 5.1)"
        );
    }

    #[test]
    fn test_display_half() {
        assert_eq!(
            format!("{}", HalfBounded::unbound_closed(0.5)),
            "(<-, 0.5]"
        );

        assert_eq!(format!("{}", HalfBounded::unbound_open(0.5)), "(<-, 0.5)");

        assert_eq!(
            format!("{}", HalfBounded::closed_unbound(0.5)),
            "[0.5, ->)"
        );

        assert_eq!(format!("{}", HalfBounded::open_unbound(0.5)), "(0.5, ->)")
    }

    #[test]
    fn test_display_interval() {
        assert_eq!(format!("{}", EBounds::<i8>::empty()), "{}");

        assert_eq!(format!("{}", EBounds::<i8>::unbound()), "(<-, ->)");
    }

    #[test]
    fn test_display_set() {
        assert_eq!(
            format!(
                "{}",
                EBounds::unbound_closed(-9.9)
                    .union(&EBounds::open(5.5, 9.9))
                    .union(&EBounds::closed_open(11.1, 22.2))
                    .union(&EBounds::open_unbound(33.3))
            ),
            "{(<-, -9.9], (5.5, 9.9), [11.1, 22.2), (33.3, ->)}"
        )
    }
}

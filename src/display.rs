use crate::bounds::Bounds;
use crate::ival::{Bound, IVal, Side};
use crate::{FiniteInterval, HalfInterval, Interval};

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
            Self::NonZero(left, right) => {
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

impl<T: std::fmt::Display + Clone> std::fmt::Display for HalfInterval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}",
            format_ival(Side::Left, self.left()),
            format_ival(Side::Right, self.right())
        )
    }
}

impl<T: std::fmt::Display + Clone> std::fmt::Display for Interval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infinite => write!(f, "(<-, ->)"),
            Self::Finite(inner) => inner.fmt(f),
            Self::Half(inner) => inner.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_finite() {
        assert_eq!(format!("{}", FiniteInterval::<i8>::Empty), "{}");

        assert_eq!(format!("{}", FiniteInterval::closed(0, 5)), "[0, 5]");

        assert_eq!(format!("{}", FiniteInterval::open(0.1, 5.1)), "(0.1, 5.1)");

        assert_eq!(
            format!("{}", FiniteInterval::openclosed(0.1, 5.1)),
            "(0.1, 5.1]"
        );

        assert_eq!(
            format!("{}", FiniteInterval::closedopen(0.1, 5.1)),
            "[0.1, 5.1)"
        );
    }

    #[test]
    fn test_display_half() {
        assert_eq!(
            format!("{}", HalfInterval::unbound_closed(0.5)),
            "(<-, 0.5]"
        );

        assert_eq!(format!("{}", HalfInterval::unbound_open(0.5)), "(<-, 0.5)");

        assert_eq!(
            format!("{}", HalfInterval::closed_unbound(0.5)),
            "[0.5, ->)"
        );

        assert_eq!(format!("{}", HalfInterval::open_unbound(0.5)), "(0.5, ->)")
    }

    #[test]
    fn test_format_interval() {
        assert_eq!(format!("{}", Interval::<i8>::empty()), "{}");

        assert_eq!(format!("{}", Interval::<i8>::unbound()), "(<-, ->)");
    }
}
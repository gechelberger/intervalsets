use crate::ival::{Bound, IVal, Side};
use crate::FiniteInterval;

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

fn format_ival<T: std::fmt::Display>(side: Side, ival: Option<&IVal<T>>) -> String {
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

impl<T: std::fmt::Display> std::fmt::Display for FiniteInterval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "{{}}"),
            Self::NonZero(left, right) => {
                write!(
                    f,
                    "{}, {}",
                    format_ival(Side::Left, Some(left)),
                    format_ival(Side::Right, Some(right)),
                )
            }
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
}

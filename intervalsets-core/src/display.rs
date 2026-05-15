use core::fmt::{self, Write};

use crate::bound::{BoundType, FiniteBound, Side};
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval, MaybeDisjoint};

fn write_bound_type(
    f: &mut fmt::Formatter<'_>,
    side: Side,
    bound_type: Option<BoundType>,
) -> fmt::Result {
    let delim = match bound_type.unwrap_or(BoundType::Open) {
        BoundType::Closed => side.select('[', ']'),
        BoundType::Open => side.select('(', ')'),
    };

    f.write_char(delim)
}

fn write_bound<T>(
    f: &mut fmt::Formatter<'_>,
    side: Side,
    bound: Option<&FiniteBound<T>>,
) -> fmt::Result
where
    T: fmt::Display,
{
    match side {
        Side::Left => {
            write_bound_type(f, side, bound.map(|x| x.bound_type()))?;
            match bound {
                None => f.write_str("..")?,
                Some(inner) => f.write_fmt(format_args!("{}", inner.value()))?,
            }
        }
        Side::Right => {
            match bound {
                None => f.write_str("..")?,
                Some(inner) => f.write_fmt(format_args!("{}", inner.value()))?,
            }
            write_bound_type(f, side, bound.map(|x| x.bound_type()))?;
        }
    }

    Ok(())
}

impl<T: fmt::Display> fmt::Display for FiniteInterval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.view_raw() {
            None => f.write_str("{}")?,
            Some((lhs, rhs)) => {
                write_bound(f, Side::Left, Some(lhs))?;
                f.write_str(", ")?;
                write_bound(f, Side::Right, Some(rhs))?;
            }
        }

        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for HalfInterval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.side() {
            Side::Left => {
                write_bound(f, Side::Left, Some(self.finite_bound()))?;
                f.write_str(", ")?;
                write_bound::<T>(f, Side::Right, None)?;
            }
            Side::Right => {
                write_bound::<T>(f, Side::Left, None)?;
                f.write_str(", ")?;
                write_bound(f, Side::Right, Some(self.finite_bound()))?;
            }
        }

        Ok(())
    }
}

impl<T: fmt::Display> fmt::Display for EnumInterval<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unbounded => f.write_str("(.., ..)"),
            Self::Finite(inner) => inner.fmt(f),
            Self::Half(inner) => inner.fmt(f),
        }
    }
}

impl<T: fmt::Display> fmt::Display for MaybeDisjoint<T> {
    /// `Connected(iv)` delegates to the inner `EnumInterval`'s
    /// `Display`. `Disjoint(a, b)` wraps both pieces in set-notation
    /// braces, e.g. `{[0, 5], [10, 15]}`. Empty MD prints as `{}`
    /// (inherited from the empty `FiniteInterval` display).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connected(iv) => iv.fmt(f),
            Self::Disjoint(a, b) => {
                f.write_char('{')?;
                a.fmt(f)?;
                f.write_str(", ")?;
                b.fmt(f)?;
                f.write_char('}')
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

    #[test]
    fn test_display_finite() {
        assert_eq!(std::format!("{}", EnumInterval::<i8>::empty()), "{}");

        assert_eq!(std::format!("{}", EnumInterval::closed(0, 5)), "[0, 5]");

        assert_eq!(
            std::format!("{}", EnumInterval::open(0.1, 5.1)),
            "(0.1, 5.1)"
        );

        assert_eq!(
            std::format!("{}", EnumInterval::open_closed(0.1, 5.1)),
            "(0.1, 5.1]"
        );

        assert_eq!(
            std::format!("{}", EnumInterval::closed_open(0.1, 5.1)),
            "[0.1, 5.1)"
        );
    }

    #[test]
    fn test_display_half() {
        assert_eq!(
            std::format!("{}", EnumInterval::unbound_closed(0.5)),
            "(.., 0.5]"
        );

        assert_eq!(
            std::format!("{}", EnumInterval::unbound_open(0.5)),
            "(.., 0.5)"
        );

        assert_eq!(
            std::format!("{}", EnumInterval::closed_unbound(0.5)),
            "[0.5, ..)"
        );

        assert_eq!(
            std::format!("{}", EnumInterval::open_unbound(0.5)),
            "(0.5, ..)"
        )
    }

    #[test]
    fn test_display_interval() {
        assert_eq!(std::format!("{}", EnumInterval::<i8>::empty()), "{}");

        assert_eq!(
            std::format!("{}", EnumInterval::<i8>::unbounded()),
            "(.., ..)"
        );
    }

    #[test]
    fn test_display_maybe_disjoint_empty() {
        // Empty MD prints as `{}` via the underlying empty
        // FiniteInterval display.
        assert_eq!(std::format!("{}", MaybeDisjoint::<i8>::empty()), "{}");
    }

    #[test]
    fn test_display_maybe_disjoint_connected_delegates() {
        let md = MaybeDisjoint::from_interval(EnumInterval::closed(0, 5));
        assert_eq!(std::format!("{}", md), "[0, 5]");
    }

    #[test]
    fn test_display_maybe_disjoint_disjoint_uses_set_notation() {
        let md = MaybeDisjoint::from_pair(EnumInterval::closed(0, 5), EnumInterval::closed(10, 15));
        assert_eq!(std::format!("{}", md), "{[0, 5], [10, 15]}");
    }

    #[test]
    fn test_display_maybe_disjoint_with_unbounded_piece() {
        let md = MaybeDisjoint::from_pair(
            EnumInterval::unbound_closed(0_i32),
            EnumInterval::closed_unbound(10),
        );
        assert_eq!(std::format!("{}", md), "{(.., 0], [10, ..)}");
    }
}

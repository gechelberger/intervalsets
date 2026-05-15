//! `FromStr` impls for the interval types — the inverse of
//! [`Display`](crate::sets::EnumInterval#impl-Display-for-EnumInterval<T>).
//!
//! Grammar matches what `Display` emits, so `s.parse::<EnumInterval<T>>()`
//! round-trips against `format!("{x}")`. See the table on
//! [`EnumInterval`](crate::sets::EnumInterval)'s `FromStr` impl below.

use core::str::FromStr;

use crate::error::ParseIntervalError;
use crate::factory::{TryFiniteFactory, TryHalfBoundedFactory, UnboundedFactory};
use crate::numeric::Element;
use crate::sets::{EnumInterval, FiniteInterval, HalfInterval};

/// Parses an [`EnumInterval`] from its `Display` form.
///
/// # Grammar
///
/// | Form                | Example       |
/// |---------------------|---------------|
/// | closed-closed       | `[0, 10]`     |
/// | open-open           | `(0, 10)`     |
/// | closed-open         | `[0, 10)`     |
/// | open-closed         | `(0, 10]`     |
/// | closed-unbound      | `[0, ..)`     |
/// | open-unbound        | `(0, ..)`     |
/// | unbound-closed      | `(.., 10]`    |
/// | unbound-open        | `(.., 10)`    |
/// | unbounded           | `(.., ..)`    |
/// | empty               | `{}`          |
///
/// Whitespace around delimiters and the comma is ignored.
///
/// Note: the unbounded side **must** use an open delimiter (`(` on
/// the left, `)` on the right) — `[.., x]` is a syntax error, since
/// infinity is never "included." This matches `Display`'s output.
///
/// # Examples
///
/// ```
/// use intervalsets_core::prelude::*;
///
/// let x: EnumInterval<i32> = "[0, 10]".parse().unwrap();
/// assert_eq!(x, EnumInterval::closed(0, 10));
///
/// let x: EnumInterval<f64> = "(0.0, 10.0)".parse().unwrap();
/// assert_eq!(x, EnumInterval::open(0.0, 10.0));
///
/// let x: EnumInterval<f64> = "[0.0, 10.0)".parse().unwrap();
/// assert_eq!(x, EnumInterval::closed_open(0.0, 10.0));
///
/// let x: EnumInterval<f64> = "(0.0, 10.0]".parse().unwrap();
/// assert_eq!(x, EnumInterval::open_closed(0.0, 10.0));
///
/// let x: EnumInterval<i32> = "[0, ..)".parse().unwrap();
/// assert_eq!(x, EnumInterval::closed_unbound(0));
///
/// let x: EnumInterval<i32> = "(.., 10]".parse().unwrap();
/// assert_eq!(x, EnumInterval::unbound_closed(10));
///
/// let x: EnumInterval<i32> = "(.., ..)".parse().unwrap();
/// assert_eq!(x, EnumInterval::unbounded());
///
/// let x: EnumInterval<i32> = "{}".parse().unwrap();
/// assert_eq!(x, EnumInterval::empty());
///
/// // Round-trip with Display.
/// let x = EnumInterval::open_closed(1.5, 7.5);
/// assert_eq!(format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
/// ```
impl<T> FromStr for EnumInterval<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s == "{}" {
            return Ok(Self::empty());
        }

        // `{...}` (other than `{}`) is the set form for `MaybeDisjoint` /
        // `IntervalSet`; a single interval can't be parsed from it.
        if s.starts_with('{') {
            return Err(ParseIntervalError::Syntax);
        }

        let (open, body, close) = peel(s).ok_or(ParseIntervalError::Syntax)?;
        let (lhs_str, rhs_str) = split_at_comma(body).ok_or(ParseIntervalError::Syntax)?;
        let lhs_str = lhs_str.trim();
        let rhs_str = rhs_str.trim();

        match (lhs_str, rhs_str) {
            ("..", "..") => {
                if open != '(' || close != ')' {
                    return Err(ParseIntervalError::Syntax);
                }
                Ok(Self::unbounded())
            }
            ("..", r) => {
                if open != '(' {
                    return Err(ParseIntervalError::Syntax);
                }
                let val = T::from_str(r).map_err(ParseIntervalError::Element)?;
                match close {
                    ']' => Ok(Self::try_unbound_closed(val)?),
                    ')' => Ok(Self::try_unbound_open(val)?),
                    _ => Err(ParseIntervalError::Syntax),
                }
            }
            (l, "..") => {
                if close != ')' {
                    return Err(ParseIntervalError::Syntax);
                }
                let val = T::from_str(l).map_err(ParseIntervalError::Element)?;
                match open {
                    '[' => Ok(Self::try_closed_unbound(val)?),
                    '(' => Ok(Self::try_open_unbound(val)?),
                    _ => Err(ParseIntervalError::Syntax),
                }
            }
            (l, r) => {
                let lhs = T::from_str(l).map_err(ParseIntervalError::Element)?;
                let rhs = T::from_str(r).map_err(ParseIntervalError::Element)?;
                match (open, close) {
                    ('[', ']') => Ok(Self::try_closed(lhs, rhs)?),
                    ('(', ')') => Ok(Self::try_open(lhs, rhs)?),
                    ('[', ')') => Ok(Self::try_closed_open(lhs, rhs)?),
                    ('(', ']') => Ok(Self::try_open_closed(lhs, rhs)?),
                    _ => Err(ParseIntervalError::Syntax),
                }
            }
        }
    }
}

/// Parses a [`FiniteInterval`] — accepts only `{}` and the four finite
/// bracket combinations. Half-bounded and unbounded inputs yield
/// [`ParseIntervalError::Syntax`].
impl<T> FromStr for FiniteInterval<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = EnumInterval::<T>::from_str(s)?;
        FiniteInterval::try_from(inner).map_err(|_| ParseIntervalError::Syntax)
    }
}

/// Parses a [`HalfInterval`] — accepts only the four half-bounded
/// forms. Empty / finite / fully-unbounded inputs yield
/// [`ParseIntervalError::Syntax`].
impl<T> FromStr for HalfInterval<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = EnumInterval::<T>::from_str(s)?;
        HalfInterval::try_from(inner).map_err(|_| ParseIntervalError::Syntax)
    }
}

fn peel(s: &str) -> Option<(char, &str, char)> {
    let mut chars = s.chars();
    let open = chars.next()?;
    if open != '[' && open != '(' {
        return None;
    }
    let close = s.chars().next_back()?;
    if close != ']' && close != ')' {
        return None;
    }
    let body_start = open.len_utf8();
    let body_end = s.len() - close.len_utf8();
    if body_start > body_end {
        return None;
    }
    Some((open, &s[body_start..body_end], close))
}

fn split_at_comma(body: &str) -> Option<(&str, &str)> {
    let idx = body.find(',')?;
    Some((&body[..idx], &body[idx + 1..]))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};

    // ---- happy path: every Display form round-trips ----

    #[test]
    fn round_trip_finite_closed() {
        let x = EnumInterval::closed(0_i32, 10);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<i32>>().unwrap(), x);
    }

    #[test]
    fn round_trip_finite_open() {
        let x = EnumInterval::open(0.0_f64, 10.0);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
    }

    #[test]
    fn round_trip_finite_closed_open() {
        let x = EnumInterval::closed_open(0.0_f64, 10.0);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
    }

    #[test]
    fn round_trip_finite_open_closed() {
        let x = EnumInterval::open_closed(0.0_f64, 10.0);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
    }

    #[test]
    fn round_trip_half_closed_unbound() {
        let x = EnumInterval::closed_unbound(0_i32);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<i32>>().unwrap(), x);
    }

    #[test]
    fn round_trip_half_open_unbound() {
        let x = EnumInterval::open_unbound(0.5_f64);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
    }

    #[test]
    fn round_trip_half_unbound_closed() {
        let x = EnumInterval::unbound_closed(10_i32);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<i32>>().unwrap(), x);
    }

    #[test]
    fn round_trip_half_unbound_open() {
        let x = EnumInterval::unbound_open(0.5_f64);
        assert_eq!(std::format!("{x}").parse::<EnumInterval<f64>>().unwrap(), x);
    }

    #[test]
    fn round_trip_unbounded() {
        let x = EnumInterval::<i32>::unbounded();
        assert_eq!(std::format!("{x}").parse::<EnumInterval<i32>>().unwrap(), x);
    }

    #[test]
    fn round_trip_empty() {
        let x = EnumInterval::<i32>::empty();
        assert_eq!(std::format!("{x}").parse::<EnumInterval<i32>>().unwrap(), x);
    }

    // ---- whitespace lenience ----

    #[test]
    fn whitespace_around_delims_is_ignored() {
        let x: EnumInterval<i32> = "  [ 0 , 10 ]  ".parse().unwrap();
        assert_eq!(x, EnumInterval::closed(0, 10));
    }

    // ---- negative endpoints ----

    #[test]
    fn negative_endpoints() {
        let x: EnumInterval<i32> = "[-10, -1]".parse().unwrap();
        assert_eq!(x, EnumInterval::closed(-10, -1));
    }

    // ---- variant-specific impls ----

    #[test]
    fn finite_impl_accepts_finite_and_empty() {
        let x: FiniteInterval<i32> = "[0, 10]".parse().unwrap();
        assert_eq!(x, FiniteInterval::closed(0, 10));

        let x: FiniteInterval<i32> = "{}".parse().unwrap();
        assert_eq!(x, FiniteInterval::empty());
    }

    #[test]
    fn finite_impl_rejects_unbounded() {
        let r: Result<FiniteInterval<i32>, _> = "[0, ..)".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<FiniteInterval<i32>, _> = "(.., ..)".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn half_impl_accepts_half_only() {
        let x: HalfInterval<i32> = "[0, ..)".parse().unwrap();
        assert_eq!(x, HalfInterval::closed_unbound(0));

        let r: Result<HalfInterval<i32>, _> = "[0, 10]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<HalfInterval<i32>, _> = "{}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    // ---- negative tests: syntax errors ----

    #[test]
    fn rejects_set_form() {
        // `{[0, 5], [10, 15]}` is MaybeDisjoint / IntervalSet syntax,
        // not a single interval.
        let r: Result<EnumInterval<i32>, _> = "{[0, 5], [10, 15]}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn rejects_closed_bracket_on_unbounded_side() {
        // `[.., x]` — closed bracket on the unbounded side is invalid.
        let r: Result<EnumInterval<i32>, _> = "[.., 5]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<EnumInterval<i32>, _> = "[0, ..]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<EnumInterval<i32>, _> = "[.., ..]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn rejects_missing_delims() {
        let r: Result<EnumInterval<i32>, _> = "0, 10".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn rejects_missing_comma() {
        let r: Result<EnumInterval<i32>, _> = "[0 10]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn rejects_garbage() {
        let r: Result<EnumInterval<i32>, _> = "".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<EnumInterval<i32>, _> = "[]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    // ---- negative tests: element and bound errors ----

    #[test]
    fn surfaces_element_parse_error() {
        let r: Result<EnumInterval<i32>, _> = "[abc, 10]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Element(_))));
    }

    #[test]
    fn rejects_crossed_bounds() {
        let r: Result<EnumInterval<i32>, _> = "[10, 0]".parse();
        assert!(matches!(r, Err(ParseIntervalError::InvalidBoundPair)));
    }

    #[test]
    fn rejects_nan_element() {
        let r: Result<EnumInterval<f64>, _> = "[NaN, 0]".parse();
        assert!(matches!(r, Err(ParseIntervalError::InvalidElement)));
    }
}

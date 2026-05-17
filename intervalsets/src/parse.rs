//! `FromStr` for [`Interval`] and [`IntervalSet`].
//!
//! [`Interval::from_str`](Interval) delegates to the inner
//! [`EnumInterval`] parser. [`IntervalSet::from_str`](IntervalSet)
//! accepts both the bare §2 interval form (single-piece set) and the
//! brace-wrapped §3.1 set form (any piece count). See
//! `docs/specs/string_repr.md` for the full grammar; canonical
//! emission drops outer braces for piece counts ≤ 1, but the parser
//! also accepts brace-wrapped inputs at those counts as non-canonical
//! shapes. `IntervalSet::new` handles sorting, merging, and dropping
//! empty pieces.

use core::str::FromStr;

use intervalsets_core::error::ParseIntervalError;
use intervalsets_core::sets::EnumInterval;

use crate::numeric::Element;
use crate::{Interval, IntervalSet};

/// Parses an [`Interval`] from its `Display` form. See
/// [`EnumInterval`'s `FromStr` impl](intervalsets_core::sets::EnumInterval)
/// for the full grammar.
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let x: Interval<i32> = "[0, 10]".parse().unwrap();
/// assert_eq!(x, Interval::closed(0, 10));
///
/// let x: Interval<f64> = "(0.0, 10.0]".parse().unwrap();
/// assert_eq!(x, Interval::open_closed(0.0, 10.0));
///
/// let x: Interval<i32> = "(.., 10)".parse().unwrap();
/// assert_eq!(x, Interval::unbound_open(10));
///
/// let x: Interval<i32> = "{}".parse().unwrap();
/// assert_eq!(x, Interval::empty());
///
/// // Round-trip with Display.
/// let x = Interval::open_closed(1.5, 7.5);
/// assert_eq!(format!("{x}").parse::<Interval<f64>>().unwrap(), x);
/// ```
impl<T> FromStr for Interval<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EnumInterval::<T>::from_str(s).map(Self::from)
    }
}

/// Parses an [`IntervalSet`] from its `Display` form. The grammar
/// accepts two shapes:
///
/// | Form                      | Example                            |
/// |---------------------------|------------------------------------|
/// | bare §2 interval form     | `{}`, `[0, 10]`, `(.., 5]`, `(.., ..)` |
/// | brace-wrapped set form    | `{[0, 5] U [10, 15] U [20, 30]}`   |
///
/// A bare interval is treated as a zero- or one-piece set; the
/// brace-wrapped form carries any piece count. Each piece is a valid
/// [`EnumInterval`] (see its `FromStr` impl). Pieces don't need to be
/// sorted, normalized, or non-overlapping at the input level —
/// [`IntervalSet::new`] sorts, merges connected pieces, and drops
/// empties to satisfy the `IntervalSet` invariants.
///
/// Canonical emission (per `docs/specs/string_repr.md` §3.1) uses the
/// bare form for piece counts ≤ 1 and the brace-wrapped form for
/// counts ≥ 2; brace-wrapped inputs at counts ≤ 1 (e.g. `{[0, 10]}`)
/// are accepted but non-canonical, and re-emission normalizes them.
///
/// # Examples
///
/// ```
/// use intervalsets::prelude::*;
///
/// let empty: IntervalSet<i32> = "{}".parse().unwrap();
/// assert_eq!(empty, IntervalSet::empty());
///
/// // Single-piece sets parse from the bare interval form (canonical) ...
/// let single: IntervalSet<i32> = "[0, 10]".parse().unwrap();
/// assert_eq!(single, IntervalSet::from(Interval::closed(0, 10)));
///
/// // ... or from the brace-wrapped form (non-canonical input, accepted).
/// let single: IntervalSet<i32> = "{[0, 10]}".parse().unwrap();
/// assert_eq!(single, IntervalSet::from(Interval::closed(0, 10)));
///
/// let multi: IntervalSet<i32> = "{[0, 5] U [10, 15]}".parse().unwrap();
/// let expected = Interval::closed(0, 5).union(Interval::closed(10, 15));
/// assert_eq!(multi, expected);
///
/// // Round-trip with Display.
/// let x = Interval::closed(0, 5).union(Interval::closed(10, 15));
/// assert_eq!(format!("{x}").parse::<IntervalSet<i32>>().unwrap(), x);
///
/// // Unsorted / overlapping pieces are normalized by `IntervalSet::new`.
/// let x: IntervalSet<i32> = "{[10, 15] U [0, 5]}".parse().unwrap();
/// let y: IntervalSet<i32> = "{[0, 5] U [10, 15]}".parse().unwrap();
/// assert_eq!(x, y);
/// ```
impl<T> FromStr for IntervalSet<T>
where
    T: Element + FromStr,
{
    type Err = ParseIntervalError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        // Brace-wrapped set form: matched outer `{` and `}`.
        if s.starts_with('{') && s.ends_with('}') {
            // Strip exactly one byte on each side (ASCII `{` and `}`).
            let body = s[1..s.len() - 1].trim();
            if body.is_empty() {
                return Ok(IntervalSet::empty());
            }
            let segments = split_on_top_level_u(body)?;
            let mut pieces = Vec::with_capacity(segments.len());
            for seg in segments {
                pieces.push(seg.trim().parse::<Interval<T>>()?);
            }
            return Ok(IntervalSet::new(pieces));
        }
        // Bare §2 interval form: treat as a single-piece set.
        Interval::<T>::from_str(s).map(Self::from)
    }
}

/// Walk `body` char by char, tracking `[](){}` depth. At depth 0, split
/// on a `U` that is flanked by ASCII whitespace on both sides. Reject
/// empty segments (leading/trailing/double `U`).
fn split_on_top_level_u<E>(body: &str) -> Result<Vec<&str>, ParseIntervalError<E>> {
    let bytes = body.as_bytes();
    let mut segments = Vec::new();
    let mut depth: i32 = 0;
    let mut seg_start: usize = 0;
    let mut i: usize = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'[' | b'(' | b'{' => depth += 1,
            b']' | b')' | b'}' => depth -= 1,
            b'U' if depth == 0 => {
                let prev_ws = i > 0 && bytes[i - 1].is_ascii_whitespace();
                let next_ws = i + 1 < bytes.len() && bytes[i + 1].is_ascii_whitespace();
                if prev_ws && next_ws {
                    let seg = body[seg_start..i].trim();
                    if seg.is_empty() {
                        return Err(ParseIntervalError::Syntax);
                    }
                    segments.push(seg);
                    seg_start = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let tail = body[seg_start..].trim();
    if tail.is_empty() {
        return Err(ParseIntervalError::Syntax);
    }
    segments.push(tail);
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::{FiniteFactory, HalfBoundedFactory, UnboundedFactory};
    use crate::ops::Union;

    #[test]
    fn round_trip_each_form() {
        let cases: [Interval<f64>; 9] = [
            Interval::closed(0.0, 10.0),
            Interval::open(0.0, 10.0),
            Interval::closed_open(0.0, 10.0),
            Interval::open_closed(0.0, 10.0),
            Interval::closed_unbound(0.0),
            Interval::open_unbound(0.0),
            Interval::unbound_closed(10.0),
            Interval::unbound_open(10.0),
            Interval::unbounded(),
        ];
        for x in cases {
            let printed = format!("{x}");
            let parsed: Interval<f64> = printed.parse().unwrap();
            assert_eq!(parsed, x, "round-trip failed for {printed}");
        }
    }

    #[test]
    fn empty_round_trip() {
        let x = Interval::<i32>::empty();
        let parsed: Interval<i32> = format!("{x}").parse().unwrap();
        assert_eq!(parsed, x);
    }

    // ---- IntervalSet FromStr ----

    #[test]
    fn set_empty() {
        let x: IntervalSet<i32> = "{}".parse().unwrap();
        assert_eq!(x, IntervalSet::empty());
    }

    #[test]
    fn set_single_piece() {
        let x: IntervalSet<i32> = "{[0, 10]}".parse().unwrap();
        assert_eq!(x, IntervalSet::from(Interval::closed(0, 10)));
    }

    #[test]
    fn set_multi_piece() {
        let x: IntervalSet<i32> = "{[0, 5] U [10, 15]}".parse().unwrap();
        let expected = Interval::closed(0, 5).union(Interval::closed(10, 15));
        assert_eq!(x, expected);
    }

    #[test]
    fn set_three_pieces_with_half_open() {
        let x: IntervalSet<f64> = "{(.., -10.0] U (0.0, 1.0) U [5.0, ..)}".parse().unwrap();
        let expected = Interval::unbound_closed(-10.0_f64)
            .union(Interval::open(0.0, 1.0))
            .union(Interval::closed_unbound(5.0));
        assert_eq!(x, expected);
    }

    #[test]
    fn set_normalizes_unsorted() {
        let unsorted: IntervalSet<i32> = "{[10, 15] U [0, 5]}".parse().unwrap();
        let sorted: IntervalSet<i32> = "{[0, 5] U [10, 15]}".parse().unwrap();
        assert_eq!(unsorted, sorted);
    }

    #[test]
    fn set_normalizes_overlapping() {
        let overlap: IntervalSet<i32> = "{[0, 10] U [5, 15]}".parse().unwrap();
        let merged: IntervalSet<i32> = "{[0, 15]}".parse().unwrap();
        assert_eq!(overlap, merged);
    }

    #[test]
    fn set_whitespace_tolerance() {
        let x: IntervalSet<i32> = "  {  [0, 5]  U  [10, 15]  }  ".parse().unwrap();
        let expected = Interval::closed(0, 5).union(Interval::closed(10, 15));
        assert_eq!(x, expected);
    }

    #[test]
    fn set_round_trip() {
        let cases: [IntervalSet<i32>; 4] = [
            IntervalSet::empty(),
            IntervalSet::from(Interval::closed(0, 10)),
            Interval::closed(0, 5).union(Interval::closed(10, 15)),
            Interval::unbound_closed(-10)
                .union(Interval::open(0, 5))
                .union(Interval::closed_unbound(20)),
        ];
        for x in cases {
            let printed = format!("{x}");
            let parsed: IntervalSet<i32> = printed.parse().unwrap();
            assert_eq!(parsed, x, "round-trip failed for {printed}");
        }
    }

    #[test]
    fn set_accepts_empty_piece() {
        let mixed: IntervalSet<i32> = "{[0, 5] U {} U [10, 15]}".parse().unwrap();
        let expected = Interval::closed(0, 5).union(Interval::closed(10, 15));
        assert_eq!(mixed, expected);

        let all_empty: IntervalSet<i32> = "{{} U {}}".parse().unwrap();
        assert_eq!(all_empty, IntervalSet::empty());
    }

    #[test]
    fn set_accepts_bare_interval_form() {
        // Spec §3.1: bare §2 forms parse as zero- or one-piece sets.
        let s: IntervalSet<i32> = "[0, 10]".parse().unwrap();
        assert_eq!(s, IntervalSet::from(Interval::closed(0, 10)));

        let s: IntervalSet<f64> = "(.., 5.0]".parse().unwrap();
        assert_eq!(s, IntervalSet::from(Interval::unbound_closed(5.0)));

        let s: IntervalSet<i32> = "(.., ..)".parse().unwrap();
        assert_eq!(s, IntervalSet::from(Interval::unbounded()));
    }

    #[test]
    fn set_rejects_mismatched_braces() {
        let r: Result<IntervalSet<i32>, _> = "{[0, 10]".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));

        let r: Result<IntervalSet<i32>, _> = "[0, 10]}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn set_rejects_unbraced_multi_piece() {
        // Without outer braces, ` U ` is not a top-level separator —
        // the bare-interval path tries to parse it as a single piece
        // and rejects (either Syntax or Element, depending on which
        // sub-rule fires first).
        let r: Result<IntervalSet<i32>, _> = "[0, 5] U [10, 15]".parse();
        assert!(r.is_err());
    }

    #[test]
    fn set_rejects_leading_separator() {
        let r: Result<IntervalSet<i32>, _> = "{ U [0, 10]}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn set_rejects_trailing_separator() {
        let r: Result<IntervalSet<i32>, _> = "{[0, 10] U }".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn set_rejects_double_separator() {
        let r: Result<IntervalSet<i32>, _> = "{[0, 5] U  U [10, 15]}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn set_rejects_bad_piece() {
        let r: Result<IntervalSet<i32>, _> = "{[0, 5] U [10}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Syntax)));
    }

    #[test]
    fn set_surfaces_element_parse_error() {
        let r: Result<IntervalSet<i32>, _> = "{[abc, 10]}".parse();
        assert!(matches!(r, Err(ParseIntervalError::Element(_))));
    }
}

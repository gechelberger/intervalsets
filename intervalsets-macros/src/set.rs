//! Splitting + pre-validation for the `set!` macro.
//!
//! `set!`'s input mirrors `IntervalSet`'s runtime `FromStr` grammar
//! (see `docs/specs/string_repr.md`):
//!
//! - Any bare §2 interval form (`{}`, `[0, 10]`, `(.., 5]`, `(.., ..)`,
//!   …) — treated as a zero- or one-piece set.
//! - `{piece U piece U ...}` — brace-wrapped multi-piece form, ASCII
//!   `U` separator with whitespace on both sides.
//!
//! Each piece is a valid interval per the existing
//! [`shape::parse_shape`](crate::shape::parse_shape) grammar; pre-parsing
//! each segment at expansion time surfaces compile errors with clear
//! per-piece feedback.
//!
//! This mirrors (but does not share code with) the runtime
//! `IntervalSet: FromStr` impl. Same precedent as the existing shape
//! parser duplication: avoid dragging the outer `intervalsets` crate
//! into the proc-macro's build graph for ~40 lines of pure-string
//! logic.

use crate::shape::{parse_shape, Form, ShapeError};

/// Result of parsing a `set!` literal: an ordered list of validated
/// pieces. `pieces.is_empty()` corresponds to the input `{}`.
pub(crate) struct SetParts {
    pub pieces: Vec<Form>,
}

pub(crate) enum SetError {
    /// A `U` separator had no segment on one side.
    EmptySegment,
    /// A piece was not a valid interval literal.
    Shape(ShapeError),
}

impl SetError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::EmptySegment => {
                "empty segment between `U` separators; check for leading, trailing, or doubled `U`"
                    .to_string()
            }
            Self::Shape(e) => e.message().to_string(),
        }
    }
}

pub(crate) fn parse_set(s: &str) -> Result<SetParts, SetError> {
    let s = s.trim();
    // Brace-wrapped set form: matched outer `{` and `}`.
    if s.starts_with('{') && s.ends_with('}') {
        // Strip exactly one byte on each side (ASCII `{` and `}`).
        let body = s[1..s.len() - 1].trim();
        if body.is_empty() {
            return Ok(SetParts { pieces: Vec::new() });
        }
        let segments = split_on_top_level_u(body)?;
        let mut pieces = Vec::with_capacity(segments.len());
        for seg in segments {
            let form = parse_shape(seg.trim()).map_err(SetError::Shape)?;
            pieces.push(form);
        }
        return Ok(SetParts { pieces });
    }
    // Bare §2 interval form: treat as a single-piece set.
    let form = parse_shape(s).map_err(SetError::Shape)?;
    Ok(SetParts { pieces: vec![form] })
}

/// Walk `body` byte by byte, tracking `[](){}` depth. At depth 0,
/// split on a `U` whose surroundings (after `body` has been trimmed)
/// are whitespace or the body boundary, with at least one actual
/// whitespace on a side so a bare `U` body doesn't count as a
/// separator. Empty segments produce `EmptySegment`.
fn split_on_top_level_u(body: &str) -> Result<Vec<&str>, SetError> {
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
                let prev_is_ws = i > 0 && bytes[i - 1].is_ascii_whitespace();
                let next_is_ws = i + 1 < bytes.len() && bytes[i + 1].is_ascii_whitespace();
                let prev_boundary = i == 0 || prev_is_ws;
                let next_boundary = i + 1 == bytes.len() || next_is_ws;
                // Require at least one *actual* whitespace neighbor so a
                // bare `U` body (e.g. body = "U") doesn't count as a separator.
                if prev_boundary && next_boundary && (prev_is_ws || next_is_ws) {
                    let seg = body[seg_start..i].trim();
                    if seg.is_empty() {
                        return Err(SetError::EmptySegment);
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
        return Err(SetError::EmptySegment);
    }
    segments.push(tail);
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_count(s: &str) -> Result<usize, &'static str> {
        match parse_set(s) {
            Ok(p) => Ok(p.pieces.len()),
            Err(SetError::EmptySegment) => Err("empty_segment"),
            Err(SetError::Shape(_)) => Err("shape"),
        }
    }

    #[test]
    fn empty() {
        assert_eq!(ok_count("{}"), Ok(0));
        assert_eq!(ok_count("{ }"), Ok(0));
        assert_eq!(ok_count("  { }  "), Ok(0));
    }

    #[test]
    fn single_piece() {
        assert_eq!(ok_count("{[0, 10]}"), Ok(1));
        assert_eq!(ok_count("{(0, 10)}"), Ok(1));
        assert_eq!(ok_count("{(.., 10]}"), Ok(1));
        assert_eq!(ok_count("{(.., ..)}"), Ok(1));
    }

    #[test]
    fn multi_piece() {
        assert_eq!(ok_count("{[0, 5] U [10, 15]}"), Ok(2));
        assert_eq!(ok_count("{[0, 1] U (10, 24) U [20, 35)}"), Ok(3));
    }

    #[test]
    fn whitespace_tolerance() {
        assert_eq!(ok_count("{  [0, 5]   U   [10, 15]  }"), Ok(2));
        assert_eq!(ok_count("  {  [0, 5]\tU\n[10, 15]  }  "), Ok(2));
    }

    #[test]
    fn u_inside_bound_not_split() {
        // `U` inside a bracket is an identifier in a bound expression,
        // not a separator. Depth > 0 at that point, so we don't split.
        assert_eq!(ok_count("{[U, V]}"), Ok(1));
    }

    #[test]
    fn accepts_bare_interval_form() {
        // Spec §3.1: bare §2 forms are accepted as one-piece sets.
        assert_eq!(ok_count("[0, 10]"), Ok(1));
        assert_eq!(ok_count("(.., 5]"), Ok(1));
        assert_eq!(ok_count("(.., ..)"), Ok(1));
        // Empty (§2.4) form — produces zero pieces.
        assert_eq!(ok_count("{}"), Ok(0));
    }

    #[test]
    fn rejects_mismatched_braces() {
        // No longer "missing braces" — the bare-form fallback runs
        // and parse_shape rejects with its own error.
        assert_eq!(ok_count("{[0, 10]"), Err("shape"));
        assert_eq!(ok_count("[0, 10]}"), Err("shape"));
        assert_eq!(ok_count(""), Err("shape"));
    }

    #[test]
    fn rejects_leading_u() {
        assert_eq!(ok_count("{ U [0, 10]}"), Err("empty_segment"));
    }

    #[test]
    fn rejects_trailing_u() {
        assert_eq!(ok_count("{[0, 10] U }"), Err("empty_segment"));
    }

    #[test]
    fn rejects_double_u() {
        assert_eq!(ok_count("{[0, 5] U  U [10, 15]}"), Err("empty_segment"));
    }

    #[test]
    fn rejects_bad_piece() {
        // `[10` (no closer) — shape parser rejects.
        assert_eq!(ok_count("{[0, 5] U [10}"), Err("shape"));
        // crossed bounds in a piece — shape parser passes (cross check is at expand);
        // but missing brackets — shape parser rejects.
        assert_eq!(ok_count("{0, 10}"), Err("shape"));
    }

    #[test]
    fn non_whitespace_flanked_u_not_separator() {
        // `Up` is an identifier; we should not split inside it. The body
        // ends up being a single segment that fails shape parsing.
        assert_eq!(ok_count("{[0, Up]}"), Ok(1));
    }
}

//! Structural ("shape") parser for interval literal strings.
//!
//! Operates on the raw `&str` from the input `LitStr`. Recognizes the
//! ten grammar forms and returns a [`Form`] carrying the bound bodies
//! as `proc_macro2::TokenStream`s (for downstream parsing as
//! `syn::Expr`s). Does not invoke `T::from_str` or touch element types.
//!
//! This is intentionally a small, independent re-implementation of the
//! structural parts of `intervalsets_core::parse::FromStr` (`peel` and
//! `split_at_comma`). Keeping it duplicated avoids dragging
//! `intervalsets-core` into the proc-macro's build graph for ~30 lines
//! of pure-string logic.
//!
//! Splitting on the top-level comma uses token-stream parsing rather
//! than character scanning, so bound expressions can legitimately
//! contain commas inside `()`, `[]`, `{}`, char literals, or string
//! literals: `[(1, 2).0, 10]`, `[BigDecimal::from(0), x]`. The one
//! caveat is `<>` — turbofish containing top-level commas
//! (`Vec::<i32, A>::new()`) is not balanced and will split at the
//! inner comma; this is rare enough not to bother with in v1.

use proc_macro2::{TokenStream, TokenTree};

/// One row of the interval grammar.
pub(crate) enum Form {
    /// `{}`
    Empty,
    /// `[a, b]`
    Closed(TokenStream, TokenStream),
    /// `(a, b)`
    Open(TokenStream, TokenStream),
    /// `[a, b)`
    ClosedOpen(TokenStream, TokenStream),
    /// `(a, b]`
    OpenClosed(TokenStream, TokenStream),
    /// `[a, ..)`
    ClosedUnbound(TokenStream),
    /// `(a, ..)`
    OpenUnbound(TokenStream),
    /// `(.., a]`
    UnboundClosed(TokenStream),
    /// `(.., a)`
    UnboundOpen(TokenStream),
    /// `(.., ..)`
    Unbounded,
}

/// Structural failure modes for an interval literal string.
pub(crate) enum ShapeError {
    MissingBrackets,
    MissingComma,
    ClosedOnUnboundedSide,
    SetNotation,
}

impl ShapeError {
    pub(crate) fn message(&self) -> &'static str {
        match self {
            Self::MissingBrackets => {
                "malformed interval syntax: expected one of `[a, b]` `(a, b)` `[a, b)` \
                 `(a, b]` `[a, ..)` `(a, ..)` `(.., a]` `(.., a)` `(.., ..)` or `{}`"
            }
            Self::MissingComma => "malformed interval syntax: missing `,` between bounds",
            Self::ClosedOnUnboundedSide => {
                "unbounded side must use an open delimiter — write `(.., x]` not `[.., x]` \
                 (infinity is never \"included\")"
            }
            Self::SetNotation => {
                "`{...}` is set notation; `interval!` produces a single interval — \
                 use `set!` for multi-piece sets, or `{}` for the empty interval"
            }
        }
    }
}

/// Classify an interval literal string into a [`Form`].
pub(crate) fn parse_shape(s: &str) -> Result<Form, ShapeError> {
    let s = s.trim();

    if s == "{}" {
        return Ok(Form::Empty);
    }

    if s.starts_with('{') {
        return Err(ShapeError::SetNotation);
    }

    let (open, body, close) = peel(s).ok_or(ShapeError::MissingBrackets)?;
    let (lhs, rhs) = split_body_at_comma(body)?;
    let lhs_is_dotdot = is_dotdot(&lhs);
    let rhs_is_dotdot = is_dotdot(&rhs);

    match (lhs_is_dotdot, rhs_is_dotdot) {
        (true, true) => {
            if open != '(' || close != ')' {
                return Err(ShapeError::ClosedOnUnboundedSide);
            }
            Ok(Form::Unbounded)
        }
        (true, false) => {
            if open != '(' {
                return Err(ShapeError::ClosedOnUnboundedSide);
            }
            match close {
                ']' => Ok(Form::UnboundClosed(rhs)),
                ')' => Ok(Form::UnboundOpen(rhs)),
                _ => Err(ShapeError::MissingBrackets),
            }
        }
        (false, true) => {
            if close != ')' {
                return Err(ShapeError::ClosedOnUnboundedSide);
            }
            match open {
                '[' => Ok(Form::ClosedUnbound(lhs)),
                '(' => Ok(Form::OpenUnbound(lhs)),
                _ => Err(ShapeError::MissingBrackets),
            }
        }
        (false, false) => match (open, close) {
            ('[', ']') => Ok(Form::Closed(lhs, rhs)),
            ('(', ')') => Ok(Form::Open(lhs, rhs)),
            ('[', ')') => Ok(Form::ClosedOpen(lhs, rhs)),
            ('(', ']') => Ok(Form::OpenClosed(lhs, rhs)),
            _ => Err(ShapeError::MissingBrackets),
        },
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

fn split_body_at_comma(body: &str) -> Result<(TokenStream, TokenStream), ShapeError> {
    let ts: TokenStream = body.parse().map_err(|_| ShapeError::MissingComma)?;

    let mut lhs: Vec<TokenTree> = Vec::new();
    let mut rhs: Vec<TokenTree> = Vec::new();
    let mut found_comma = false;

    for tt in ts {
        if found_comma {
            rhs.push(tt);
        } else if matches!(&tt, TokenTree::Punct(p) if p.as_char() == ',') {
            found_comma = true;
        } else {
            lhs.push(tt);
        }
    }

    if !found_comma {
        return Err(ShapeError::MissingComma);
    }

    Ok((TokenStream::from_iter(lhs), TokenStream::from_iter(rhs)))
}

fn is_dotdot(ts: &TokenStream) -> bool {
    let tokens: Vec<TokenTree> = ts.clone().into_iter().collect();
    if tokens.len() != 2 {
        return false;
    }
    // Spacing varies by what follows in the original source (e.g. in
    // `.., ..` the first `..`'s second dot is `Joint` because `,`
    // follows; in `.., 10` the second `..`'s second dot is `Alone`).
    // Exact-two-Punct-dots is enough — `...` or `..=` won't match
    // because their token count differs.
    matches!(&tokens[0], TokenTree::Punct(p) if p.as_char() == '.')
        && matches!(&tokens[1], TokenTree::Punct(p) if p.as_char() == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(s: &str) -> Result<&'static str, &'static str> {
        match parse_shape(s) {
            Ok(Form::Empty) => Ok("empty"),
            Ok(Form::Closed(_, _)) => Ok("closed"),
            Ok(Form::Open(_, _)) => Ok("open"),
            Ok(Form::ClosedOpen(_, _)) => Ok("closed_open"),
            Ok(Form::OpenClosed(_, _)) => Ok("open_closed"),
            Ok(Form::ClosedUnbound(_)) => Ok("closed_unbound"),
            Ok(Form::OpenUnbound(_)) => Ok("open_unbound"),
            Ok(Form::UnboundClosed(_)) => Ok("unbound_closed"),
            Ok(Form::UnboundOpen(_)) => Ok("unbound_open"),
            Ok(Form::Unbounded) => Ok("unbounded"),
            Err(ShapeError::MissingBrackets) => Err("missing_brackets"),
            Err(ShapeError::MissingComma) => Err("missing_comma"),
            Err(ShapeError::ClosedOnUnboundedSide) => Err("closed_on_unbounded"),
            Err(ShapeError::SetNotation) => Err("set_notation"),
        }
    }

    #[test]
    fn all_grammar_forms() {
        assert_eq!(classify("{}"), Ok("empty"));
        assert_eq!(classify("[0, 10]"), Ok("closed"));
        assert_eq!(classify("(0, 10)"), Ok("open"));
        assert_eq!(classify("[0, 10)"), Ok("closed_open"));
        assert_eq!(classify("(0, 10]"), Ok("open_closed"));
        assert_eq!(classify("[0, ..)"), Ok("closed_unbound"));
        assert_eq!(classify("(0, ..)"), Ok("open_unbound"));
        assert_eq!(classify("(.., 10]"), Ok("unbound_closed"));
        assert_eq!(classify("(.., 10)"), Ok("unbound_open"));
        assert_eq!(classify("(.., ..)"), Ok("unbounded"));
    }

    #[test]
    fn whitespace_tolerance() {
        assert_eq!(classify("  [ 0 , 10 ]  "), Ok("closed"));
        assert_eq!(classify("\t[0,\n10)\r\n"), Ok("closed_open"));
    }

    #[test]
    fn negative_endpoints() {
        assert_eq!(classify("[-10, -1]"), Ok("closed"));
    }

    #[test]
    fn nested_paren_in_bound() {
        // `(1, 2).0` contains a comma but inside parens — must not be split there.
        assert_eq!(classify("[(1, 2).0, 10]"), Ok("closed"));
    }

    #[test]
    fn function_call_in_bound() {
        assert_eq!(
            classify("[BigDecimal::from(0), BigDecimal::from(10)]"),
            Ok("closed")
        );
    }

    #[test]
    fn string_literal_in_bound() {
        // String literal with internal comma must not be split there.
        assert_eq!(classify(r#"["a,b", "c,d"]"#), Ok("closed"));
    }

    #[test]
    fn rejects_missing_brackets() {
        assert_eq!(classify("0, 10"), Err("missing_brackets"));
        assert_eq!(classify(""), Err("missing_brackets"));
        assert_eq!(classify("[0, 10"), Err("missing_brackets"));
        assert_eq!(classify("0, 10]"), Err("missing_brackets"));
    }

    #[test]
    fn rejects_missing_comma() {
        assert_eq!(classify("[0 10]"), Err("missing_comma"));
        assert_eq!(classify("[]"), Err("missing_comma"));
        assert_eq!(classify("()"), Err("missing_comma"));
    }

    #[test]
    fn rejects_closed_on_unbounded_side() {
        assert_eq!(classify("[.., 10]"), Err("closed_on_unbounded"));
        assert_eq!(classify("[0, ..]"), Err("closed_on_unbounded"));
        assert_eq!(classify("[.., ..]"), Err("closed_on_unbounded"));
        assert_eq!(classify("(.., ..]"), Err("closed_on_unbounded"));
        assert_eq!(classify("[.., ..)"), Err("closed_on_unbounded"));
    }

    #[test]
    fn rejects_set_notation() {
        assert_eq!(classify("{[0, 5], [10, 15]}"), Err("set_notation"));
        assert_eq!(classify("{0}"), Err("set_notation"));
    }
}

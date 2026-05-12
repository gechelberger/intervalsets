use core::convert::Infallible;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Bound-pair invariants violated. Covers two related conditions:
    ///
    /// 1. **Crossed bounds in a `FiniteInterval`** — `lhs > rhs` after
    ///    normalization. Raised by
    ///    [`FiniteInterval::try_new`](crate::sets::FiniteInterval::try_new)
    ///    and the interval types' `Deserialize` paths.
    /// 2. **Structural `OrdBoundPair` violations** — an `OrdBound` of
    ///    the wrong kind for its position (e.g. `RightUnbounded` on
    ///    the left), an unmatched empty marker, or `left.value() > right.value()`.
    ///    Raised by
    ///    [`OrdBoundPair::try_new`](crate::bound::ord::OrdBoundPair::try_new).
    ///
    /// Both contexts share this variant because callers rarely need to
    /// distinguish them in error handling. If a future use case
    /// requires distinguishing, this variant can be split additively
    /// (the enum is `#[non_exhaustive]`).
    #[error("interval or bound-pair invariants violated (crossed bounds, or structurally invalid OrdBoundPair)")]
    InvalidBoundPair,

    /// A bound's limit value was rejected as a valid bound limit.
    /// Covers two related conditions:
    ///
    /// 1. **Validate rejection** — `Element::validate` returned `None`.
    ///    Library float types reject `±INF` and `NaN` here; user types
    ///    override to enforce their own predicate.
    /// 2. **Comparison failure** — `partial_cmp` returned `None` (i.e.
    ///    [`TotalOrderError`]) on a value that reached a `try_cmp`
    ///    site. For library types this means NaN slipped past validate
    ///    via the Tier-4 bypass; for user types with intrinsic partial
    ///    order it can mean two individually-valid values are mutually
    ///    incomparable — surface that condition by tightening
    ///    `Element::validate` if it matters.
    ///
    /// The two contexts share this variant because the user-facing
    /// answer is the same: "this value isn't a usable bound limit."
    /// `From<TotalOrderError> for Error` produces this variant.
    #[error("bound limit rejected (validate or partial_cmp failure)")]
    InvalidBoundLimit,

    /// Arithmetic-on-bounds failure. Wraps [`MathError`].
    #[error(transparent)]
    Math(#[from] MathError),
}

impl From<TotalOrderError> for Error {
    fn from(_: TotalOrderError) -> Self {
        Error::InvalidBoundLimit
    }
}

impl From<Infallible> for Error {
    /// Lets a `TryAdd`/`TrySub`/`TryMul`/`TryDiv` impl with
    /// `Error = Infallible` (e.g. `BigInt`, `Saturating<T>`) satisfy
    /// the set-level bound `<T as TryAdd>::Error: Into<Error>` without
    /// going through the `MathError` intermediate. `From` impls don't
    /// compose, so this hop has to be spelled out.
    fn from(x: Infallible) -> Self {
        match x {}
    }
}

/// Arithmetic-on-bounds failure surfaced by value-level [`TryAdd`],
/// [`TrySub`], [`TryMul`], and [`TryDiv`] impls.
///
/// The two variants follow the C `<errno.h>` `ERANGE` / `EDOM` axis:
/// **result-side** failure (`Range`) vs **input-side** failure (`Domain`).
/// This axis carves the same joint cleanly across integer, float, decimal,
/// fixed-point, and big-int `T`s, so the same enum is the umbrella for
/// every library-provided endpoint type.
///
/// Mechanism-level mapping (documented bijection): `Range ≡ Overflow`,
/// `Domain ≡ NonFinite`. The variant names follow the math-stdlib axis
/// because it survives across `T` types without renaming.
///
/// [`TryAdd`]: crate::ops::math::TryAdd
/// [`TrySub`]: crate::ops::math::TrySub
/// [`TryMul`]: crate::ops::math::TryMul
/// [`TryDiv`]: crate::ops::math::TryDiv
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[non_exhaustive]
pub enum MathError {
    /// Result outside `T`'s representable range.
    ///
    /// Covers integer overflow (including signed `MIN / -1`) and floats
    /// producing `INF`. Mechanism-level: `Range ≡ Overflow`.
    #[error("arithmetic overflow")]
    Range,

    /// Operation undefined for the given inputs.
    ///
    /// Covers integer divide-by-zero and floats producing `NaN`.
    /// Mechanism-level: `Domain ≡ NonFinite`.
    ///
    /// Note: `1.0 / 0.0 = INF` is reported here as `Domain` even though
    /// strict C `<errno.h>` would call it `Range`. The implementation
    /// uses a single `is_finite()` check that does not split `INF` from
    /// `NaN`; adding an `is_nan()`-vs-`is_inf()` discriminator just to
    /// match the strict axis is not worth the churn.
    #[error("non-finite result")]
    Domain,
    // future, alloc-gated:
    // #[cfg(feature = "alloc")]
    // Custom(Box<dyn core::error::Error + Send + Sync>),
}

impl From<Infallible> for MathError {
    fn from(x: Infallible) -> Self {
        match x {}
    }
}

/// Failed comparison of `PartialOrd` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ::thiserror::Error)]
#[error("incomparable values")]
pub struct TotalOrderError;

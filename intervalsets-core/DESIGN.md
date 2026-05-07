# Design

This document is a designer/contributor reference for `intervalsets-core`. It
captures the principles we apply when shaping the public API surface and
contract guarantees. It is **not** a user document and is intentionally not
linked from rustdoc — user-facing rustdoc states what each API does and how it
can fail; the principles below explain why.

When making API or contract decisions, cite the principle by name (e.g. "P3" or
"the validating-API principle") so the reasoning carries forward.

A note on terminology: throughout this document, "validating API" / "validating
tier" refers to constructors and operations that enforce or rely on type
invariants — distinct from Rust's `safe` / `unsafe` keywords, which refer
specifically to memory safety. The crate is `forbid(unsafe_code)` end-to-end;
correctness here is about *invariant preservation*, not memory safety.

# Principles

## P1. Correctness is non-negotiable.

No other goal beats correctness. A faster wrong answer is worse than a slower
right one.

*In practice*: any change that admits a wrong-answer mode requires explicit
justification; ambiguity defaults to the more-validating choice.

## P2. Silent corruption is unacceptable.

A wrong answer that looks plausible is the worst failure mode. Catch it at the
boundary; surface it loudly.

*In practice*: errors are `Result` or panic, never best-effort fallback values.

## P3. The validating API preserves correctness.

A caller using only validating-tier APIs cannot produce a wrong answer. The
crate provides multiple validating shapes (truly infallible, infallible-when-
closed, `try_*` + panicking sugar) — those shapes are an ergonomics/performance
trade-off in the *contract*, not a principle. Bypass APIs (`*_assume_valid`)
exist as a deliberate, scoped exception, public only because the outer crate
needs them for performance.

*In practice*: the privacy boundary should approximate the validation
boundary. `*_assume_valid` items are the known gap, justified by performance
and labeled accordingly. Adding new public items that can produce wrong
answers (outside the assume-valid family) is a P3 violation and needs explicit
sign-off.

## P4. Types carry invariants; operations preserve them.

We decompose the problem into small types (`FiniteInterval`, `HalfInterval`,
`EnumInterval`, `MaybeDisjoint`) each with a precise, documented invariant.
Validation happens at construction; operations on validated inputs trust the
invariant and produce validated outputs.

*In practice*: `try_new` / `Deserialize` / `try_*` constructors reject NaN at
the boundary; downstream set-algebra ops never re-validate. Operations that
take valid inputs are *closed* under the invariant. This is the source of why
most set-algebra ops can be infallible without a `try_*` variant.

## P5. Explicit fallibility, with consistent vocabulary.

Operations that *can* fail say so in their type. Operations that can't, don't.
Identifiers carry semantics across the whole crate.

*In practice*: `try_*` is reserved for `Result`-fallible operations whose `Err`
is a logical violation. `Option`-on-domain operations get descriptive names
that surface the precondition (e.g. `MergeConnected`).

## P6. The bound axis and the fallibility axis are independent.

`T: PartialOrd` vs `T: Ord` is a *who-can-call* policy choice driven by float
support and per-trait performance trade-offs. `Result` vs infallible is a
*can-it-go-wrong* correctness choice. Stating one doesn't determine the other.

*In practice*: `Union` requires `T: Ord` not because it can fail (it can't —
see P4) but because of the stronger-guarantee policy in `numeric.rs:138-143`.
Documentation must keep these axes from getting confused.

## P7. Documentation is part of the contract.

A public item without a stated panic/error story is incomplete. Principles
inform the docs but don't appear in them — user-facing rustdoc states what the
API does and how it can fail, not why we made it that way.

*In practice*: every public trait gets a `# Contract` rustdoc section. Examples
cover the corner cases the contract names (NaN, empty, MIN/MAX), not just the
happy path.

## P8. Pathological inputs are first-class test cases.

NaN, empty, MIN/MAX, adjacency edges, normalization corner cases. The contract
is only as good as its test coverage.

*In practice*: every contract claim has a corresponding test exercising the
edge case it names — as named unit tests, not buried in property tests.

## P9. `no_std`, no-alloc.

Embedded use is a first-class target. Operations that could produce multiple
pieces use `MaybeDisjoint` (up to two pieces) instead of allocating; multi-piece
sets live in the outer crate.

*In practice*: `MaybeDisjoint` is the canonical "result is one or two pieces"
type. Any new producer that could return more than two pieces does not belong
in core.

//! Template: prove your custom `T`'s `try_op` impls are panic-free
//! under Kani symbolic execution.
//!
//! The `intervalsets-core` Tier 3a contract states that
//! `TryAdd::try_add` (and `TrySub` / `TryMul` / `TryDiv`) on a storage
//! type `T` **must not panic in release**. The crate verifies this
//! for its own library-provided storage types (integer primitives,
//! `f32` / `f64`, `Option<T>`, the feature-crate types under
//! `core-panic-canary/src/proofs/storage_types/`). For user-defined
//! `T`, the contract is honor-system — `intervalsets-core` cannot
//! force you to run a verifier — but the harness shape below is
//! cheap, mirrors what the internal canary does, and is the
//! recommended way to discharge the proof obligation.
//!
//! # How to use
//!
//! 1. Copy this file into your crate. `cargo kani` discovers `lib`,
//!    `bin`, and `tests` targets but **not** `examples/`, so for a
//!    self-runnable proof drop it under `tests/kani.rs` or
//!    `src/bin/kani.rs`. (It can stay under `examples/` if you only
//!    want it as compilable documentation.)
//! 2. Replace `MyType` with the type you want to verify, and replace
//!    the placeholder impls with yours.
//! 3. Run `cargo kani --tests` (for `tests/`) or `cargo kani --bin
//!    kani` (for `src/bin/`). Kani is Linux/Mac-only; on CI use
//!    `model-checking/kani-github-action`.
//!
//! Under regular `cargo` (no `--cfg kani`) the proofs compile out via
//! `#[cfg(kani)]`, so the file is a no-op that just confirms the
//! impls compile. Under `cargo kani` the proof attributes activate
//! and Kani enumerates every input bit pattern within the harness
//! bounds.
//!
//! # What the proof actually shows
//!
//! Bounded symbolic execution over your impl's full call graph: every
//! reachable arithmetic / array index / unwrap / assertion. If any
//! input could panic, Kani returns a counter-example. Strictly
//! stronger than running concrete fixtures because the proof covers
//! the entire input space.

use intervalsets_core::error::MathError;
use intervalsets_core::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

/// Replace with your real storage type.
///
/// This stand-in wraps an `i32` so Kani can drive the proof via
/// `kani::any::<i32>()`. For a type whose internal state is more than
/// one primitive field, either:
///   - implement `kani::Arbitrary` for your type, or
///   - construct your type field-by-field from `kani::any()` calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MyType(i32);

// === Replace these with your real impls ============================
//
// The bodies below use `checked_*` to surface integer overflow as
// `Err(MathError::Range)` and explicit divide-by-zero detection for
// `Err(MathError::Domain)`. Mirror the macro lineup in
// `intervalsets-core/src/ops/math/macros.rs` if your type has a
// similar shape, or roll your own — Kani only checks that nothing
// panics, not which Error variant you produce.

impl TryAdd for MyType {
    type Output = MyType;
    type Error = MathError;

    fn try_add(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .checked_add(rhs.0)
            .map(MyType)
            .ok_or(MathError::Range)
    }
}

impl TrySub for MyType {
    type Output = MyType;
    type Error = MathError;

    fn try_sub(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .checked_sub(rhs.0)
            .map(MyType)
            .ok_or(MathError::Range)
    }
}

impl TryMul for MyType {
    type Output = MyType;
    type Error = MathError;

    fn try_mul(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        self.0
            .checked_mul(rhs.0)
            .map(MyType)
            .ok_or(MathError::Range)
    }
}

impl TryDiv for MyType {
    type Output = MyType;
    type Error = MathError;

    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        if rhs.0 == 0 {
            return Err(MathError::Domain);
        }
        self.0
            .checked_div(rhs.0)
            .map(MyType)
            .ok_or(MathError::Range)
    }
}

// === Kani proofs ===================================================
//
// Under `cargo kani`, the `kani` cfg is set and the harnesses below
// are compiled in. Under regular `cargo`, the cfg is absent and the
// harnesses vanish — the file remains a valid no-op example.

#[cfg(kani)]
fn any_my_type() -> MyType {
    MyType(kani::any())
}

#[cfg(kani)]
#[kani::proof]
fn try_add_my_type_no_panic() {
    let _ = any_my_type().try_add(any_my_type());
}

#[cfg(kani)]
#[kani::proof]
fn try_sub_my_type_no_panic() {
    let _ = any_my_type().try_sub(any_my_type());
}

#[cfg(kani)]
#[kani::proof]
fn try_mul_my_type_no_panic() {
    let _ = any_my_type().try_mul(any_my_type());
}

#[cfg(kani)]
#[kani::proof]
fn try_div_my_type_no_panic() {
    let _ = any_my_type().try_div(any_my_type());
}

fn main() {
    // Keep the example self-contained as an executable. The proofs
    // run only under `cargo kani`; running `cargo run --example` just
    // exercises a couple of `try_*` calls so the impls don't bit-rot.
    let _ = MyType(2).try_add(MyType(3));
    let _ = MyType(2).try_sub(MyType(3));
    let _ = MyType(2).try_mul(MyType(3));
    let _ = MyType(2).try_div(MyType(3));
}

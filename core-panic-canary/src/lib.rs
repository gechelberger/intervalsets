//! Panic-free verification proofs for `intervalsets-core`.
//!
//! Each module under `proofs::` is a Kani harness that mirrors a
//! `[[bin]]` linker canary in `src/bin/`. Compiles to an empty lib
//! outside of Kani.

#![cfg_attr(not(kani), allow(dead_code))]

#[cfg(kani)]
pub mod proofs;

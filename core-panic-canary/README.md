# core-panic-canary

Verifies the panic-free claims documented in
`intervalsets-core/src/ops/mod.rs` for the Tier 1, Tier 2, and Tier 3
operations on `FiniteInterval`, `HalfInterval`, and `EnumInterval` via
Kani symbolic execution.

This crate is `publish = false` — it exists purely as a local / CI
verification harness. `intervalsets-core` is the package downstream
users actually depend on.

## What "panic-free" means here

`intervalsets-core/src/ops/mod.rs` defines four tiers of guarantee for
the set-algebra operations:

- **Tier 1** — cannot panic, cannot error, for *any* input.
- **Tier 2** — cannot panic given inputs that satisfy their type
  invariants (constructed via the validating factories).
- **Tier 3** — `try_*` returns `Result`; the sugar form panics by
  design on documented `Err` paths.
- **Tier 4** — `*_assume_valid` bypass; caller asserts the precondition.

This crate verifies that Tiers 1, 2, and the `try_*` form of Tier 3
hold their no-panic guarantees over the stated input domains. Tier 4
bypass APIs are out of scope.

## How verification works

For each in-scope operation there is a `#[kani::proof]` harness under
`src/proofs/<group>.rs`. Kani enumerates every input within the
harness's bounds (e.g. all `i64` pairs, up to the per-crate
`default-unwind`) and proves no panic edge is reachable. Strictly
stronger than running concrete fixtures because the proof covers the
entire input space, not just the values the caller happened to pick.

```sh
# install the pinned Kani toolkit (one-time)
just update-kani

# single harness (substring filter)
just kani contains_finite

# CI smoke set (one fast harness per fully-verified group)
just check-kani

# everything
just kani
```

`STATUS.md` tracks per-harness verification state — wall-clock times
are recorded after each successful local run and harnesses still
pending verification are listed under their group.

## Layout

```
src/
  lib.rs                   # cfg-gated kani entry; empty lib otherwise
  proofs/
    mod.rs                 # proof module index
    tier1.rs               # Tier 1 ops (Contains, Intersects, Connects)
    tier2_*.rs             # one file per Tier 2 op
    tier3_*.rs             # one file per Tier 3 try_* op
STATUS.md                  # per-harness Kani verification state
Cargo.toml                 # path-dep on intervalsets-core
```

## Adding a new harness

1. Add `#[kani::proof] fn <op>_<lhs>_<rhs>_<scalar>_no_panic()` under
   `src/proofs/<group>.rs`, following the existing naming.
2. Run `just kani <name>`; on success, record the wall-clock time in
   `STATUS.md`.
3. If the new harness completes a previously-partial group, add a
   fast representative to `just check-kani` and drop the group from
   that recipe's SKIPPED block.

## Future work: linker-canary mode

Kani proves the *logical* claim — no panic edge is reachable in the
abstract. It does not verify that LLVM actually dead-code-eliminates
the panic site in the release artifact users compile. A linker-canary
mode complements that.

### What it would buy us

- **Release-binary proof** (vs. logical proof). The `no_panic` macro
  emits a link-time symbol that resolves only if the optimizer
  eliminates every panic edge from the annotated function. A
  Kani-clean impl that LLVM happens to leave a panic stub in would
  pass Kani and fail the canary — that's the gap the canary closes.
- **Optimizer-regression early warning.** rustc / LLVM upgrades that
  perturb DCE on a hot path show up as a clear link error in CI
  rather than a slow runtime regression spotted later.

### Architecture: canary-owned wrappers

`no_panic` is a function-level attribute, so it can sit on a wrapper
function this crate owns rather than on the upstream impl. The
intended layout:

- `core-panic-canary` adds `no-panic = "0.1"` to its own dependencies.
  `intervalsets-core` stays untouched — no annotations, no feature
  flag, no optional deps.
- For each Tier 1/2/3 impl in scope, write a thin `#[no_panic]`
  wrapper in this crate that calls into the impl with concrete
  fixtures, e.g.:

  ```rust
  #[no_panic::no_panic]
  fn check_contains_finite_i64(a: FiniteInterval<i64>, b: i64) -> bool {
      a.contains(&b)
  }
  ```

  At `--release`, `#[inline(always)]` on the impl side lets the
  optimizer fold the impl body into the wrapper, so `no_panic`'s
  panic-edge analysis sees the full call graph.
- A set of `[[bin]]` entries (one per tier slice, mirroring the
  `tier1.rs` / `tier2_*.rs` / `tier3_*.rs` layout under `proofs/`)
  invokes every wrapper with `black_box` fixtures so the symbols
  aren't dead-code-eliminated.
- A `panic-check` justfile recipe runs `cargo build --release --bins`
  inside this crate. Release-mode is mandatory — debug builds skip
  the DCE that `no_panic` relies on.

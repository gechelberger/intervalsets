This repository uses the `just` task runner for most canonical devops. The available tasks can be found by invoking `just --list`. Run `just fmt` before each commit. Use `just test` for code changes, or `just test-all` if doc examples were updated. Run `just ci` for a signal that a changeset might be correct when ready to push. It should catch most regressions.

The pre-commit hook (lefthook) runs `just fmt-check`, `just clippy`, and `just typos` in parallel. A fmt diff or new clippy warning blocks the commit — run `just fmt` first.

## Workspace

- `intervalsets-core` — no_std/no_alloc, single- and two-piece (`MaybeDisjoint`) intervals.
- `intervalsets` — alloc-friendly multi-piece set built on the core.
- `core-panic-canary` — Kani proofs of panic-freedom on storage-type impls.
- `benchmarks` — Criterion benches.

## Where design context lives

Two directories, by lifecycle stage:

- **`scratch/`** (gitignored) — ephemeral working notes, audits, and in-flight investigations. Check before assuming a question is unconsidered, but expect the contents to be in motion. The agent should keep an index of topics covered.
- **`docs/design/`** — design docs that are close to ready. Currently `measure-api.md`, `storage-type-cast.md`. Treat as authoritative.
- **`docs/spec/`** -- design docs that are focused on the shape of an api, not the why, but the how.
- **`docs/primer/`** -- short, high-level quickstart guides to onboard new users or maintainers

Notes graduate from `scratch/` into `docs/design/` when they stabilize.

## Feature gates

`intervalsets-core` is no_std/no_alloc by default. Most storage-type integrations (`fixed`, `rust_decimal`, `bigdecimal`, `num-bigint`, `ordered-float`, `serde`, `quickcheck`, `approx`, `arbitrary`) live behind feature flags in `intervalsets-core/src/feat/`. A bare `cargo check` skips them; pass `--features` (or use `just ci`, which sweeps the canonical combinations) when working in those modules.

## Stability

Pre-1.0: breaking API changes are welcome until RC. Don't add backward-compatibility shims for hypothetical callers.

## Expected Available Tools

Most will have a standard form, executable through just. For example `just fmt` is preferable to `cargo fmt` because our formatting rules depend on the nightly toolchain.

### Intractable Just Tasks
- Never run any **powerset** tasks. These explode combinatorially. They should be run by maintainers on occasion to catch regressions but are too expensive to include in the development cycle. (Consider adding to CI on a monthly cycle).

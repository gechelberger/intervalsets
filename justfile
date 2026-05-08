# Common tasks for intervalsets dev.

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# the minimum supported rust version
MSRV := "1.87.0"

# the target rust version
RV := "stable"

# the kani symbolic-execution toolkit version. .github/workflows/rust.yml
# pins the same value — keep them in sync. Dependabot can't track this
# (kani-verifier isn't a cargo dep), so `update-kani` prints a warning
# when a newer release is on crates.io.
KANI_VERSION := "0.67.0"

# just run the tests
default: test

# bootstrap cargo-binstall (downloads prebuilt binaries instead of compiling).
# The only tool we still build from source; everything else uses binstall.
# Re-runs are no-ops when already current.
update-binstall:
    @echo 'bootstrap cargo-binstall'
    cargo install cargo-binstall --locked

# install and/or update build tools and environment
update-tools: update-binstall
    @echo 'update the dev environment'

    # update rustup
    rustup self update

    # install a no-std target
    rustup target add thumbv6m-none-eabi

    # update the default channel
    rustup update

    # make sure we have the MSRV toolchain installed
    rustup toolchain install {{ MSRV }}

    # make sure we have our target toolchain installed
    rustup toolchain install {{ RV }}

    # make sure that we have the nightly compiler for docs
    rustup toolchain install nightly

    # watch the docs as you work
    cargo binstall static-web-server --locked --no-confirm

    # parallel test runner
    cargo binstall cargo-nextest --locked --no-confirm

    # benchmarking
    cargo binstall cargo-criterion --locked --no-confirm

    # coverage
    cargo binstall cargo-llvm-cov --locked --no-confirm

    # debug macros
    cargo binstall cargo-expand --locked --no-confirm

    # check features
    cargo binstall cargo-hack --locked --no-confirm

    # check dependencies
    cargo binstall cargo-udeps --locked --no-confirm

    # check codebase for loose ends
    cargo binstall ripgrep --locked --no-confirm

    # license / advisory / source checks
    cargo binstall cargo-deny --locked --no-confirm

    # semver compatibility verification
    cargo binstall cargo-semver-checks --locked --no-confirm

    # typo detection
    cargo binstall typos-cli --locked --no-confirm

    # release automation (workspace-aware version bump + tag + publish)
    cargo binstall cargo-release --locked --no-confirm

# install lefthook binary (debian/ubuntu via cloudsmith apt repo)
[linux]
install-lefthook-bin:
    curl -1sLf 'https://dl.cloudsmith.io/public/evilmartians/lefthook/setup.deb.sh' | sudo -E bash
    sudo apt install -y lefthook

# install lefthook binary (windows via winget)
[windows]
install-lefthook-bin:
    winget install --id evilmartians.lefthook --silent

# install lefthook binary (macos via homebrew)
[macos]
install-lefthook-bin:
    brew install lefthook

# install lefthook pre-commit hooks
setup-hooks: install-lefthook-bin
    # wire up git hooks (configured via lefthook.yml at repo root)
    lefthook install

# Pinned to KANI_VERSION (kept in sync with .github/workflows/rust.yml).
# Chains check-kani-version afterward to surface upstream releases that
# dependabot can't see (kani-verifier isn't a cargo dep).
#
# install pinned Kani symbolic-execution tooling (used by the panic-free proofs)
update-kani: && check-kani-version
    cargo binstall kani-verifier@{{ KANI_VERSION }} --locked --no-confirm
    cargo kani setup

# Bumping the pin is a two-line change (justfile + .github/workflows/rust.yml).
#
# warn when crates.io has a newer kani-verifier than KANI_VERSION
[unix]
check-kani-version:
    #!/usr/bin/env bash
    set -uo pipefail
    latest="$(cargo search kani-verifier --limit 1 2>/dev/null | sed -n 's/^kani-verifier = "\([^"]*\)".*/\1/p')"
    if [ -n "$latest" ] && [ "$latest" != "{{ KANI_VERSION }}" ]; then
        printf '\n  WARN: kani-verifier %s is available (pinned: %s).\n        Bump KANI_VERSION in justfile and .github/workflows/rust.yml.\n\n' "$latest" "{{ KANI_VERSION }}"
    fi

[windows]
check-kani-version:
    @echo "kani-verifier pinned to {{ KANI_VERSION }}; latest on crates.io:"
    @cargo search kani-verifier --limit 1

# `-A missing-docs` and `-A rustdoc::missing-crate-level-docs` are transitional
# opt-outs matching the clippy CI job — workspace lints stay "warn" but doc
# build doesn't gate on them until the existing backlog is fixed.
# Tracked in #TRACKING_ISSUE.
#
# build the docs
[env("RUSTDOCFLAGS", "-D warnings -A missing-docs -A rustdoc::missing-crate-level-docs --cfg docsrs")]
doc:
    cargo +nightly doc \
        --workspace \
        --all-features \
        --no-deps \
        --exclude benchmarks

alias d := doc

# launch a file server for docs
doc-serve port="8080": doc
    static-web-server --root target/doc --port {{ port }} -z

[working-directory('intervalsets-core')]
doc-check-core:
    cargo rustc --lib -- -W missing-docs -W rustdoc::missing-crate-level-docs -W rustdoc::broken-intra-doc-links

[working-directory('intervalsets')]
doc-check-main:
    cargo rustc --lib -- -W missing-docs -W rustdoc::missing-crate-level-docs -W rustdoc::broken-intra-doc-links

doc-check: doc-check-core doc-check-main

# run the tests
test pattern="":
    cargo +{{ RV }} nextest run --all-features --no-fail-fast {{ pattern }}

alias t := test

# run the doctests (nextest does not execute doctests)
test-doc:
    cargo +{{ RV }} test --all-features --doc

# run unit tests and doctests
test-all: test test-doc

# format the code base
fmt:
    cargo +nightly fmt --all

# verify formatting (used by lefthook + ci)
fmt-check:
    cargo +nightly fmt --all -- --check

# run clippy with the workspace gate (used by lefthook + ci)
# `-A missing-docs` is a transitional opt-out matching the CI clippy job —
# workspace lint stays at "warn" so authors see warnings locally, but the
# gate doesn't fail on the existing backlog. Tracked in #TRACKING_ISSUE.
clippy:
    cargo +{{ RV }} clippy --all-features --all-targets -- -D warnings -A missing-docs

# license / advisory / source checks
deny:
    cargo deny --all-features check

# semver compatibility check (informational pre-1.0)
semver-checks:
    cargo semver-checks --workspace --baseline-rev 9062fa1e1cb72b22a09f525088399d03f7f65346

# typo scan
typos:
    typos

# auto-fix typos (review with `git diff` before committing)
fix-typos:
    typos --write-changes

# check the build
check:
    cargo +{{ RV }} check --all-features

# run the test suite against the msrv (unit/integration + doctests)
check-msrv:
    cargo +{{ MSRV }} nextest run --all-features
    cargo +{{ MSRV }} test --all-features --doc

# build against a no-std target
check-no-std:
    cargo +{{ RV }} hack check --package intervalsets-core --each-feature \
        --exclude-features std,num-bigint,bigdecimal,arbitrary,quickcheck \
        --target thumbv6m-none-eabi \
        --verbose

# check that all possible feature combinations compile
# 2^n possible combinations
[working-directory('intervalsets-core')]
check-core-feature-powerset:
    cargo +{{ RV }} hack check --feature-powerset --no-dev-deps

# check that all possible feature combinations compile
# 2^n possible combinations
[working-directory('intervalsets')]
check-main-feature-powerset:
    cargo +{{ RV }} hack check --feature-powerset --no-dev-deps

# check the benchmarks
check-bench:
    just bench --no-run

# check the dependency tree for unused deps
check-deps:
    cargo +nightly udeps --all-features --all-targets

# clean old build artifacts
clean:
    cargo clean

# run the micro benchmarks
bench pattern="":
    cargo +{{ RV }} criterion --package benchmarks {{ pattern }}

# run the core crate micro benchmarks
bench-core pattern="":
    just bench "--bench intervalsets_core {{ pattern }}"

# run the main crate micro benchmarks
bench-main pattern="":
    just bench "--bench intervalsets {{ pattern }}"

# check the ci targets locally
ci: fmt-check clippy typos doc test test-doc check-msrv check-no-std check-bench check-kani deny semver-checks
    @echo "CI checks complete"

# kani: symbolic-execution proof that the panic-free claims hold for all
# inputs within each harness's bounds. See core-panic-canary/README.md.
#
# `debug-assertions=off` so debug_asserts aren't treated as panics.
#
# Note: Kani requires `overflow-checks=on` for sound analysis, which is
# stricter than release-mode `+ - *` semantics (release wraps silently;
# Kani treats overflow as a panic). Arithmetic harnesses must therefore
# bound their inputs to avoid overflow, or accept that the proof covers
# "no panic AND no overflow" rather than just "no panic in release".
# Signed integer division overflow (e.g. `i64::MIN / -1`) panics under
# any setting — Rust always panics on `/` and `%` overflow.
#
# run kani symbolic-execution proofs (filter='' runs all; jobs>1 runs in parallel)
[env("RUSTFLAGS", "-C debug-assertions=off")]
kani filter="" jobs="1":
    cargo kani -p core-panic-canary {{ if jobs != "1" { "-j " + jobs + " --output-format terse" } else { "" } }} {{ if filter == "" { "" } else { "--harness " + filter } }}

# Catches Kani install / harness-wiring breakage without paying the full
# per-harness cost. STATUS.md is the source of truth for per-harness
# verification state — keep this list in sync. Pick the fastest variant
# per group; when a partial group becomes fully verified, add a
# representative below and drop it from SKIPPED.
#
# SKIPPED groups (partial / wip — see STATUS.md):
#   tier3_div  — 1/9 verified
#   tier3_mul  — 1/9 verified
#   tier3_hull — 6/10 verified
#
# CI smoke gate — one fast representative kani proof per verified group
[env("RUSTFLAGS", "-C debug-assertions=off")]
check-kani:
    cargo kani -p core-panic-canary --output-format terse \
        --harness contains_finite_i64_no_panic \
        --harness complement_half_i64_no_panic \
        --harness intersection_finite_finite_i64_no_panic \
        --harness union_finite_finite_i64_no_panic \
        --harness difference_half_half_i64_no_panic \
        --harness into_finite_finite_i64_no_panic \
        --harness into_elements_finite_i64_no_panic \
        --harness merge_connected_finite_finite_i64_no_panic \
        --harness try_add_finite_finite_i64_no_panic \
        --harness try_sub_finite_finite_i64_no_panic \
        --harness try_split_finite_i64_no_panic \
        --harness try_with_left_finite_i64_no_panic

# scan codebase for pre-release markers
loose-ends:
    rg --glob !justfile --ignore-case 'dbg!|fixme|todo|wip|xxx' .

# Common tasks for intervalsets dev.

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# the minimum supported rust version
MSRV := "1.87.0"

# the target rust version
RV := "stable"

# just run the tests
default: test

# install and/or update build tools and environment
update:
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
    cargo install static-web-server --locked

    # markdown book for higher level docs
    cargo install mdbook --locked

    # benchmarking
    cargo install cargo-criterion --locked

    # coverage
    cargo install cargo-llvm-cov --locked

    # debug macros
    cargo install cargo-expand --locked

    # check features
    cargo install cargo-hack --locked

    # check dependencies
    cargo install cargo-updeps --locked

    # checks commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
    cargo install commitlint-rs --locked

    # check codebase for loose ends
    cargo install ripgrep --locked

update-static-analysis-tools: #update-binstall
    cargo binstall kani-verifier --locked --no-confirm
    cargo kani setup

setup: update
    # building the test dependencies installs newest githooks (husky-rs)
    cargo clean && cargo test build

# build the docs
[env("RUSTDOCFLAGS", "-D warnings --cfg docsrs")]
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
    cargo +{{ RV }} test --all-features {{ pattern }}

alias t := test

book-serve:
    mdbook serve book

book-test:
    mdbook build book
    cargo +{{ RV }} test --package book --doc

# format the code base
fmt:
    cargo +{{ RV }} fmt

# check the build
check:
    cargo +{{ RV }} check --all-features

# run the test suite against the msrv
check-msrv:
    cargo +{{ MSRV }} test --all-features

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
ci: doc book-test test check-msrv check-no-std check-bench
    @echo "CI checks complete"

# canary: link-time verification that the panic-free claims hold at the
# instantiations exercised by each example. One example per tier so the
# Tier 1 ("any input") and Tier 2 ("invariant-respecting input") scopes
# stay distinct. Local-only — release builds are slow and the canary is
# opt-in. Add new tier examples to this target as they land.
[working-directory('core-panic-canary')]
panic-check:
    cargo +{{ RV }} build --release --bins

# kani: symbolic-execution proof that the panic-free claims hold for all
# inputs within each harness's bounds. Stronger than `panic-check` (it
# doesn't depend on optimizer cleverness), but per-harness and slow.
#
# `debug-assertions=off` matches the `#[no_panic]` cfg gate
# (`not(debug_assertions)`) so debug_asserts aren't treated as panics.
#
# Note: Kani requires `overflow-checks=on` for sound analysis, which is
# stricter than release-mode `+ - *` semantics (release wraps silently;
# Kani treats overflow as a panic). Arithmetic harnesses must therefore
# bound their inputs to avoid overflow, or accept that the proof covers
# "no panic AND no overflow" rather than just "no panic in release".
# Signed integer division overflow (e.g. `i64::MIN / -1`) panics under
# any setting — Rust always panics on `/` and `%` overflow.
[env("RUSTFLAGS", "-C debug-assertions=off")]
kani filter="" jobs="1":
    cargo kani -p core-panic-canary {{ if jobs != "1" { "-j " + jobs + " --output-format terse" } else { "" } }} {{ if filter == "" { "" } else { "--harness " + filter } }}

# scan codebase for pre-release markers
loose-ends:
    rg --glob !justfile --ignore-case 'dbg!|fixme|todo|wip|xxx' .

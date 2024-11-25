# Common tasks for intervalsets dev.

# the minimum supported rust version
MSRV := "1.81.0"

# the target rust version
RV := "nightly"

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
    rustup toolchain install {{MSRV}}

    # make sure that we have the nightly compiler
    rustup toolchain install nightly

    # watch the docs as you work
    cargo install static-web-server --locked

    # benchmarking
    cargo install cargo-criterion --locked

    # coverage
    cargo install cargo-llvm-cov --locked

    # debug macros
    cargo install cargo-expand --locked

    # check features
    cargo install cargo-hack --locked

    # checks commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
    cargo install commitlint-rs --locked

    # check codebase for loose ends
    cargo install ripgrep --locked

setup: update
    # building the test dependencies installs newest githooks (husky-rs)
    cargo clean && cargo test build

# build the docs
doc:
    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo +{{RV}} doc \
        --workspace \
        --all-features \
        --no-deps \
        --exclude benchmarks
        
alias d := doc

# launch a file server for docs
doc-serve port="8080": doc
    static-web-server --root target/doc --port {{port}} -z

# run the tests
test pattern="":
    cargo +{{RV}} test --all-features {{pattern}}

alias t := test

# format the code base
fmt:
    cargo +{{RV}} fmt

# check the build
check:
    cargo +{{RV}} check --all-features

# run the test suite against the msrv
check-msrv:
    cargo +{{MSRV}} test --all-features

# build against a no-std target
check-no-std:
    cargo +{{RV}} hack check --package intervalsets-core --each-feature \
        --exclude-features std,num-bigint,bigdecimal,arbitrary,quickcheck \
        --target thumbv6m-none-eabi \
        --verbose

# check the benchmarks
check-bench:
    just bench --no-run

# clean old build artifacts
clean:
    cargo clean

# run the micro benchmarks
bench pattern="":
    cargo +{{RV}} criterion --package benchmarks {{pattern}}

# run the core crate micro benchmarks
bench-core pattern="":
    just bench "--bench intervalsets_core {{pattern}}"

# run the main crate micro benchmarks
bench-main pattern="":
    just bench "--bench intervalsets {{pattern}}"

# check the ci targets locally
ci: doc test check-msrv check-no-std check-bench
    @echo "CI checks complete"

# scan codebase for pre-release markers
loose-ends:
    rg --glob !justfile --ignore-case 'dbg!|fixme|todo|wip|xxx' .
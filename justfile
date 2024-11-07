# cargo install just

MSRV := "1.81.0"

# just run the tests
default: test

# install and/or update build tools and environment
setup:
    @echo 'refreshing the dev environment'

    # update rustup
    rustup self update

    # update the default channel
    rustup update

    # make sure we have the MSRV toolchain installed
    rustup toolchain install {{MSRV}}

    # watch the docs as you work
    cargo install static-web-server --locked

    # benchmarking
    cargo install cargo-criterion --locked

    # coverage
    cargo install cargo-llvm-cov --locked

    # debug macros
    cargo install cargo-expand --locked

    # checks commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
    cargo install commitlint-rs --locked

    # building the test dependencies installs newest githooks (husky-rs)
    cargo clean && cargo test build

alias d := docs

# build the docs
docs:
    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --all-features --no-deps

# launch a file server for docs
serve-docs port="8080": docs
    static-web-server --root target/doc --port {{port}} -z

# run the test suite against the msrv
check-msrv:
    cargo +{{MSRV}} test --all-features

alias t := test

# run the tests
test pattern="":
    cargo test --all-features {{pattern}}

# check the build
check:
    cargo check --all-features

# clean old build artifacts
clean:
    cargo clean

# run the micro benchmarks
bench pattern="":
    cargo criterion {{pattern}}

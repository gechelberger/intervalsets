# intervalsets

[![CI][gh-image]][gh-checks]
[![intervalsets on docs.rs][docsrs-image]][docsrs]
[![codecov.io][codecov-img]][codecov-link]
[![intervalsets on crates.io][cratesio-image]][cratesio]
![Crates.io MSRV](https://img.shields.io/crates/msrv/intervalsets)

[gh-image]: https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg?branch=main
[gh-checks]: https://github.com/gechelberger/intervalsets/actions/workflows/test.yml?query=branch%3Amain
[docsrs-image]: https://docs.rs/intervalsets/badge.svg
[docsrs]: https://docs.rs/intervalsets
[cratesio-image]: https://img.shields.io/crates/v/intervalsets.svg
[cratesio]: https://crates.io/crates/intervalsets
[cratesio-msrv-image]: https://img.shields.io/crates/msrv/intervalsets
[codecov-img]: https://img.shields.io/codecov/c/github/gechelberger/intervalsets?logo=codecov
[codecov-link]: https://codecov.io/gh/gechelberger/intervalsets

This crate provides bounded and unbounded intervals 
implemented as sets with all the associated set operations.

See the [documentation](https://docs.rs/intervalsets/latest) for details.

## Features

* Generic intervals for all primitive types
    * Custom types may be supported by implementing the `Domain` trait
* Supports all boundary conditions (ie. empty, open, closed, unbound_open, etc...)
    * Integer types are always normalized to closed form.
    * Bounds trait provides simple accessors
* General set operations
    * union
        * merged is a special case of union; returns None if A and B are disjoint.
    * intersection
    * complement
    * difference
    * symmetric difference
* Set construction
    * factory functions for all boundary conditions
    * convex hull
        * from iterable of points
        * from iterable of other sets
* Set predicates
    * contains
    * intersects
* Set mappings
    * general user supplied functions
* IntervalSet (set of intervals)
    * disjoint ordered subsets
    * supports all set operations
* Simple conversion between types with From/Into traits.
* Convenient display
    * Interval<_>: {}, (10, 15], (<-, 2), etc...
    * IntervalSet<_>: {(10, 15], [20, ->)}

## development

### hooks

```sh
# the commit-msg git hook uses commitlint
cargo install commitlint-rs

# should install git hooks
cargo clean && cargo test
```

#### commit msgs

This project follows a subset of [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)
for changelog management with git-cliff. [.commitlintrc.yaml] defines the linting
rules.

```sh
# minor semver change, closes github issue #55
git commit -m "feat: [resolves #55] added new function struct::foo"

# major semver change, references github issue #67
# single quotes required because of the exclamation point.
git commit -m 'feat!: [issue #67] changed public api for Bar'

# patch semver change, closes github issue #33
git commit -m "fix: [resolves #33] fence post error in Baz"

# no semver change
git commit -m "chore: changed ci pipeline"
```
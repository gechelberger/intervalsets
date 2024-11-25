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

These crates provide generic bounded and unbounded intervals with associated set operations.

See the [intervalsets](https://docs.rs/intervalsets/latest) and 
[intervalsets-core](https://docs.rs/intervalsets-core/latest) documentation for details.

## Organization

The intervalsets-core crate encapsulates functionality for no-alloc environments.

The intervalsets crate builds upon the core functionality to support arbitrary
disjoint sets.

## Examples

todo: link to core/examples and intervalsets/examples

```rust
use intervalsets::prelude::*;

let reserved = Interval::closed_open(0, 100)
    .union(Interval::closed_open(200, 300))
    .union(Interval::closed_open(400, 500));

let requests: Vec<Interval<_>> = vec![
    [10, 20].into(),
    (150..160).into(),
    [200, 210].into(),
    (300, 400).into()
];

let (acceptable, rejected): (Vec<_>, Vec<_>) = requests.into_iter()
    .partition(|interval| !reserved.intersects(interval));

assert_eq!(acceptable, vec![
    Interval::closed_open(150, 160),
    Interval::open(300, 400),
]);

assert_eq!(rejected, vec![
    Interval::closed(10, 20),
    Interval::closed(200, 210),
])
```

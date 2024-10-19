# intervalsets

![CI](https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg)

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

### git hooks

```bash
cargo install cargo-hook
cargo hook
```

### outstanding
* integration tests
* benchmarks
* fuzz
* docs
# intervalsets

![CI](https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg)

This crate provides bounded and unbounded intervals 
implemented as sets with all the associated set operations.

## Features

* Generic intervals for all primitive types
    * [Custom types](custom-types) may be supported by implementing the `Numeric` trait
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
    * padding
    * shifting
    * general user supplied functions
* IntervalSet (set of intervals)
    * disjoint ordered subsets
    * supports all set operations

## Usage

```
let interval = Interval::closed(0.0, 100.0);
let hull = Interval::convex_hull([4.0, 220.0, 10.0, -44.0, 30.0, 99.0]);
assert!(hull.contains(&interval));
assert_eq!(hull.size().unwrap(), 264);
```

### Custom Types

The `num-traits` crate is used to generalize
support for types of interval boundaries but 
intervals need to be able to distinguish between
integer types and broader ones, so an implementation
of `Numeric` must be provided.

This is also important for concepts of Measure
if we ever get around to supporting those.

```
pub struct MyRationalNum { ... }

impl Numeric for MyRationalNum {

    /// MyRationalNum will not be normalized
    fn numeric_set() -> NumericSet {
        NumericSet::Real
    }

    fn try_finite_add(&self, rhs: &Self) -> Self {
        self.checked_add(self.clone(), rhs.clone())
    }

    fn try_finite_sub(&self, rhs: &Self) -> Self {
        self.checked_sub(self.clone(), rhs.clone())
    }
}
```

## development

### git hooks

```
cargo install cargo-hook
cargo hook
```

### outstanding
* unit test coverage
* benchmarks
* docs
* should interval bounds be CoW<'_, T>?
* more formal concepts of measure
    * lebesgue?
    * counting?
* contiguity between disjoint sets?
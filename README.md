# intervalsets

![CI](https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg)

This crate provides bounded and unbounded intervals 
implemented as sets with all the associated set operations.

## Features

* Generic intervals for all primitive types
    * [Custom types](#custom-types) may be supported by implementing the `Domain` trait
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
* Simple conversion between types with From/Into traits.
* Convenient display
    * Interval<_>: {}, (10, 15], (<-, 2), etc...
    * IntervalSet<_>: {(10, 15], [20, ->)}

## Usage

```rust
let interval = Interval::closed(0.0, 100.0);
let hull = Interval::convex_hull([4.0, 220.0, 10.0, -44.0, 30.0, 99.0]);
assert!(hull.contains(&interval));
assert_eq!(hull.size().unwrap(), 264);
```

## [Custom Types]

A few external library types are currently supported:
* rust_decimal
* num-bigint
* chrono

### Discrete types

For discrete data types (like integers) we prefer to
normalize to closed form. 

ie. [1, 2] is preferred over (0, 3)

We do this by implementing the [`Domain`] trait for any type
we wish to use in our Intervals/Sets.

The `try_adjacent` impl should return the next greater or 
smaller value. This must agree with the PartialOrd impl for
the type and it must be invertable.

```rust
let x = MyBigint::from(0);
let y = x.try_adjacent(Side::Right).unwrap();
let z = y.try_adjacent(Side::Left).unwrap();
assert_eq!(x, z);
```

#### Example

```rust
use intervalsets::{Side, Domain};

pub struct MyBigInt { ... }

impl Domain for MyBigInt {
    /// This type will be normalized to closed form,
    /// or left in open form when try_adjacent would
    /// overflow.
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        // quantum does not have to be 1 for all types, 
        // but for the type implementing `Domain`, there 
        // should be no other value(s) possible between 
        // the current `self` and our calculated adjacent.
        let quantum = MyBigint::ONE;
        match side {
            Side::Left => self.checked_sub(&quantum),
            Side::Right => self.checked_add(&quantum),
        }
    }
}
```

### Continuous(ish) types

For more continuous types such as floats, the open/closed
representation serves perfectly well. To use a custom type 
simply return None and no normalization will take place.

```rust
impl Domain for MyContinuousType {
    fn try_adjacent(&self, side: Side) -> Option<Self> {
        None
    }
}
```

For simplicity a macro exists that does just this.

```rust
intervalsets::continuous_domain_impl!(MyContinuousType);
```

## development

### git hooks

```bash
cargo install cargo-hook
cargo hook
```

### fuzzing

```bash
cargo install cargo-fuzz
```

### outstanding
* better unit test coverage
* integration tests
* benchmarks
* docs
* contiguity between disjoint sets?
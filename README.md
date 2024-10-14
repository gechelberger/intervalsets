# intervalsets

![CI](https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg)

This crate provides bounded and unbounded intervals 
implemented as sets with all the associated set operations.

## Features

## Examples


## development

### git hooks

```
cargo install cargo-hook
cargo hook
```


# TODO:
* unit test coverage
* public interface docstrings
* should interval bounds be CoW<'_, T>?


# possible features
* more formal concepts of measure
    * lebesgue
    * counting
* contiguity between disjoint sets?


### Supported Types

All primitive numeric types are supported by default.

#### Custom Types

The `num-traits` crate is used to generalize
support for types of interval boundaries but 
intervals need to be able to distinguish between
integer types and broader ones, so an implementation
of `Numeric` must be provided.

This is also important for concepts of Measure
if we ever get around to supporting those.

```
pub struct MyRationalNum {}

impl Numeric for MyRationalNum {

    /// MyRationalNum will not be normalized
    fn numeric_set() -> NumericSet {
        NumericSet::Real
    }
}

pub struct MyUnsignedBigInt {}

impl Numeric for MyUnsignedBigInt {
    
    /// MyUnsignedBigInt will be normalized
    fn numeric_set() -> NumericSet {
        NumericSet::Natural
    }
}
```
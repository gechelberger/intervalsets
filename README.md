# intervalsets

![CI](https://github.com/gechelberger/intervalsets/actions/workflows/rust.yml/badge.svg)

This crate provides intervals as sets.

## development

### git hooks

```
cargo install cargo-hook
cargo hook
```


# TODO:
* interval sets
* Decide Send + Sync
* thiserror error handling?
* get rid of Copy trait bound for T on Interval?
* unit test coverage
* continuous integration with github
* github badges
* public interface docstrings
* normalization of intervals for integer types
* Make the Interval<T> enum accept a Cow<>?


# possible features
* random generator over defined interval?
* optional library features 
    * define Normalize BigInt? Decimal?
* more formal concepts of measure
    * lebesgue
    * counting
* contiguity between disjoint sets?

# traits
* Difference
* SymmetricDifference
* AdjacentTo? Connects? 
    * [0, 1] (1, 2)


### Supported Types

All primitive numeric types are supported.

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

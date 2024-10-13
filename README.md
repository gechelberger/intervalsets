
# TODO:
* interval sets
* thiserror error handling?
* get rid of Copy trait bound for T on Interval
* unit test coverage
* continuous integration with github
* github badges
* public interface docstrings
* normalization of intervals for integer types


# possible features
* random generator over defined interval?
* optional library features 
    * define Normalize BigInt? Decimal?
* more formal concepts of measure
    * lebesgue?
    * counting

# traits
* Normalize
* Contains?
* SetOperations?
* Difference?
* AdjacentTo? Connects?

### naming?
* Bound => BoundCond?
* IVal => Bound?

### notes
f32:NAN and f64::NAN break things... not sure how best to deal with that.

### Supported Types

All primitive numeric types are supported.

#### Custom Types

The `num-traits` crate is used to generalize
support for types of interval boundaries but 
intervals need to be able to distinguish between
integer types and broader ones, so an implementation
of `Numeric` must be provided.

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

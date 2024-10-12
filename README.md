
# TODO:
* interval sets
* thiserror error handling
* get rid of Copy trait bound for T on Interval
* unit test coverage
* continuous integration with github
* github badges
* public interface docstrings
* normalization of intervals for integer types

# possible features
* random generator over defined interval?
* optional library features 
    * define Normalize for other BigDecimal libraries?

# traits
* Normalize
* Contains?
* SetOperations?
* Difference?

### naming?
* Bound => BoundCond?
* IVal => Bound?

### notes
f32:NAN and f64::NAN break things... not sure how best to deal with that.

### Supported Types

All primitive numeric types are supported.
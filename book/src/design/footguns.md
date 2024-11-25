# Footguns

## Normalized Conversions

Most of the time normalization is transparent to the user, but it is a
potential source of error, particularly when converting types that have
implicit open bounds.

```rust
use intervalsets_core::prelude::*;

let discrete = EnumInterval::open(0, 10);
assert_eq!(discrete.lval(), Some(&1));
assert_eq!(discrete.rval(), Some(&9));
assert_eq!(discrete, (0, 10).into());
assert_eq!(discrete, [1, 9].into());
```

## Floating Point Types

Making `Ord` a trait bound for most APIs would eliminate a whole class of errors 
(TotalOrderError), but floats come with a whole host of complexities regardless.

 * `NAN` is not part of the default ordering, though there is a `total_cmp`
    available now.
 * rounding error can cause issues with testing values near a finite bound.
 * `FiniteBound(f32::INFINITY)` and `FiniteBound(f32::NEG_INFINITY)`are both
    valid syntax, though all manner of headache inducing semantically speaking.

Sometimes, floats are still the right tool for the job, and it is left to the
user to choose the right approach for the given problem. Fixed precision
decimal types like `rust_decimal` do side step some pitfalls.

## Fallibility

```rust
use intervalsets_core::prelude::*;

let x = FiniteInterval::open(1.0, 0.0);
assert_eq!(x, FiniteInterval::empty());

// total order error -> panic
// infallible for properly implemented [`Ord`] types.
let result = std::panic::catch_unwind(|| {
    FiniteInterval::open(f32::NAN, 0.0);
});
assert!(result.is_err());

let x = FiniteInterval::strict_open(f32::NAN, 0.0);
assert_eq!(x, None);

let x = FiniteInterval::strict_open(1.0, 0.0);
assert_eq!(x, FiniteInterval::empty());
```

Silent failures can make it difficult to isolate logic errors as they are
able to propogate further from their source before detection.

```rust
use intervalsets_core::prelude::*;
let interval = EnumInterval::closed(0, 10);

let oops = interval
    .with_left_closed(20) // empty here
    .with_right(None);
assert_ne!(oops, EnumInterval::closed_unbound(20));
assert_eq!(oops, EnumInterval::empty());

let fixed = interval.with_right(None).with_left_closed(20);
assert_eq!(fixed, EnumInterval::closed_unbound(20));
```
# intervalsets-macros

Compile-time-checked interval literal macros for
[`intervalsets`](https://docs.rs/intervalsets) and
[`intervalsets-core`](https://docs.rs/intervalsets-core).

This crate is a thin proc-macro support crate. You typically don't
depend on it directly — the macros are re-exported by their parent
crates:

- `intervalsets::interval!("[0, 10]")` produces an `Interval<T>`.
- `intervalsets::set!("{[0, 5] U [10, 15]}")` produces an `IntervalSet<T>`.
- `intervalsets_core::enum_interval!("[0, 10]")` produces an
  `EnumInterval<T>` (no-std / no-alloc friendly).

All accept the same grammar as the corresponding runtime
[`FromStr`](https://docs.rs/intervalsets-core/latest/intervalsets_core/sets/enum.EnumInterval.html#impl-FromStr-for-EnumInterval%3CT%3E)
impl. Malformed input fails to build instead of panicking at runtime.

```rust
use intervalsets::prelude::*;

let x: Interval<i32> = interval!("[0, 10)");
assert_eq!(x, Interval::closed_open(0, 10));

// Optional storage-type hint (turbofish):
let y = interval!("(.., ..)", i32);
assert_eq!(y, Interval::<i32>::unbounded());

// Multi-piece set:
let s = set!("{[0, 5] U [10, 15] U [20, 30]}", i32);
```

See each macro's documentation for the full grammar table.

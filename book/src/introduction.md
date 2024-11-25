intervalsets
============

`intervalsets` is a family of crates for working with intervals and numeric sets.

* [`intervalsets-core`](https://crates.io/crates/intervalsets-core) -- no-std, no-alloc functionality.
* [`intervalsets`](https://crates.io/crates/intervalsets) -- extended functionality requiring allocations.

This booklet is intended to cover information applicable to the family of crates. 
For specifics see the [`intervalsets-core`]() or [`intervalsets`]() documentation.

Limitations
-----------

This family of crates is intended to provide robust, general implementations
of intervals with a convenient `Set` based api, and support pluggable
user provided data-types. While attention has been given to performance,
there are many optimizations that can not be included without a loss of generality.

Currently [interval arithmetic](https://en.wikipedia.org/wiki/Interval_arithmetic)
is not supported, and while it may be in the future, it will never be as
performant as a specialized library like [inari](https://docs.rs/inari/latest/inari/).

Contributing
------------

[Contributions](https://github.com/gechelberger/intervalsets/blob/main/CONTRIB.md) are welcome.

License
-------

`intervalsets` is released under the [`MIT license`](https://mit-license.org/).
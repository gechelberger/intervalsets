# Goals

`intervalsets` intends to provide flexible, reliable tools
for working with intervals and numeric sets. 

## Correctness

None of the other goals matter if the results can't be trusted.

There is an extensive test suite to ensure that operations produce the intended results.

## Generality

All set and interval types provided are generic over the type of element(s) in the set.

## Portability

These are low level abstractions which should be deployable in almost any environment.

`intervalsets-core`, by default, should be usable in any embedded environment, with or 
without an allocator. The crate does provide some optional features for externally defined 
`set element types` that require allocation. These must live in `intervalsets-core` due to rust`s 
[orphan rule](https://github.com/Ixrec/rust-orphan-rules) since the required traits 
are defined in `intervalsets-core`.

`intervalsets` should be usable in a no-std environment but does require an allocator to
support collections of intervals.

## Robustness

Fault tolerance is critical, especially in embedded environments.

todo: (design/fallability.md)

## Performance

todo:

## Ease of use

todo: 
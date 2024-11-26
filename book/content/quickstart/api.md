# api

## types

| Case          | Core Implementation | Main Implementation |
|---------------|:--------------------|:--------------------|
| Empty Set     | FiniteInterval      | Interval            |
| Fully Bounded | FiniteInterval      | Interval            |
| Left Bounded  | HalfInterval        | Interval            |
| Right Bounded | HalfInterval        | Interval            |
| Unbounded     | EnumInterval        | Interval            |
| Disjoint      | &mdash;             | IntervalSet         |

<!-- todo: | Overlapping | None | [`IntervalTree`] | -->

## operations

Most operations are provided via traits in order to present a consistent,
easy to use API for each type.

| Operation         | Op Type   | Core    | Main    | Description |
|-------------------|-----------|:-------:|:-------:|-------------|
| Contains      | predicate | &check; | &check; | Test if A is a superset of B                     |
| Intersects    | predicate | &check; | &check; | Test for some shared element of A and B          |
| Adjacent      | predicate | &check; | &check; | Test if set bounds are connected                 |
| Width         | measure   | &check; | &check; | Find the width/diameter/length of a set          |
| Count         | measure   | &check; | &check; | Count the elements of a set                      |
| Intersection  | binary    | &check; | &check; | The intersection set of two sets                 |
| TryMerge      | binary    | &check; | &check; | The union of two connected sets                  |
| Split         | function  | &check; | &check; | Two sets partitioned by some element             |
| IntoFinite    | unary     | &check; | &check; | Convert to a finite interval limited by `element type`   |
| Complement    | unary     | &mdash; | &check; | The (possibly disjoint) complement of a set              |
| Union         | binary    | &mdash; | &check; | The (possibly disjoint) union of two sets                |
| Difference    | binary    | &mdash; | &check; | The (possibly disjoint) difference of set A from B       |
| SymDifference | binary    | &mdash; | &check; | The (possibly disjoint) symmetric difference of two sets |

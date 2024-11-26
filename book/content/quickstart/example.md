# Examples

## Filter overlapping intervals

```rust
use intervalsets::prelude::*;

let reserved = Interval::closed_open(0, 100)
    .union(Interval::closed_open(200, 300))
    .union(Interval::closed_open(400, 500));

let requests: Vec<Interval<_>> = vec![
    [10, 20].into(),
    (150..160).into(),
    [200, 210].into(),
    (300, 400).into()
];

let (acceptable, rejected): (Vec<_>, Vec<_>) = requests.into_iter()
    .partition(|interval| !reserved.intersects(interval));

assert_eq!(acceptable, vec![
    Interval::closed_open(150, 160),
    Interval::open(300, 400),
]);

assert_eq!(rejected, vec![
    Interval::closed(10, 20),
    Interval::closed(200, 210),
])
```
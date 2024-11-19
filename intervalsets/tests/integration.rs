/*

use chrono::{DateTime, TimeDelta, Utc};
use intervalsets::{continuous_domain_impl, Interval};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct MyDT(DateTime<Utc>);

continuous_domain_impl!(MyDT);

#[test]
fn test_foo() {
    let a = Utc::now();
    let delta = TimeDelta::new(1000, 0).unwrap();
    let b = a + delta;

    let i = Interval::closed(MyDT(a), MyDT(b));
}
 */

use intervalsets::prelude::*;

#[test]
fn from_readme() {
    let reserved = Interval::closed_open(0, 100)
        .union(Interval::closed_open(200, 300))
        .union(Interval::closed_open(400, 500));

    let requests: Vec<Interval<_>> = vec![
        [10, 20].into(),
        (150..160).into(),
        [200, 210].into(),
        (300, 400).into(),
    ];

    let (acceptable, rejected): (Vec<_>, Vec<_>) = requests
        .into_iter()
        .partition(|interval| !reserved.intersects(interval));

    assert_eq!(
        acceptable,
        vec![Interval::closed_open(150, 160), Interval::open(300, 400),]
    );

    assert_eq!(
        rejected,
        vec![Interval::closed(10, 20), Interval::closed(200, 210),]
    )
}

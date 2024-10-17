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
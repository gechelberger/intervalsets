use intervalsets::contains::Contains;
use intervalsets::Interval;

#[test]
fn itest_contains() {
    let interval = Interval::open_closed(0, 10);
    assert!(!interval.contains(&0));
    assert!(interval.contains(&5));
    assert!(interval.contains(&10));
    assert!(!interval.contains(&11));
}

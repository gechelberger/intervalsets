#![no_main]

use libfuzzer_sys::fuzz_target;

#[derive(Debug, Clone, arbitrary::Arbitrary)]
struct Data {
    f32_left: f32,
    f32_right: f32,
    i64_left: i64,
    i64_right: i64,
}

use intervalsets::Interval;
use intervalsets::IntervalSet;
use intervalsets::Complement;

fuzz_target!(|data: Data| {
    if !f32::is_nan(data.f32_left) && !f32::is_nan(data.f32_right) {
        let set: IntervalSet<_> = Interval::closed_open(data.f32_left, data.f32_right).into();
        assert_eq!(set.complement().complement(), set);
    }

    let set: IntervalSet<_> = Interval::closed(data.i64_left, data.i64_right).into();
    assert_eq!(set.complement().complement(), set);

});

//! Counting billable quarter-hours across a fragmented work day.
//!
//! Quarter-hour timekeeping is the canonical "fractional discrete
//! units you actually sum" domain: legal, consulting, and creative
//! invoices quote work in `0.25h` ticks, and the very question a
//! timesheet answers is "how many of those ticks fit across this set
//! of work sessions?" — i.e. `Measure::measure` (which on a discrete
//! type returns cardinality). Total hours billed is
//! `measure() * 0.25`, the literal sum of the per-tick values.
//!
//! `FixedU32<U2>` makes the grid exact: every value is a multiple of
//! `0.25h`, arithmetic stays on the grid, and `Measure::measure` over
//! a `MaybeDisjoint` *sums* the per-session counts. That's the
//! operation a timesheet actually does — fragmented hours around
//! lunch, meetings, and after-hours work all roll up.
//!
//! Floats can't: `0.1 + 0.2 != 0.3`, and the spacing between
//! representable values isn't uniform, so summing tick counts across
//! pieces wouldn't be well-defined.

use fixed::types::extra::U2;
use fixed::FixedU32;
use intervalsets_core::measure::Measure;
use intervalsets_core::ops::Union;
use intervalsets_core::prelude::*;

// Hours-of-day at 0.25h (15-minute) precision. `fixed` doesn't ship
// a named alias for `<U32, U2>`, so name it here.
type Hour = FixedU32<U2>;

fn h(hours: f64) -> Hour {
    Hour::from_num(hours)
}

fn main() {
    // Two billable sessions in today's work day. Half-open on the
    // right so `[9.00, 12.00)` is exactly 12 ticks (slots starting
    // at 9:00, 9:15, ..., 11:45) without an off-by-one on the noon
    // mark — the library normalizes the open right bound by one ULP.
    let morning = FiniteInterval::closed_open(h(9.00), h(12.00));
    let afternoon = FiniteInterval::closed_open(h(13.00), h(18.00));

    let workday = morning.union(afternoon);

    // measure() over the disjoint set SUMS the per-piece tick counts.
    //   morning:   (12.00 - 9.00) / 0.25 = 12 ticks (3.00h)
    //   afternoon: (18.00 - 13.00) / 0.25 = 20 ticks (5.00h)
    let ticks = workday.measure().finite();
    assert_eq!(ticks, 12 + 20);
    println!("{ticks} ticks today = {:.2}h billable", ticks as f64 * 0.25);

    // One client engagement — sub-range of the afternoon block.
    let client_call = FiniteInterval::closed_open(h(14.00), h(15.50));
    let cc_ticks = client_call.measure().finite();
    assert_eq!(cc_ticks, 6);
    println!(
        "client call: {cc_ticks} ticks = {:.2}h",
        cc_ticks as f64 * 0.25
    );
}

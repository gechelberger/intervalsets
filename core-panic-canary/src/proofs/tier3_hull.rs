//! Phase 2 — `ConvexHull::try_hull` for representative impls at i64.
//!
//! Iterator-consuming impl. Each harness uses a fixed-size array
//! (3 elements) rather than a Vec to keep the loop bound static and
//! within the workspace `default-unwind = 4`. Each element is
//! independently nondeterministic, so the proof covers all
//! min/max orderings on a 3-element input.
//!
//! Coverage: 4 macro impls (FiniteInterval/EnumInterval × T/&T) +
//! the FiniteInterval-item hand impls. EnumInterval-item hand impls
//! are also covered, but with a smaller-search-space [EI; 2] array.

use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::ConvexHull;
use intervalsets_core::sets::{EnumInterval, FiniteInterval};

fn any_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::closed(kani::any(), kani::any())
}

fn any_enum() -> EnumInterval<i64> {
    let kind: u8 = kani::any();
    kani::assume(kind < 5);
    match kind {
        0 => EnumInterval::<i64>::closed(kani::any(), kani::any()),
        1 => EnumInterval::<i64>::closed_unbound(kani::any()),
        2 => EnumInterval::<i64>::unbound_closed(kani::any()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

#[kani::proof]
fn try_hull_finite_t_array3_no_panic() {
    let elems: [i64; 3] = [kani::any(), kani::any(), kani::any()];
    let _ = <FiniteInterval<i64> as ConvexHull<i64>>::try_hull(elems);
}

#[kani::proof]
fn try_hull_finite_ref_t_array3_no_panic() {
    let elems: [i64; 3] = [kani::any(), kani::any(), kani::any()];
    let _ = <FiniteInterval<i64> as ConvexHull<&i64>>::try_hull(elems.iter());
}

#[kani::proof]
fn try_hull_enum_t_array3_no_panic() {
    let elems: [i64; 3] = [kani::any(), kani::any(), kani::any()];
    let _ = <EnumInterval<i64> as ConvexHull<i64>>::try_hull(elems);
}

#[kani::proof]
fn try_hull_enum_ref_t_array3_no_panic() {
    let elems: [i64; 3] = [kani::any(), kani::any(), kani::any()];
    let _ = <EnumInterval<i64> as ConvexHull<&i64>>::try_hull(elems.iter());
}

#[kani::proof]
fn try_hull_finite_finite_array3_no_panic() {
    let elems = [any_finite(), any_finite(), any_finite()];
    let _ = <FiniteInterval<i64> as ConvexHull<FiniteInterval<i64>>>::try_hull(elems);
}

#[kani::proof]
fn try_hull_finite_ref_finite_array3_no_panic() {
    let elems = [any_finite(), any_finite(), any_finite()];
    let _ = <FiniteInterval<i64> as ConvexHull<&FiniteInterval<i64>>>::try_hull(elems.iter());
}

#[kani::proof]
fn try_hull_enum_finite_array3_no_panic() {
    let elems = [any_finite(), any_finite(), any_finite()];
    let _ = <EnumInterval<i64> as ConvexHull<FiniteInterval<i64>>>::try_hull(elems);
}

#[kani::proof]
fn try_hull_enum_ref_finite_array3_no_panic() {
    let elems = [any_finite(), any_finite(), any_finite()];
    let _ = <EnumInterval<i64> as ConvexHull<&FiniteInterval<i64>>>::try_hull(elems.iter());
}

#[kani::proof]
fn try_hull_enum_enum_array2_no_panic() {
    let elems = [any_enum(), any_enum()];
    let _ = <EnumInterval<i64> as ConvexHull<EnumInterval<i64>>>::try_hull(elems);
}

#[kani::proof]
fn try_hull_enum_ref_enum_array2_no_panic() {
    let elems = [any_enum(), any_enum()];
    let _ = <EnumInterval<i64> as ConvexHull<&EnumInterval<i64>>>::try_hull(elems.iter());
}

//! Phase 3 — `IntoElementIterator::into_elements` for all 4 impls at
//! i64.
//!
//! Proves that *constructing* the iterator does not panic. Iterating
//! the resulting `Elements<i64>` / `DisjointElements<i64>` is out of
//! scope (the iterator can be enormous and would explode unwind
//! bounds).

use intervalsets_core::bound::FiniteBound;
use intervalsets_core::disjoint::MaybeDisjoint;
use intervalsets_core::factory::traits::*;
use intervalsets_core::ops::{Complement, IntoElementIterator};
use intervalsets_core::sets::{EnumInterval, FiniteInterval, HalfInterval};

fn make_finite() -> FiniteInterval<i64> {
    FiniteInterval::<i64>::satisfy_bounds(
        FiniteBound::closed(kani::any::<i64>()),
        FiniteBound::closed(kani::any::<i64>()),
    )
}

fn make_half() -> HalfInterval<i64> {
    let at: i64 = kani::any();
    if kani::any() {
        HalfInterval::<i64>::closed_unbound(at)
    } else {
        HalfInterval::<i64>::unbound_closed(at)
    }
}

fn make_enum() -> EnumInterval<i64> {
    let kind: u8 = kani::any();
    kani::assume(kind < 5);
    match kind {
        0 => EnumInterval::<i64>::satisfy_bounds(
            FiniteBound::closed(kani::any::<i64>()),
            FiniteBound::closed(kani::any::<i64>()),
        ),
        1 => EnumInterval::<i64>::closed_unbound(kani::any()),
        2 => EnumInterval::<i64>::unbound_closed(kani::any()),
        3 => EnumInterval::<i64>::unbounded(),
        _ => EnumInterval::<i64>::empty(),
    }
}

// Take the complement of a FiniteInterval to drive a nondet
// MaybeDisjoint covering all 3 variants (Consumed / Connected /
// Disjoint), without needing to construct one by hand.
fn make_maybe_disjoint() -> MaybeDisjoint<i64> {
    make_finite().complement()
}

#[kani::proof]
fn into_elements_finite_i64_no_panic() {
    let _ = make_finite().into_elements();
}

#[kani::proof]
fn into_elements_half_i64_no_panic() {
    let _ = make_half().into_elements();
}

#[kani::proof]
fn into_elements_enum_i64_no_panic() {
    let _ = make_enum().into_elements();
}

#[kani::proof]
fn into_elements_maybe_disjoint_i64_no_panic() {
    let _ = make_maybe_disjoint().into_elements();
}

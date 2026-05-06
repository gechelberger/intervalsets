//! Linker-time canary for the Tier 2 panic-free claims
//! (`Complement`, `Intersection`, `Union`, `Difference`, `IntoFinite`,
//! `IntoElementIterator`, `MergeConnected`).
//!
//! Tier 2 promises "cannot panic given inputs satisfying their type
//! invariants." Every fixture below is built through the validating
//! constructors so the precondition holds; an invariant-violating
//! input is out of scope and would belong to Tier 4.
//!
//! Sanity-build (no canary; verifies the example compiles):
//!     cargo build --example panic_free_tier2 --release
//!
//! Actual canary (link fails if any annotated impl can panic):
//!     cargo build --example panic_free_tier2 --features panic-free-check --release
//!
//! Every annotated Tier 2 impl must be invoked below — an impl that
//! is never called is dead-code-eliminated, the `no_panic` symbol
//! never resolves, and a function that would have failed the check
//! passes silently.

use core::hint::black_box;

use intervalsets_core::disjoint::MaybeDisjoint;
use intervalsets_core::prelude::*;

fn main() {
    // Fixtures at T = i64. FiniteInterval/HalfInterval/EnumInterval are
    // Copy when T is Copy, so we reuse freely; MaybeDisjoint is only
    // Clone, so we rebuild it as needed.
    let f = FiniteInterval::<i64>::closed(0, 10);
    let f2 = FiniteInterval::<i64>::closed(20, 30);
    let h_l = HalfInterval::<i64>::closed_unbound(5);
    let h_r = HalfInterval::<i64>::unbound_closed(15);
    let e = EnumInterval::<i64>::closed(0, 10);
    let e2 = EnumInterval::<i64>::closed_unbound(5);

    // Complement: 3 owned (FiniteInterval, HalfInterval, EnumInterval)
    // + by-ref blanket monomorphized at each.
    black_box(f.complement());
    black_box(h_l.complement());
    black_box(e.complement());
    black_box((&f).complement());
    black_box((&h_l).complement());
    black_box((&e).complement());

    // Intersection: 6 hand-written + 6 dispatch (EnumInterval × …) +
    // 6 commutative (X × EnumInterval / FiniteInterval × HalfInterval-flipped).
    // Both owned and by-ref forms.
    black_box(f.intersection(f2));
    black_box((&f).intersection(&f2));
    black_box(f.intersection(h_l));
    black_box((&f).intersection(&h_l));
    black_box(h_l.intersection(h_r));
    black_box((&h_l).intersection(&h_r));
    black_box(e.intersection(e2));
    black_box((&e).intersection(&e2));
    black_box(e.intersection(h_l));
    black_box((&e).intersection(&h_l));
    black_box(e.intersection(f));
    black_box((&e).intersection(&f));
    black_box(h_l.intersection(f));
    black_box((&h_l).intersection(&f));
    black_box(f.intersection(e));
    black_box((&f).intersection(&e));
    black_box(h_l.intersection(e));
    black_box((&h_l).intersection(&e));

    // Union: same 9 type-pair instantiations × {owned, ref}.
    black_box(f.union(f2));
    black_box((&f).union(&f2));
    black_box(h_l.union(h_r));
    black_box((&h_l).union(&h_r));
    black_box(f.union(h_l));
    black_box((&f).union(&h_l));
    black_box(h_l.union(f));
    black_box((&h_l).union(&f));
    black_box(e.union(f));
    black_box((&e).union(&f));
    black_box(e.union(h_l));
    black_box((&e).union(&h_l));
    black_box(e.union(e2));
    black_box((&e).union(&e2));
    black_box(f.union(e));
    black_box((&f).union(&e));
    black_box(h_l.union(e));
    black_box((&h_l).union(&e));

    // Difference: 9 owned via difference_via_complement + by-ref blanket
    // monomorphized at each pair. Difference is not commutative, so each
    // ordering is its own impl.
    black_box(f.difference(f2));
    black_box((&f).difference(&f2));
    black_box(h_l.difference(h_r));
    black_box((&h_l).difference(&h_r));
    black_box(f.difference(h_l));
    black_box((&f).difference(&h_l));
    black_box(h_l.difference(f));
    black_box((&h_l).difference(&f));
    black_box(e.difference(f));
    black_box((&e).difference(&f));
    black_box(e.difference(h_l));
    black_box((&e).difference(&h_l));
    black_box(e.difference(e2));
    black_box((&e).difference(&e2));
    black_box(f.difference(e));
    black_box((&f).difference(&e));
    black_box(h_l.difference(e));
    black_box((&h_l).difference(&e));

    // IntoFinite: 3 owned impls. (HalfInterval/EnumInterval need
    // num_traits::Bounded + PartialOrd; i64 satisfies both.)
    black_box(f.into_finite());
    black_box(h_l.into_finite());
    black_box(e.into_finite());

    // IntoElementIterator: 4 owned `into_elements` + 4 inherent
    // `.elements()` borrowed constructors. Forward + reverse iteration
    // exercises both `next` and `next_back` on Elements / DisjointElements.
    let connected: MaybeDisjoint<i64> = MaybeDisjoint::Connected(EnumInterval::closed(0, 5));
    let disjoint: MaybeDisjoint<i64> =
        (EnumInterval::closed(0, 2), EnumInterval::closed(10, 12)).into();

    black_box(f.into_elements().count());
    black_box(f.into_elements().rev().count());
    black_box(h_l.into_elements().count());
    black_box(h_l.into_elements().rev().count());
    black_box(e.into_elements().count());
    black_box(e.into_elements().rev().count());
    black_box(connected.clone().into_elements().count());
    black_box(disjoint.clone().into_elements().rev().count());

    black_box(f.elements().count());
    black_box(f.elements().rev().count());
    black_box(h_l.elements().count());
    black_box(h_l.elements().rev().count());
    black_box(e.elements().count());
    black_box(e.elements().rev().count());
    black_box(connected.elements().count());
    black_box(disjoint.elements().rev().count());

    // MergeConnected: 6 hand (FiniteInterval × Self, HalfInterval × Self,
    // HalfInterval × FiniteInterval) + 6 dispatch (EnumInterval × …) +
    // 6 commutative. Owned and by-ref each.
    black_box(f.merge_connected(f2));
    black_box((&f).merge_connected(&f2));
    black_box(h_l.merge_connected(h_r));
    black_box((&h_l).merge_connected(&h_r));
    black_box(h_l.merge_connected(f));
    black_box((&h_l).merge_connected(&f));
    black_box(e.merge_connected(f));
    black_box((&e).merge_connected(&f));
    black_box(e.merge_connected(h_l));
    black_box((&e).merge_connected(&h_l));
    black_box(e.merge_connected(e2));
    black_box((&e).merge_connected(&e2));
    black_box(f.merge_connected(h_l));
    black_box((&f).merge_connected(&h_l));
    black_box(f.merge_connected(e));
    black_box((&f).merge_connected(&e));
    black_box(h_l.merge_connected(e));
    black_box((&h_l).merge_connected(&e));
}

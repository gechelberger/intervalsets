//! Arithmetic operators (Add / Sub / Mul / Div) and their panic-free Try*
//! siblings over Interval and IntervalSet.
//!
//! # Construction discipline for IntervalSet results
//!
//! Every IntervalSet result path here builds via **union-fold** -- accumulating
//! into an IntervalSet by repeated `Union` calls -- rather than collecting into
//! a Vec and passing it through `IntervalSet::new`.
//!
//! `IntervalSet::new` is the defensive public constructor: it accepts
//! arbitrary user input and pays for filtering empties, checking invariants,
//! sorting, and merging connected intervals. Pointless work when our inputs
//! are themselves valid Intervals / IntervalSets -- Self's invariants already
//! guarantee non-empty, sorted, disjoint subsets, and Try* operations applied
//! to valid subsets produce valid results.
//!
//! Worse, `IntervalSet::new`'s sort uses `partial_cmp().expect(...)`, which
//! panics on NaN-tainted bounds. That defeats the panic-free contract of the
//! Try* impls -- a NaN should surface as `Err(TotalOrderError)`, not a
//! panic from a deeply nested validation call.
//!
//! `Union`, by contrast, is the right operation both semantically (we're
//! combining interval sets) and structurally: its body goes straight through
//! `IntervalSet::new_assume_valid(MergeSortedByValue::new(...))` over
//! already-sorted inputs. No re-validation, no panic path.

mod add;
mod div;
mod mul;
mod sub;

pub use intervalsets_core::ops::math::{TryAdd, TryDiv, TryMul, TrySub};

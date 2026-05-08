# Kani harness verification status

Tracks which harnesses in `src/proofs/` have been confirmed locally
with `just kani`. Times next to each harness are wall-clock from the
most recent successful run on this machine. Times may swing ~10–30%
under parallel jobs vs. sequential.

## Tier 1

- [x] `tier1::contains_finite_i64_no_panic` — <1s

## Tier 2 — `Complement` (`tier2_complement.rs`)

- [x] `complement_finite_i64_no_panic` — 1.2s
- [x] `complement_half_i64_no_panic` — 1.1s
- [x] `complement_enum_i64_no_panic` — 3.0s

## Tier 2 — `Intersection` (`tier2_intersection.rs`)

- [x] `intersection_finite_finite_i64_no_panic` — 1.4s
- [x] `intersection_finite_half_i64_no_panic` — 2.4s
- [x] `intersection_half_finite_i64_no_panic` — 2.4s
- [x] `intersection_half_half_i64_no_panic` — 2.0s
- [x] `intersection_finite_enum_i64_no_panic` — 4.3s
- [x] `intersection_enum_finite_i64_no_panic` — 3.5s
- [x] `intersection_half_enum_i64_no_panic` — 4.3s
- [x] `intersection_enum_half_i64_no_panic` — 4.4s
- [x] `intersection_enum_enum_i64_no_panic` — 9.2s

## Tier 2 — `Union` (`tier2_union.rs`)

- [x] `union_finite_finite_i64_no_panic` — 4.9s
- [x] `union_finite_half_i64_no_panic` — 6.9s
- [x] `union_half_finite_i64_no_panic` — 5.8s
- [x] `union_half_half_i64_no_panic` — 5.3s
- [x] `union_finite_enum_i64_no_panic` — 14.2s
- [x] `union_enum_finite_i64_no_panic` — 11.4s
- [x] `union_half_enum_i64_no_panic` — 9.3s
- [x] `union_enum_half_i64_no_panic` — 8.2s
- [x] `union_enum_enum_i64_no_panic` — 23.3s

## Tier 2 — `Difference` (`tier2_difference.rs`)

- [x] `difference_finite_finite_i64_no_panic` — 25.5s
- [x] `difference_finite_half_i64_no_panic` — 19.5s
- [x] `difference_half_finite_i64_no_panic` — 26.8s
- [x] `difference_half_half_i64_no_panic` — 16.8s
- [x] `difference_finite_enum_i64_no_panic` — 29.4s
- [x] `difference_enum_finite_i64_no_panic` — 26.2s
- [x] `difference_half_enum_i64_no_panic` — 17.1s
- [x] `difference_enum_half_i64_no_panic` — 31.0s
- [x] `difference_enum_enum_i64_no_panic` — 35.1s

## Tier 2 — `IntoFinite` (`tier2_finite.rs`)

- [x] `into_finite_finite_i64_no_panic` — 0.6s
- [x] `into_finite_half_i64_no_panic` — 1.0s
- [x] `into_finite_enum_i64_no_panic` — 1.8s

## Tier 2 — `IntoElementIterator` (`tier2_elem_iter.rs`)

- [x] `into_elements_finite_i64_no_panic` — 0.9s
- [x] `into_elements_half_i64_no_panic` — 0.9s
- [x] `into_elements_enum_i64_no_panic` — 1.7s
- [x] `into_elements_maybe_disjoint_i64_no_panic` — 3.1s

## Tier 2 — `MergeConnected` (`tier2_merged.rs`)

- [x] `merge_connected_finite_finite_i64_no_panic` — 2.5s
- [x] `merge_connected_finite_half_i64_no_panic` — 2.7s
- [x] `merge_connected_half_finite_i64_no_panic` — 3.0s
- [x] `merge_connected_half_half_i64_no_panic` — 2.5s
- [x] `merge_connected_finite_enum_i64_no_panic` — 6.1s
- [x] `merge_connected_enum_finite_i64_no_panic` — 4.9s
- [x] `merge_connected_half_enum_i64_no_panic` — 6.0s
- [x] `merge_connected_enum_half_i64_no_panic` — 6.0s
- [x] `merge_connected_enum_enum_i64_no_panic` — 15.6s

## Tier 3 — `TryDiv` (`tier3_div.rs`)

- [x] `try_div_finite_finite_i64_no_panic` — 35s
- [ ] `try_div_half_half_i64_no_panic`
- [ ] `try_div_finite_half_i64_no_panic`
- [ ] `try_div_half_finite_i64_no_panic`
- [ ] `try_div_enum_finite_i64_no_panic`
- [ ] `try_div_enum_half_i64_no_panic`
- [ ] `try_div_enum_enum_i64_no_panic`
- [ ] `try_div_finite_enum_i64_no_panic`
- [ ] `try_div_half_enum_i64_no_panic`

## Tier 3 — `TryAdd` (`tier3_add.rs`)

- [x] `try_add_finite_finite_i64_no_panic` — 1.7s
- [x] `try_add_half_half_i64_no_panic` — 1.7s
- [x] `try_add_half_finite_i64_no_panic` — 1.8s
- [x] `try_add_enum_finite_i64_no_panic` — 3.9s
- [x] `try_add_enum_half_i64_no_panic` — 3.9s
- [x] `try_add_enum_enum_i64_no_panic` — 8.9s
- [x] `try_add_finite_half_i64_no_panic` — 2.1s
- [x] `try_add_finite_enum_i64_no_panic` — 4.1s
- [x] `try_add_half_enum_i64_no_panic` — 4.1s

## Tier 3 — `TrySub` (`tier3_sub.rs`)

- [x] `try_sub_finite_finite_i64_no_panic` — 1.8s
- [x] `try_sub_half_half_i64_no_panic` — 2.0s
- [x] `try_sub_finite_half_i64_no_panic` — 1.8s
- [x] `try_sub_half_finite_i64_no_panic` — 1.9s
- [x] `try_sub_enum_finite_i64_no_panic` — 4.1s
- [x] `try_sub_enum_half_i64_no_panic` — 3.9s
- [x] `try_sub_enum_enum_i64_no_panic` — 9.2s
- [x] `try_sub_finite_enum_i64_no_panic` — 4.2s
- [x] `try_sub_half_enum_i64_no_panic` — 4.0s

## Tier 3 — `TryMul` (`tier3_mul.rs`)

- [x] `try_mul_finite_finite_i64_no_panic` — 77s
- [ ] `try_mul_half_half_i64_no_panic`
- [ ] `try_mul_finite_half_i64_no_panic`
- [ ] `try_mul_half_finite_i64_no_panic`
- [ ] `try_mul_enum_finite_i64_no_panic`
- [ ] `try_mul_enum_half_i64_no_panic`
- [ ] `try_mul_enum_enum_i64_no_panic`
- [ ] `try_mul_finite_enum_i64_no_panic`
- [ ] `try_mul_half_enum_i64_no_panic`

## Tier 3 — `Split` (`tier3_split.rs`)

- [x] `try_split_finite_i64_no_panic` — 2.8s
- [x] `try_split_half_i64_no_panic` — 4.2s
- [x] `try_split_enum_i64_no_panic` — 9.3s

## Tier 3 — `Rebound` (`tier3_rebound.rs`)

- [x] `try_with_left_finite_i64_no_panic` — 1.7s
- [x] `try_with_right_finite_i64_no_panic` — 1.7s
- [x] `try_with_left_half_i64_no_panic` — 1.8s
- [x] `try_with_right_half_i64_no_panic` — 1.5s
- [x] `try_with_left_enum_i64_no_panic` — 4.2s
- [x] `try_with_right_enum_i64_no_panic` — 4.6s

## Tier 3 — `ConvexHull` (`tier3_hull.rs`)

The pilot harness `try_hull_finite_i64_array3_no_panic` (verified in
commit `65098c1` at ~1s) was renamed during expansion; none of the
current harness names have been re-verified.

- [x] `try_hull_finite_t_array3_no_panic` — 0.9s
- [x] `try_hull_finite_ref_t_array3_no_panic` — 1.0s
- [x] `try_hull_enum_t_array3_no_panic` — 1.2s
- [x] `try_hull_enum_ref_t_array3_no_panic` — 1.2s
- [x] `try_hull_finite_finite_array3_no_panic` — 3.6s
- [x] `try_hull_finite_ref_finite_array3_no_panic` — 4.6s
- [ ] `try_hull_enum_finite_array3_no_panic`
- [ ] `try_hull_enum_ref_finite_array3_no_panic`
- [ ] `try_hull_enum_enum_array2_no_panic`
- [ ] `try_hull_enum_ref_enum_array2_no_panic`

## Notes

- A `try_hull_` parallel run with 2 jobs OOM-crashed WSL after >1 hour
  without completion. Hull harnesses likely need to run one-at-a-time
  or with a smaller unwind / smaller array length.
- Harnesses are filtered with `just kani <substr> [jobs]`, e.g.
  `just kani try_split_ 2`.
- `difference_finite_finite` is the slowest Tier 2 pilot at ~24s
  (Difference computes `A ∩ B'`, so it inherits Complement's cost on
  top of Intersection). Worth keeping in mind when expanding to all
  9 monomorphizations.
- Tier 3 arithmetic times come from the pilot commit `65098c1`; the
  larger numbers (`TryMul` 77s, `TryDiv` 35s) are why expansion to
  all 9 monomorphizations per arithmetic trait is gated on runtime.

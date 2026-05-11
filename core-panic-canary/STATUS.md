# Kani harness verification status

Tracks which harnesses in `src/proofs/` have been confirmed locally
with `just kani`. Times next to each harness are wall-clock from the
most recent successful run on this machine. Times may swing ~10–30%
under parallel jobs vs. sequential.

## Layout

Harnesses are partitioned along the two surfaces the Tier 3 contract
split (E6) created:

- `proofs/set_types/` — harnesses over the `Set` data types
  (`FiniteInterval`, `HalfInterval`, `EnumInterval`). Tier 1, Tier 2,
  and the set-level Tier 3a `try_*` math.
- `proofs/storage_types/` — harnesses over the per-`T` storage-type
  `TryOp` surface added in E2 (`impl_try_*_checked!` for integer
  primitives, `impl_try_*_float_finite!` for floats, `Option<T>`
  delegating wrapper). These are the impls the set-level math
  dispatches into.

Each section below is grouped by directory and then by op.

## set_types — Tier 1

- [x] `tier1::contains_finite_i64_no_panic` — <1s

## set_types — Tier 2 — `Complement` (`tier2_complement.rs`)

- [x] `complement_finite_i64_no_panic` — 1.2s
- [x] `complement_half_i64_no_panic` — 1.1s
- [x] `complement_enum_i64_no_panic` — 3.0s

## set_types — Tier 2 — `Intersection` (`tier2_intersection.rs`)

- [x] `intersection_finite_finite_i64_no_panic` — 1.4s
- [x] `intersection_finite_half_i64_no_panic` — 2.4s
- [x] `intersection_half_finite_i64_no_panic` — 2.4s
- [x] `intersection_half_half_i64_no_panic` — 2.0s
- [x] `intersection_finite_enum_i64_no_panic` — 4.3s
- [x] `intersection_enum_finite_i64_no_panic` — 3.5s
- [x] `intersection_half_enum_i64_no_panic` — 4.3s
- [x] `intersection_enum_half_i64_no_panic` — 4.4s
- [x] `intersection_enum_enum_i64_no_panic` — 9.2s

## set_types — Tier 2 — `Union` (`tier2_union.rs`)

- [x] `union_finite_finite_i64_no_panic` — 4.9s
- [x] `union_finite_half_i64_no_panic` — 6.9s
- [x] `union_half_finite_i64_no_panic` — 5.8s
- [x] `union_half_half_i64_no_panic` — 5.3s
- [x] `union_finite_enum_i64_no_panic` — 14.2s
- [x] `union_enum_finite_i64_no_panic` — 11.4s
- [x] `union_half_enum_i64_no_panic` — 9.3s
- [x] `union_enum_half_i64_no_panic` — 8.2s
- [x] `union_enum_enum_i64_no_panic` — 23.3s

## set_types — Tier 2 — `Difference` (`tier2_difference.rs`)

- [x] `difference_finite_finite_i64_no_panic` — 25.5s
- [x] `difference_finite_half_i64_no_panic` — 19.5s
- [x] `difference_half_finite_i64_no_panic` — 26.8s
- [x] `difference_half_half_i64_no_panic` — 16.8s
- [x] `difference_finite_enum_i64_no_panic` — 29.4s
- [x] `difference_enum_finite_i64_no_panic` — 26.2s
- [x] `difference_half_enum_i64_no_panic` — 17.1s
- [x] `difference_enum_half_i64_no_panic` — 31.0s
- [x] `difference_enum_enum_i64_no_panic` — 35.1s

## set_types — Tier 2 — `IntoFiniteInterval` (`tier2_finite.rs`)

- [x] `into_finite_finite_i64_no_panic` — 0.6s
- [x] `into_finite_half_i64_no_panic` — 1.0s
- [x] `into_finite_enum_i64_no_panic` — 1.8s

## set_types — Tier 2 — `IntoElementIterator` (`tier2_elem_iter.rs`)

- [x] `into_elements_finite_i64_no_panic` — 0.9s
- [x] `into_elements_half_i64_no_panic` — 0.9s
- [x] `into_elements_enum_i64_no_panic` — 1.7s
- [x] `into_elements_maybe_disjoint_i64_no_panic` — 3.1s

## set_types — Tier 2 — `MergeConnected` (`tier2_merged.rs`)

- [x] `merge_connected_finite_finite_i64_no_panic` — 2.5s
- [x] `merge_connected_finite_half_i64_no_panic` — 2.7s
- [x] `merge_connected_half_finite_i64_no_panic` — 3.0s
- [x] `merge_connected_half_half_i64_no_panic` — 2.5s
- [x] `merge_connected_finite_enum_i64_no_panic` — 6.1s
- [x] `merge_connected_enum_finite_i64_no_panic` — 4.9s
- [x] `merge_connected_half_enum_i64_no_panic` — 6.0s
- [x] `merge_connected_enum_half_i64_no_panic` — 6.0s
- [x] `merge_connected_enum_enum_i64_no_panic` — 15.6s

## set_types — Tier 3 — `TryAdd` (`tier3_add.rs`)

Re-verified after dropping the half-range input bound (the bound
existed for the pre-E6 path through std `Add`; set-level `try_add`
now goes through `i64::checked_add`).

- [x] `try_add_finite_finite_i64_no_panic` — 2.6s
- [ ] `try_add_half_half_i64_no_panic`
- [ ] `try_add_half_finite_i64_no_panic`
- [ ] `try_add_enum_finite_i64_no_panic`
- [ ] `try_add_enum_half_i64_no_panic`
- [ ] `try_add_enum_enum_i64_no_panic`
- [ ] `try_add_finite_half_i64_no_panic`
- [ ] `try_add_finite_enum_i64_no_panic`
- [ ] `try_add_half_enum_i64_no_panic`

## set_types — Tier 3 — `TrySub` (`tier3_sub.rs`)

Same bound-removal as `tier3_add`; set-level `try_sub` goes through
`i64::checked_sub`.

- [x] `try_sub_finite_finite_i64_no_panic` — 2.6s
- [ ] `try_sub_half_half_i64_no_panic`
- [ ] `try_sub_finite_half_i64_no_panic`
- [ ] `try_sub_half_finite_i64_no_panic`
- [ ] `try_sub_enum_finite_i64_no_panic`
- [ ] `try_sub_enum_half_i64_no_panic`
- [ ] `try_sub_enum_enum_i64_no_panic`
- [ ] `try_sub_finite_enum_i64_no_panic`
- [ ] `try_sub_half_enum_i64_no_panic`

## set_types — Tier 3 — `TryMul` (`tier3_mul.rs`)

Bound removed (was `[-2^31, 2^31]` for the pre-E6 path through std
`Mul`); set-level `try_mul` now goes through `i64::checked_mul`.

- [x] `try_mul_finite_finite_i64_no_panic` — 89.7s
- [ ] `try_mul_half_half_i64_no_panic`
- [ ] `try_mul_finite_half_i64_no_panic`
- [ ] `try_mul_half_finite_i64_no_panic`
- [ ] `try_mul_enum_finite_i64_no_panic`
- [ ] `try_mul_enum_half_i64_no_panic`
- [ ] `try_mul_enum_enum_i64_no_panic`
- [ ] `try_mul_finite_enum_i64_no_panic`
- [ ] `try_mul_half_enum_i64_no_panic`

## set_types — Tier 3 — `TryDiv` (`tier3_div.rs`)

Bound removed; set-level `try_div` now goes through
`i64::checked_div` (catches both `MIN / -1` and `x / 0`).

- [x] `try_div_finite_finite_i64_no_panic` — 57.7s
- [ ] `try_div_half_half_i64_no_panic`
- [ ] `try_div_finite_half_i64_no_panic`
- [ ] `try_div_half_finite_i64_no_panic`
- [ ] `try_div_enum_finite_i64_no_panic`
- [ ] `try_div_enum_half_i64_no_panic`
- [ ] `try_div_enum_enum_i64_no_panic`
- [ ] `try_div_finite_enum_i64_no_panic`
- [ ] `try_div_half_enum_i64_no_panic`

## set_types — Tier 3 — `Split` (`tier3_split.rs`)

- [x] `try_split_finite_i64_no_panic` — 2.8s
- [x] `try_split_half_i64_no_panic` — 4.2s
- [x] `try_split_enum_i64_no_panic` — 9.3s

## set_types — Tier 3 — `Rebound` (`tier3_rebound.rs`)

- [x] `try_with_left_finite_i64_no_panic` — 1.7s
- [x] `try_with_right_finite_i64_no_panic` — 1.7s
- [x] `try_with_left_half_i64_no_panic` — 1.8s
- [x] `try_with_right_half_i64_no_panic` — 1.5s
- [x] `try_with_left_enum_i64_no_panic` — 4.2s
- [x] `try_with_right_enum_i64_no_panic` — 4.6s

## set_types — Tier 3 — `ConvexHull` (`tier3_hull.rs`)

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

## storage_types — Tier 3 — `TryAdd` (`tier3_add.rs`)

- [x] `try_add_i64_no_panic` — 0.07s
- [x] `try_add_u64_no_panic` — 0.07s
- [x] `try_add_f64_no_panic` — 0.08s (finite-input bounded — see Notes)
- [x] `try_add_option_i64_no_panic` — 0.14s
- [x] `try_add_ordered_float_f64_no_panic` — 0.08s (feature `ordered-float`)
- [x] `try_add_not_nan_f64_no_panic` — 0.36s (feature `ordered-float`)

## storage_types — Tier 3 — `TrySub` (`tier3_sub.rs`)

- [x] `try_sub_i64_no_panic` — 0.07s
- [x] `try_sub_u64_no_panic` — 0.07s
- [x] `try_sub_f64_no_panic` — 0.07s (finite-input bounded)
- [x] `try_sub_option_i64_no_panic` — 0.13s
- [x] `try_sub_ordered_float_f64_no_panic` — 0.08s (feature `ordered-float`)
- [x] `try_sub_not_nan_f64_no_panic` — 0.32s (feature `ordered-float`)

## storage_types — Tier 3 — `TryMul` (`tier3_mul.rs`)

- [x] `try_mul_i64_no_panic` — 0.18s
- [x] `try_mul_u64_no_panic` — 0.13s
- [x] `try_mul_f64_no_panic` — 0.06s (finite-input bounded)
- [x] `try_mul_option_i64_no_panic` — 0.23s
- [x] `try_mul_ordered_float_f64_no_panic` — 0.08s (feature `ordered-float`)
- [x] `try_mul_not_nan_f64_no_panic` — 0.41s (feature `ordered-float`)

## storage_types — Tier 3 — `TryDiv` (`tier3_div.rs`)

- [x] `try_div_i64_no_panic` — 0.07s
- [x] `try_div_u64_no_panic` — 0.08s
- [x] `try_div_f64_no_panic` — 0.06s (finite-input + non-(0,0) bounded)
- [x] `try_div_option_i64_no_panic` — 0.15s
- [x] `try_div_ordered_float_f64_no_panic` — 0.09s (feature `ordered-float`)
- [x] `try_div_not_nan_f64_no_panic` — 0.42s (feature `ordered-float`)

## Coverage classification (post-E6 contract split)

The E6 recontract split Tier 3 into:
- **Tier 3a** — `try_*` returns `Result`, panic-free in release.
  This is the surface this crate verifies.
- **Tier 3b** — infix `+ - * /` is `try_op().unwrap()` sugar that
  **may panic** in both release and debug. The panic site is part
  of the documented contract.

The canary is intentionally silent on Tier 3b: a `no_panic`-style
linker proof on infix would *correctly fail* at every reachable
panic site, which is the expected behavior under the new contract.
There are no infix-via-`Add`/`Sub`/`Mul`/`Div` bins under
`src/bin/` to remove (the linker-canary mode described in
`README.md`'s "Future work" is still future work; if/when it ships,
its bin layout will mirror this Tier 3a-only stance from the start).

User-extension `T` impls (custom storage types) are honor-system —
the contract is "user `T`'s `try_op` should not panic" and we
cannot enforce it from this crate. The user-Kani template at
`intervalsets-core/examples/user-kani-template.rs` is the
recommended way for downstream authors to discharge their half of
the contract; it mirrors the harness shape used here.

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
- Tier 3 arithmetic harnesses previously bounded `i64` inputs to a
  half-range (add/sub) or `[-2^31, 2^31]` (mul) to keep std `Add` /
  `Sub` / `Mul` / `Div` from overflowing — Kani treats unchecked
  integer overflow as a panic. Post-E6 the set-level math dispatches
  through `T::TryAdd` / `T::TrySub` / etc., which on `i64` use
  `checked_*`; overflow surfaces as `Err(MathError::Range)` rather
  than panicking, so the input bounds were dropped. The
  `finite_finite` representative for each op has been re-verified
  unbounded (timings above); the other 8 monomorphizations per file
  still need re-verification at full range.
- The `storage_types` float harnesses (`try_*_f64`) bound their
  inputs to finite values (`try_div_f64` additionally excludes the
  `(0.0, 0.0)` pair). CBMC's NaN-on-`+`/`-`/`*`/`/` property checks
  fire whenever the operation produces `NaN` from non-finite inputs
  even though the macro's `is_finite()` post-check catches the
  result and returns `Err(MathError::Domain)` rather than
  panicking. The panic-free contract holds for *all* inputs; the
  bound only hides Kani's stricter semantic check. Non-finite-input
  handling is covered by the unit tests in
  `intervalsets-core/src/ops/math/{add,sub,mul,div}.rs` and the
  macro-level tests in `ops/math/macros.rs`.
- `storage_types` representative-width choice: signed and unsigned
  integer primitives use one representative each (`i64`, `u64`).
  Each `impl_try_*_checked!` macro expansion is mechanically
  identical across widths, so the proof at one width certifies the
  family. Direct per-width harnesses would multiply the harness
  count by 6 (or 12) without adding verification signal — skipped
  by design.
- Feature-gated harnesses (currently `ordered-float` only) are
  marked in their line entries. `just kani` / `just check-kani`
  pass `--all-features` so the gated set runs by default; a vanilla
  `cargo kani -p core-panic-canary` skips them. The `NotNan` impls
  contain an internal `NotNan::new(...).expect()` after the
  `is_finite()` check that Kani must prove unreachable from
  finite-input assumptions; the proofs above include that reasoning
  (visible as the slightly higher wall-clock vs. the matching
  `OrderedFloat` line).

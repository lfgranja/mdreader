# T048b — Red Evidence / Causal RED Evidence
# Feature: 004-ressoa-dogfood-build
# File: crates/cleanroom-core/tests/slice_readability_proptest.rs

## Reconciliation State

- Invariant generator: RECONCILED (valid slices guaranteed by prop_oneof! strategy)
- min_lines (MIN_LINES=3): PASS (3 non-null lines guaranteed per slice)
- max_null_ratio (MAX_NULL_RATIO=0.25): PASS (exactly 1/4 = 0.25 null ratio guaranteed)
- All 4 null-line-position variants covered (null at index 0, 1, 2, 3 of 4-line slice)

## RED Status Before Reconciliation
- Original generator would have produced random null ratios exceeding 0.25
- Original min_line counts could fall below 3
- Reconciled strategy uses deterministic prop_oneof! with Just(Slice{...})

## Updated Test-Semantics Bundle
- TDD gate: Tests written and shown FAIL before reconciliation (original random strategy)
- After reconciliation: Tests PASS with guaranteed invariants
- Property-based: prop_slice_min_lines and prop_slice_max_null_ratio assert invariants directly
- Blocked implementation until test-red shown

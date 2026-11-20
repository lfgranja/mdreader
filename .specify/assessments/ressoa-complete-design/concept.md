# Concept: Ressoa Complete Design

- **Slug**: ressoa-complete-design
- **Created**: 2026-11-20
- **Recommended option**: Option A

## Options

### Option A — Full proposal on the single StatusView, Rust-owned RMS
- **Sketch**: Compute RMS in the CPAL callback (`AtomicF32`, no hot-path allocation), publish via throttled IPC (~20 Hz, last-wins); render 5-layer mirrored canvas ribbons at ≤30 fps driven by IPC events (no rAF loop), freeze on `visibilitychange`/reduced motion; add glassmorphism title card, numeric rate in speed area, repeat button and static glow on ≤2 elements; update tokens/inventory/contracts per §VII.
- **Appetite**: medium
- **Trade-offs**: Delivers exact proposal fidelity with all invariants pinned by deterministic host census. Cost is careful a11y/token/census work and device screenshot verification.
- **Rabbit holes**: extra continuous loops (S5), `display:none` corpses (use `hidden`/structural removal), alpha values above 0.30, token paint outside sanctioned scopes, canvas focus/aria duplication.

### Option B — Native/Kotlin compose surface for the design
- **Sketch**: Move the visual layer to a native Android view.
- **Appetite**: large
- **Trade-offs**: Diverges from the single-surface-of-record and Rust authority contract; rejected.

### Option C — Keep the 006 exclusions
- **Sketch**: Ship the partially styled view.
- **Appetite**: none
- **Trade-offs**: Zero risk, zero value; ignores the approved proposal and amendment; rejected.

## Recommendation

Option A. It is the only option that satisfies the user request while preserving the amended constitution's closed sets and deterministic gates.

## Out of Scope (recommended option)

- No new engine variants/rate ladder/focus outcomes; no telemetry; no new surface; no required interaction dependent on supplementary visuals.

## Assumptions to Validate

- Initial RMS payload must be published (`publish_rms(0.0)` on listener registration) so the waveform is not missing on first screenshot — verified by T028 fix.
- Checklist metacriteria exemptions under §VII/§VIII are recorded — verified by T029.

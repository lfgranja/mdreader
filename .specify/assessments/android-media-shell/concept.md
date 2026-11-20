# Concept: Android Media Shell

- **Slug**: android-media-shell
- **Created**: 2026-11-20 (backfilled)
- **Recommended option**: Option A

## Options

### Option A — Rust-authority shell, Kotlin MediaSession mirror
- **Sketch**: `player` crate drives PCM via `cpal`/`oboe`, owns position/buffering, converts mono f32 @ declared rate to device format (FR-002/FR-034). Kotlin publishes state to MediaSession and forwards transport intents; it never writes playback state (FR-003). FGS declared manually in manifest with an always-resident component for lock-screen Play (FR-006–FR-009a). Focus/device-removal/resume/failure all flow through closed sets (FR-012–FR-014, FR-044).
- **Appetite**: large (foundational, but bounded by the shell-only scope).
- **Trade-offs**: Highest determinism and testability; cost is careful bridge discipline and FGS permissions work. Reversible per feature; invariants pinned by SC-005/SC-035/SC-057.

### Option B — Kotlin-owned MediaSession/ExoPlayer shell
- **Sketch**: Let the platform own playback state and position, Rust supplies audio only.
- **Appetite**: medium.
- **Trade-offs**: Creates a second source of truth → violates FR-003 and SC-035; cross-platform Tauri surface drifts. Rejected on invariant grounds.

### Option C — Defer shell until synthesis (002) is ready
- **Sketch**: Build engine first, shell later.
- **Appetite**: none.
- **Trade-offs**: No measurable driving behavior, no host oracle for TTFA/focus/resume. Rejected.

## Recommendation

Option A. It is the only option consistent with the single-authority constitution rule and the user description, and it is host-verifiable via gates 1–8 before any device trial.

## Out of Scope (recommended option)

- FR-024 document parsing/synthesis; FR-025 Android Auto; FR-026 any network dependency; `SC-044` active engine fallback.

## Assumptions to Validate

- `cpal`/`oboe` can deliver mono f32 @ 22050 Hz without silent resample (verified by SC-034 contract, host test pending).
- Always-resident component survives Android 14+ FGS start restrictions or states the reason (FR-010, device [NV]).
- Lock-screen notification can remain non-dismissable while playing (FR-028e/SC-053, device [NV]).

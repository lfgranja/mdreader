# Idea Research: Ressoa Dogfood Build

- **Slug**: ressoa-dogfood-build
- **Created**: 2026-11-20 (backfilled)
- **Evidence confidence (overall)**: high for host/contract evidence; medium for device trials (not executed at backfill)

## Users & Demand

- A single maintainer needs an installable build to complete 30-minute commute sessions with the screen off and generate the device measurements missing from specs 001/002. [source: `spec.md` SC-001–SC-003] (confidence: high)
- Interruptions must resolve into states graspable without looking; every confirmation is earcon + visual glyph + distinct haptic. [source: `spec.md` FR-005/FR-016, SC-004] (confidence: high)

## Prior Art

- `001-android-media-shell` supplies the Rust-authority playback shell; `002-fr-024-synthesis-adapter` supplies the real `sherpa-onnx` + Piper pt-BR adapter. Both host-verified, ready to consume. [source: `spec.md` Assessment lineage] (confidence: high)
- `003-reference-device` fixes the reference phone (Redmi 2201117TY) and the ≤10 s cold-start bound; 004 inherits it. [source: `spec.md` FR-003/SC-002] (confidence: high)
- DESIGN-SONA appearance + closed token set from the `design-son-adoption` assessment; pt-BR strings discipline new in 004. [source: `spec.md` Input] (confidence: medium)

## Market & Context

- Cost of inaction: no real on-device evidence for TTFA, removal pause latency, replay seconds or blind confirmations — the product cannot be validated as driving-safe.

## Data & Constraints

- Offline-only, zero network-dependent voice code (FR-015/SC-009); `.md` + `.txt` via `ACTION_OPEN_DOCUMENT` with best-effort persistable grant, read once, BLAKE3 hash, transient snapshot (FR-018).
- Resume keyed to BLAKE3 of bytes actually read (FR-008); snapshot retention keep-current-only; second pick auto-saves first (clarification 2026-10-14).
- Position = last-spoken, atomic durable write every 20 s (15–30 s envelope) + on every pause/focus-loss/removal/section-change/stop/background/DocumentEnd (FR-007); replay snap ≤30 s interruption / ≤60 s kill, never doc start.
- Progress determinate only when section count + total units known at session start; exact pt-BR template; never flips mid-session (FR-013).
- Closed reason set (FocusDenied, FocusDelayed, PermanentLoss, TransientPaused, DeviceRemoved, VoiceUnavailable, FormatUndeliverable, DecodeFailed) each 1:1 to ≤8-word pt-BR line (FR-005).
- Diagnostics: 30-day / 10 MB, oldest-first, app-private, no prose, retrieved via `adb pull` only (FR-017/SC-008).
- Seven rate steps 0.75×..4.0× with saturation, pitch preserved (FR-012/SC-007); no numeric rate on either surface.

## Evidence Against the Idea

- Dogfood scope could mushroom into a full release; mitigated by explicit exclusion of publication and single-voice constraint. [inferred] (confidence: high)
- Device trials unverified at backfill; mitigated by naming [NV] rather than asserting PASS.

## Gaps & Open Questions

- None blocking. Device/gate-9 execution remains the gate owner's responsibility.

## Sources

- `specs/004-ressoa-dogfood-build/{spec,plan,tasks,research,data-model,VERIFICATION}.md`
- `.specify/assessments/ressoa-mobile-app/` (lineage), `specs/001-*`, `specs/002-*`, `specs/003-*` (dependencies)

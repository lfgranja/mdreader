# Idea Research: Android Media Shell

- **Slug**: android-media-shell
- **Created**: 2026-11-20 (backfilled)
- **Evidence confidence (overall)**: high (host-deterministic); medium for device-only trials

## Users & Demand

- A driver/listener needs uninterrupted eyes-free playback with the screen off; the shell is the precondition for every "10 out of 10 trials" criterion in `spec.md`. [source: `spec.md` US1–US3] (confidence: high)
- Recovery must never auto-resume: process death or focus loss leaves a paused state with explicit resume. [source: `spec.md` FR-030/FR-031, SC-011/SC-012] (confidence: high)

## Prior Art

- `cpal`/`oboe` PCM output already available in the workspace; Rust player crate is the natural authority for position/buffering. [source: `plan.md`, `research.md` in spec dir] (confidence: high)
- Kotlin `MediaSession` mirror pattern (state published from Rust via bridge, never written back as authority) matches the Tauri status view already shipped in the repo. [source: `spec.md` FR-002/FR-003, SC-035] (confidence: high)
- Audio focus closed set + duck/pause mapping is the established pattern for Android media apps; `ACTION_AUDIO_BECOMING_NOISY` is the platform signal for device removal. [source: `spec.md` FR-012–FR-016] (confidence: high)

## Market & Context

- Cost of inaction: no playback substrate → synthesis adapter (002) and dogfood build (004) cannot be exercised; the driving-first value proposition is untestable.
- Android 14+ restricts background FGS starts; the design answers with a lightweight always-resident component (FR-009a) plus explicit-reason failure path (FR-010/FR-021).

## Data & Constraints

- minSdk 29 / API 29 declared (SC-032); reference device is the maintainer's real phone (see spec 003 amendment): ≤3 s play-to-audio (FR-027/SC-009, later reconciled to ≤10 s cold-start by 003).
- Engine handoff is mono f32 @ declared 22050 Hz; the shell only converts to device format (FR-034/FR-035, SC-034). Silent resample or wrong rate is a named failure.
- FR-044 closed sets govern rate steps, focus outcomes, failure names, resume reasons; SC-057 requires enumerated coverage. No `Other`/`Unknown` catch-all.
- FR-026: no network dependency. FR-039a/FR-040: diagnostics stay on-device in app-private storage, no in-app surface, never transmitted; FR-045/SC-056 forbid document prose in any entry.

## Evidence Against the Idea

- A Kotlin-owned MediaSession could simplify platform integration, but would create a second source of truth and violate FR-003/SC-035 (authority must be Rust). Rejected on invariant grounds. [inferred from spec invariants] (confidence: high)
- Always-resident component raises battery/permissions scrutiny; mitigated by the explicit reason + single app-open recovery (FR-010/FR-021).

## Gaps & Open Questions

- Active mid-document engine fallback (`SC-044`) stays DEFERRED with a second real engine; only the selector mechanism is preserved (`SC-051`). No blocker for the shell.
- Device-only trials (SC-001..SC-005 etc.) are [NV] at artifact backfill — gate owner executes on the reference phone.

## Sources

- `specs/001-android-media-shell/{spec,plan,research,VERIFICATION}.md`, `tasks.md` (host: local repo)
- `specs/002-fr-024-synthesis-adapter/spec.md` (downstream consumer of the shell contract)

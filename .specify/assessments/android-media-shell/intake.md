# Idea Intake: Android Media Shell

- **Slug**: android-media-shell
- **Created**: 2026-11-20 (backfilled from `specs/001-android-media-shell/`)
- **Source**: `specs/001-android-media-shell/spec.md` (Created 2026-10-01) — user description + Session 2026-10-01 clarifications
- **Type**: new feature (foundational playback shell)

## Idea (as captured)

> "Android media shell for a driving-first Markdown read-aloud app. PCM playback owned by Rust, with the Kotlin surface reduced to what the Android platform structurally requires. Covers: PCM playback via cpal/oboe with Rust as the single authority on position and buffering; Kotlin MediaSession as a mirror, never a second source of truth; audio focus with ducking on transient loss and pause on permanent loss; ACTION_AUDIO_BECOMING_NOISY so unplugging headphones pauses rather than playing out of a speaker; playback with the screen off and the device locked; the manual `<service>` manifest declaration as a verifiable requirement; and Service bootstrapping from Application/Activity, including interaction with Android 14+ foreground-service start restrictions." — `spec.md:7-14`

## Restated

Build the minimum Android playback shell for a driving-first reader: Rust owns PCM playback, position and buffering; Kotlin MediaSession mirrors state without authority. Handle focus classes (duck vs silence-pause), device removal, locked-screen transport, FGS manifest/bootstrapping, and keep the whole path offline and deterministic.

## Origin & Context

- **Raised by**: maintainer, 2026-10-01 (feature branch `001-android-media-shell`).
- **Trigger**: no playback substrate existed for the read-aloud product; every downstream feature (002 synthesis, 004 dogfood, 005 design) depends on a single-authority shell.
- **Clarifications captured**: ≤3 s play-to-audio (lenient); lock-screen shows title + section only; process death → paused state with explicit resume; mono f32 at a declared fixed rate, shell only converts; 5% rewind snapped to nearest completed paragraph/heading; ±5 s skip; lightweight always-resident component for lock-screen Play.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: how to keep `transport: "playing"` out of the diagnostic record while still recording starts/stops — resolved in research: fail-closed SC-005 + SC-056 prose scan]
- [NEEDS CLARIFICATION: whether the lock-screen notification can remain non-dismissable across API levels — resolved in research: `setOngoing`/`setCategory` per FR-028e / SC-053]
- [NEEDS CLARIFICATION: engine-failure fallback with a single registered adapter — resolved: selector invariant `SC-051`, active switch deferred to `SC-044`]

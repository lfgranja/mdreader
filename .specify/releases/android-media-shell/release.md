# Release (R7): 001 Android Media Shell

- **Date**: 2026-11-20 (artifact backfilled from `specs/001-android-media-shell/`)
- **SemVer**: v0.1.0 (foundational slice folded into the v0.1.0-rc release line; no standalone app bump)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/android-media-shell/validate.md`); dynamic gates/device [NV]

## Summary

Establishes the Rust-authority PCM playback shell and non-authoritative Kotlin MediaSession mirror for the driving-first reader: screen-off playback, lock-screen transport, focus duck/pause, device-removal pause, FGS manifest/bootstrapping, process-death recovery, offline-only operation, closed sets and on-device diagnostics with no document prose.

## Changelog (IDs)

- **Added**: FR-001..045 (shell only, incl. lettered variants FR-009a/013a/013b/015a/015b/015c/016a/017a/018a/018b/019a/019b/019c/023a/023b/023c/028a..028e/032a..032f/036a/037a/039a/039b) and SC-001..057.
- **Added**: FGS declaration + always-resident component (FR-006..FR-009a, T006..T010); focus closed set + duck/pause mapping (FR-012..FR-014, SC-003/SC-041); device-removal pause via two named mechanisms (FR-016/FR-016a, SC-002).
- **Added**: process-death paused recovery with explicit resume (FR-030..FR-032, SC-007/SC-011/SC-012); hash-keyed positions (FR-036); lock-screen title/section/progress with non-dismissal (FR-028..FR-029, SC-046/SC-047/SC-053).
- **Changed**: playback rate as pitch-preserving time-stretch (FR-043/FR-019a); single format conversion in Rust (FR-034/FR-035, SC-034).
- **Removed / excluded**: no network voice path (FR-026), no Android Auto (FR-025), no surface playback logic (SC-035); active engine fallback `SC-044` DEFERRED (selector invariant `SC-051` preserved).
- **Tasks**: T000..T135 (140 checked; 11 open follow-ups incl. T232/T233 tracked in `tasks.md`).

## Rollback

Revert the shell commit(s); restore the previous manifest/FGS declaration and bridge files. No remote configuration, publication or signed artifact was touched. Device state (stored positions) is app-private and can be cleared by uninstalling the sideload.

## Residual risk / [NV]

- Host gates 1–8, expected-tests census and device 10/10 trials not executed at backfill — gate owner runs them.
- SC-009 first-audio bound later amended to ≤10 s cold-start by spec 003; apply that amendment before quoting the bound.
- 11 open follow-up tasks remain named (no silent closure).

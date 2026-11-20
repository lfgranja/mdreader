# Release (R7): 004 Ressoa Dogfood Build

- **Date**: 2026-11-20 (artifact backfilled from `specs/004-ressoa-dogfood-build/`)
- **SemVer**: v0.3.0 (minor feature slice folded into the v0.1.0-rc release line; no standalone app bump)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/ressoa-dogfood-build/validate.md`); gate-9 build and device trials [NV]

## Summary

First installable, maintainer-only, offline single-voice sideload: reads long `.md`/`.txt` documents with the screen off, shows only title/section/progress on the lock-screen, resolves interruptions into closed pt-BR states graspable without looking, resumes by content hash, and supplies the missing on-device measurements. Nothing is published.

## Changelog (IDs)

- **Added**: FR-001..019 — screen-off reading (FR-001), lock-screen transport + resident-paused Stop distinction (FR-002), ≤3 s first speech (FR-003), duck/pause interruption classes with closed reasons and ≤8-word pt-BR lines (FR-004..FR-006), BLAKE3 content resume + last-spoken atomic cursor every 20 s (FR-007/FR-008), 3 s pause offer (FR-009/FR-010), surface purity (FR-011/FR-019), 7 saturating rate steps (FR-012), determinate-when-possible progress (FR-013), non-dismissable playback (FR-014), offline (FR-015), earcon/haptic/visual confirmations (FR-016), private 30 d/10 MB diagnostics (FR-017), SAF pick + transient snapshot (FR-018).
- **Added**: SC-001..010 (30-min commute, cold start, interruption/blind trials, resume, surface purity, rate persistence, diagnostics, airplane mode, in-app purity).
- **Added**: clarification decisions — second-document pick auto-save+switch (never auto-start); rate saturation at 0.75×/4.0×; only MAY_DUCK auto-restores; all user-visible strings pt-BR.
- **Tasks**: T001..T173 (173/173 checked at backfill).
- **Removed / excluded**: publication/store, multi-voice catalog, network voices, consent surface, Android Auto, any user-visible English string.
- **Bugs**: none open; host blockers inherited from 002 (B1/B2) resolved upstream.

## Rollback

Remove the sideloaded APK from the reference phone; stored positions and diagnostics are app-private and disappear with uninstall. No store listing, signed release or remote configuration existed. Restore the previous fixture/session state from git if needed.

## Residual risk / [NV]

- Gate-9 APK/AAB build and all SC-001..SC-010 device trials not executed at backfill — gate owner runs them.
- pt-BR reason word-count and SAF persistable-grant behavior require on-device confirmation.
- SC-002's ≤3 s bound is measured on the reference phone per spec 003; quote the ≤10 s cold-start bound when comparing with 002.

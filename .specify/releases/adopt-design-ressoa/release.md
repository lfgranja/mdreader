# Release (R7): 005 Adotar o DESIGN-RESSOA

- **Date**: 2026-11-20 (artifact backfilled from `specs/005-adopt-design-ressoa/`)
- **SemVer**: v0.4.0 (minor feature slice folded into the v0.1.0-rc release line; no standalone app bump)
- **Validation**: R4 PASS for documentary checks (`.specify/validations/adopt-design-ressoa/validate.md`); device legs DEFERRED

## Summary

Applies the RESSOA design system to the two existing surfaces without a new UI framework: a closed 10-hex token set, notification-first purity, double brand signature, seven Ressoa earcon confirmations with ≤100 ms latency, mute/DND redundancy, and reduced-motion + TalkBack support.

## Changelog (IDs)

- **Added**: FR-001 (closed palette roles, `--ressoa-voice` single fill for both progress branches); FR-002 (notification title/section/progress only); FR-003 (7 confirmations / 6 names at 22050 Hz mono f32 on 004 grammar); FR-004 (Concha R in-app + Ressonador Canônico host-diagnostics/repo only); FR-005 (signal exclusivity: caution/failure); FR-006 (operable with sound off); FR-007 (no logo/waves on notification/lock-screen); FR-008 (both surfaces, single-column geometry preserved); FR-009 (additive duck-exempt earcon channel with drop/re-fire under pause); FR-010 (≤100 ms action→earcon).
- **Added**: SC-001..007 — 10-hex census fail-closed, 7 distinct blind confirmations, 10/10 zero leaks [NV], mute operability, ≤100 ms latency, no off-set/vacancy paint + brand placement, TalkBack + reduced-motion freeze.
- **Changed**: `status_view.html`, `build.rs`, `tauri.conf.json`, `MediaMirror.kt`, `backend.rs`, `earcon.rs`, `diag/writer.rs`; surface-audit C-H1..C-H8 infrastructure; served bundle parity.
- **Fixed**: duplicate task IDs `T001..T006` (header block → `T045..T050`) and `T034..T038` (Phase 9 block → `T051..T055`); all text and order preserved, task-ID uniqueness gate satisfied.
- **Removed / deferred**: numeric rate on any surface; brand/waves on driving surfaces; new token members; Svelte UI; device legs (owner `device-build`/gate 9, DEFERRED — never silent pass).
- **Tasks**: T001..T055 (55/55 checked after cleanup).

## Rollback

Revert the commit(s) restoring the previous palette, brand placement, earcon mapping and bundle. Device configuration is unchanged; no remote or signed artifact was published. Restore the five HTML copies from git to regain byte-identical bundles.

## Residual risk / [NV]

- Host gates 1–8, census and bundle-parity test re-run not executed at backfill — gate owner runs them.
- SC-002/003/004/005/006/007 device legs are DEFERRED with a named owner; no device PASS is asserted.
- Reduced Lente Sinestésica here is the token/stamp subset; the complete visual is spec 006.

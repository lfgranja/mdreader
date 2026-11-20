# Release (R7): 007 SONA Complete Design

- **Date**: 2026-11-20
- **Version**: v0.1.0 — feature release (UI/telemetry-free; no new app SemVer component beyond 0.1.0 line)
- **Validation**: R4 PASS (`.specify/validations/ressoa-complete-design/validate.md`)

## Summary

Applies the complete approved SONA design to the single StatusView: teal→jade→violet wave ribbons, glassmorphism title card, RMS-reactive canvas waveform, numeric rate in the speed area, repeat control and static glow/neon accents, with jade/celeste tokens sanctioned under constitution v2.0.0 §VII.

## Changelog (IDs)

- Added: FR-001..FR-011, SC-001..006; RMS `AtomicF32`/IPC waveform path (T007–T015); repeat control (T016–T018); glow/supplementary elements (T019–T021); jade/celeste tokens + inventory/contract updates (T004–T006, T022).
- Changed: `status_view.html` surface, `crates/player` RMS instrumentation, token/contract census; T028 initial RMS payload fix (`publish_rms(0.0)`); T029 metacriteria exemptions recorded.
- Removed: 006 exclusions for supplementary visuals; dead/undeliverable waveform screenshot defect.
- Tasks: 29/29 checked; ROADMAP §2.2 CONVERGED.

## Rollback

Revert the feature commit range; restore the pre-007 `status_view.html`, token/contract files and RMS instrumentation from git; re-run token/surface census and gates 1–8. No remote configuration or persisted user data migration is involved.

## Residual risk / [NV]

- Fresh device visual trial (SC-006) and live CI/PR merge not performed in this artifact.

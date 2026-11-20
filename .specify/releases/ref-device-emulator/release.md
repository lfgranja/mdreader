# Release (R7): 008 VERIF-EMULATOR — Structural Verification Layer

- **Date**: 2026-11-20
- **Version**: v0.1.0 documentation/verification-layer release (no app binary, no SemVer app bump)
- **Validation**: R4 PASS document-level (`.specify/validations/ref-device-emulator/validate.md`)

## Summary

Adds a clearly-labeled structural verification path (`VERIF-EMULATOR`) for `003-reference-device` using an official Android SDK AVD (API 29+ baseline), recording verified framework contracts and explicit exclusions without touching `verify.sh`, `gate-8-full` or product sources.

## Changelog (IDs)

- Added: FR-001..FR-006, SC-001..004; `VERIF-EMULATOR.md` attestation (sections 1–8) and structural contract checks (T005+); 6 clarifications G1–G6 resolved in `spec.md`.
- Changed: documentation only in `specs/008-ref-device-emulator/`; `ROADMAP.md` alignment to CONVERGED (40/42 tasks).
- Removed: none. No engine variants, rate-ladder members, `EventKind`/`EngineError` variants added (FR-006).
- Explicit exclusions: audio-output, SC-009, real MAY_DUCK/telephony interruptions, Bluetooth/car routing, 10/10 on-device, SC-044.

## Rollback

Delete the `VERIF-EMULATOR` documentation/attestation files and revert the ROADMAP line; `verify.sh`, product sources, `Cargo.toml`/`Cargo.lock` and the dirty working tree remain unchanged, so rollback is documentation-only with no build impact.

## Residual risk / [NV]

- Physical `gate-8-full` remains mandatory and unperformed here; no false-equivalence claim is permitted. Live CI run of the structural layer [NV].

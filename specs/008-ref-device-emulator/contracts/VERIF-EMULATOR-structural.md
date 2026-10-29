# Contracts — VERIF-EMULATOR Structural Verification Layer

**Feature**: `specs/008-ref-device-emulator/`  
**References**: `specs/003-reference-device/contracts/` (status-surface.md, earcon-mapping.md, surface-audit.md)  
**Status**: Draft — structural verification contracts only; physical gate contracts unchanged.

## Contract Summary

- **Structural contract**: `VERIF-EMULATOR` verifies framework-level contracts of `003-reference-device` using Android SDK AVD (API 29+). It does NOT substitute physical `gate-8-full`.
- **Reference contracts** (existing, not modified):
  - `specs/003-reference-device/contracts/status-surface.md` (S1–S7): Notification rules; `VERIF-EMULATOR` confirms framework delivers notification with title + section + progress; excludes rate/voice/elapsed/remaining.
  - `specs/003-reference-device/contracts/earcon-mapping.md` (E1–E7): Earcon confirmation mappings; `VERIF-EMULATOR` confirms framework provides earcon hook; does not verify audio rendering.
  - `specs/003-reference-device/contracts/surface-audit.md` (A1–A4): Inventory / audit rules; `VERIF-EMULATOR` confirms inventory can be audited structurally.
- **Physical gate contracts** (unchanged; not covered by `VERIF-EMULATOR`):
  - `specs/001-android-media-shell/plan.md` `gate-8-full` (requires physical device; `SC-009` ≤3 s play-to-audio).
  - `verify.sh` rules (gates 1–8) remain physical-device-only.

## Verification Rules (for attestation)

- **Verified (structural)**: Manifest declaration (`mediaPlayback` FGS), `Service` lifecycle, `MediaSession` initialization, notification content limits, focus API response (`MAY_DUCK` / permanent loss), diagnostic JSONL write/read, resume fingerprint consistency, rate-step UI presence.
- Note: 8 structural contracts verified; aligns with tasks.md T007 reference (8 contracts mapped). Contract names: manifest, Service, MediaSession, notification, focus, diag, resume fingerprint, rate-step UI.
- **Explicitly excluded (physical-only)**: Audio-output verification (`22050 Hz` mono f32 → device format), `SC-009` benchmark, real `MAY_DUCK` audio volume (~20%), telephony/alarm interruption simulation, Bluetooth/car routing (`FR` device switch), 10/10 on-device trials.
- **No false-equivalence claim permitted**: Attestation must state both verified and excluded items; must link to `verify.sh` for excluded items.

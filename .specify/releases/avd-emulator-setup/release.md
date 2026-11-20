# Release (R7): 011 AVD Emulator Setup for mdreader/Ressoa

- **Date**: 2026-11-20
- **Version**: v0.1.0 infrastructure/tooling release (no app binary, no SemVer app bump)
- **Validation**: R4 PARTIAL PASS (`.specify/validations/avd-emulator-setup/validate.md`)

## Summary

Documents and verifies the local Android emulator harness for mdreader/Ressoa: `mdreader_avd` (API 33 x86_64), SDK/KVM/ADB prerequisites, stable startup flags, and the procedural bridge to spec 008 structural verification with explicit SC-009 caveats. No product source changes.

## Changelog (IDs)

- Added: FR-001..FR-008, SC-001..006; config/startup/debug-hook notes (`avd-config-review.md`, `emulator-startup-notes.md`, `research-debug-hooks.md`), assessment `avd-emulator-setup` (5 files), validation and release artifacts.
- Changed: docs only — `quickstart.md`, spec task ledger status; T001–T009 complete (SDK/KVM/AVD/ADB verification, config review, startup attempts).
- Removed: none.
- Open: T010–T023 (ABI-aware build/install/launch, spec 008 structural flows, VERIF-EMULATOR attestation, docs polish); full boot blocked in container without `/dev/kvm`.

## Rollback

Delete the added assessment/validation/release docs and revert any quickstart edits; no app artifact, manifest or persisted state is touched, so rollback is documentation-only.

## Residual risk / [NV]

- Full boot on a KVM-capable host, artifact build/install and structural verification remain environment-gated; emulator SC-009 measurements must carry device-class caveats and are never treated as reference-grade.

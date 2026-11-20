# Release (R7): 012 Debug-Only Ingest Validation Hook

- **Date**: 2026-11-20
- **Version**: v0.1.0 validation-tooling hardening release (no app SemVer bump; debug-only surfaces)
- **Validation**: R4 PARTIAL PASS (`.specify/validations/debug-ingest-hook/validate.md`)

## Summary

Hardens the debug/test-only automation seam that validates Spec 009 T184 and Spec 010 T226 ingest behavior through the production `DocIngest`/JNI path, with parseable outcomes, FR-008 evidence, durable canonical overlays, release non-exposure checks and a strict split between programmatic coverage and native SAF/a11y residuals.

## Changelog (IDs)

- Added: FR-001..FR-013, NFR-012-MAINT-001..005, SC-001..006; canonical overlay root `android-overlays/app/src/` (FR-011), release-exposure check (FR-010), residual split (FR-012), 4 contracts, assessment `debug-ingest-hook` (5 files), validation/release artifacts.
- Changed: `android-shell` debug hook surfaces, scripts/checks/docs/tasks; task ledger 54/69 checked; T184-PROG/T226-PROG evidence remains `full_parent_completion_claimed=false`.
- Removed: none from production; no hook exposure in release artifacts.
- Open: T050 (FR-010/SC-004 `mode=full`), T049/T067 device-run evidence wiring, other opt-in verification tasks per `converge.md`.

## Rollback

Remove the added canonical overlays/debug hook sources and scripts/checks/contracts/docs from git; re-run overlay `check` and release-exposure verification. Generated `src-tauri/gen/**` targets are never canonical — do not hand-edit them; regenerate after rollback.

## Residual risk / [NV]

- `mode=full` release evidence and complete FR-008 device-run classification are pending; partial/static/runtime-only checks never satisfy FR-010/SC-004. Native SAF picker cancel and TalkBack/keyboard traversal remain tracked in specs 009/010.

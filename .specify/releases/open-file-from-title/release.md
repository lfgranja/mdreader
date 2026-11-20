# Release (R7): 009 Abrir Arquivo Pelo Título e Iniciar Reprodução

- **Date**: 2026-11-20
- **Version**: v0.1.0 feature release (Android SAF/JNI ingest path)
- **Validation**: R4 PARTIAL PASS (`.specify/validations/open-file-from-title/validate.md`)

## Summary

Makes the `.meta-box` title block an accessible interactive control that opens the native SAF document picker and immediately starts playback for new `.md`/`.txt` files, while preserving the 3 s resume-offer dialog for deliberately paused documents and isolating the active transport on cancel/refusal.

## Changelog (IDs)

- Added: FR-001..FR-017, SC-001..004; `BridgeAction::PickDocument`, `DocIngest.pickDocument`, `pick_document_from_shell` JNI dispatch (T158–T161); title-box markup/listeners and focus restoration (T166–T167); refusal/cancel hardening (T172–T176).
- Changed: `status_view.html`, `android-shell` Kotlin, `src-tauri/src/lib.rs`/`shell_jni.rs`; Blake3-256 document identity and private snapshots.
- Removed: dependency on external automation for opening user documents.
- Residuals: T184 (full on-device scenarios 1–4), T184-RES-SAF-CANCEL, T184-RES-A11Y remain open; T184-PROG is non-UI only (`full_parent_completion_claimed=false`).
- Task IDs: suffixed T184-* subtasks de-checkboxed in `tasks.md` to remove duplicate T184 IDs; wording preserved.

## Rollback

Revert the commit range (e.g. `bb271cf1`, `703066b7`, `2ca2c8af`, `8b1a14a3`); restore previous `status_view.html`, Kotlin/JNI bridge files and contracts from git; re-run gates 1–8. No persisted bookmark migration to undo (Blake3 identity unchanged).

## Residual risk / [NV]

- Native SAF cancel/back and TalkBack/keyboard traversal not verified in this artifact; live CI/PR merge [NV].

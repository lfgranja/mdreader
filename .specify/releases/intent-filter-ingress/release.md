# Release (R7): 019 Android Intent Filter Ingress

- **Date**: 2026-11-20 (artifact backfilled)
- **Version**: v0.1.0 shell ingress feature (no app SemVer bump recorded in spec)
- **Validation**: R4 documentary PASS; device/gates [NV] (`.specify/validations/intent-filter-ingress/validate.md`)

## Summary

Registers Ressoa as an Android handler for `ACTION_VIEW` and `ACTION_SEND` of Markdown/text content (`content://`, `file://`, `text/markdown`, `text/plain`, `text/x-markdown`, `*.md`). The shell resolves a safe display name and streams raw UTF-8 bytes through JNI into the existing Rust ingest/fingerprint/session path; Kotlin remains a pure transport with no playback logic.

## Changelog (IDs)

- Added: FR-INT-001..007; manifest intent filters; `handleIncomingIntent` / `processIntentUri` / `processIntentText` cold-start (`onCreate`) and warm-start (`onNewIntent`) handling; `OpenableColumns.DISPLAY_NAME` resolution with safe fallback.
- Changed: `MainActivity`/`DocIngest` ingress entry points; JNI handoff to the existing `open_document_from_shell` command.
- Removed: none.

## Rollback

Revert the feature commit: remove the `<intent-filter>` declarations and the inbound-intent handling in `MainActivity`/`DocIngest`. The underlying Rust ingest path is unchanged, so no state migration or remote configuration rollback is required.

## Residual risk / [NV]

- Device-specific intent resolution and share-sheet behavior across vendor file managers/editors [NV].
- Hot-path replacement of an active playback must be confirmed on device (AC-3).

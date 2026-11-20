# Release (R7): 021 Android 16 Accessibility Compliance

- **Date**: 2026-11-20 (artifact backfilled)
- **Version**: verification record — no app SemVer bump, no code change
- **Validation**: R4 static PASS; gate 9 [NV] (`.specify/validations/android-16-accessibility-compliance/validate.md`)

## Summary

Confirms Ressoa's Android shell already satisfies the ROADMAP phase-NEXT Android 16 (API 36) accessibility item: `compileSdk/targetSdk = 36`, no legacy `announceForAccessibility`/`TYPE_ANNOUNCEMENT` calls in the 10 Kotlin shell files, and an accessible `aria-live`/ARIA frontend surface with prior TalkBack 10/10 attestation. No migration code was necessary or added.

## Changelog (IDs)

- Added: SDD artifacts (spec/plan/tasks/checklist/assessment/validate/release) under `specs/021-android-16-accessibility-compliance/` and `.specify/`.
- Verified: `build.gradle.kts` (compileSdk 36, targetSdk 36), Kotlin shell audit (zero legacy refs), `status_view.html` (`aria-live="polite"`, ARIA attributes).
- Removed: none. No manifest, code, permission or remote configuration change.

## Rollback

No production change to revert; removing the added documentation artifacts restores the prior doc state. No state migration or remote action.

## Residual risk / [NV]

- Gate 9 Android debug build with targetSdk 36 [NV] — gate owner.
- On-device TalkBack behavior not re-certified here; Spec 005 attestation remains the reference.

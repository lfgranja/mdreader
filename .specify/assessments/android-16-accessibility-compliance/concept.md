# Concept: Android 16 Accessibility Compliance (021)

- **Slug**: android-16-accessibility-compliance
- **Created**: 2026-11-20
- **Recommended option**: Option A (verification record, no code change)

## Options

### Option A — Static verification + compliance record (recommended)
- **Sketch**: Audit the 10 Kotlin files, build config and frontend surface; record measured evidence and the "already compliant" decision in a spec; leave gates to the gate owner.
- **Appetite**: small (docs only)
- **Trade-offs**: Closes the ROADMAP item with reproducible evidence and zero churn. Cost: gate 9 remains [NV] here; mitigated by explicit marker.
- **Rabbit holes**: adding unnecessary `setAccessibilityPaneTitle` calls (would violate mirror-only SC-035); confusing generated-manifest default with the real `build.gradle.kts` setting.

### Option B — Add explicit `setAccessibilityPaneTitle`/`setAccessibilityLiveRegion` calls anyway
- **Sketch**: Introduce local announcement logic in Kotlin shell.
- **Appetite**: medium
- **Trade-offs**: Diverges from SC-035 (no playback/announcement logic in Kotlin), adds untested behavior; rejected.

### Option C — Open a bug and defer
- **Sketch**: Treat as a defect and schedule later.
- **Appetite**: none
- **Trade-offs**: No defect exists; would create false debt. Rejected.

## Recommendation

Option A. Evidence shows the requirement is already satisfied; the correct artifact is a verification record (spec + evidence), not a migration.

## Out of Scope (recommended option)

- New accessibility code, manifest edits, bug creation, device TalkBack re-certification.

## Assumptions to Validate

- Static grep over the Kotlin directory is a sufficient proxy for absence of legacy announcement calls (measured: no matches).
- `build.gradle.kts` is the effective source of `targetSdk` for the generated app (measured: `targetSdk = 36`).

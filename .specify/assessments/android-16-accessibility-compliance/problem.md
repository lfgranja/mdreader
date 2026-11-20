# Problem Definition: Android 16 Accessibility Compliance (021)

- **Slug**: android-16-accessibility-compliance
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

ROADMAP.md (fase NEXT) calls for migrating residual `announceForAccessibility`/`TYPE_ANNOUNCEMENT` calls to `setAccessibilityPaneTitle`/`setAccessibilityLiveRegion` for Android 16 (API 36). Before scheduling migration work, we must verify whether such calls exist and whether the build/surface already satisfies API-36 accessibility expectations. The risk is unnecessary migration churn (or a missed regression) if the state is assumed rather than measured.

## Affected Users & Stakeholders

- **Users**: TalkBack users relying on live-region/pane announcements during playback.
- **Stakeholders**: maintainer (mirror-only shell contract, build config), QA/gate owner (gate 9), product (ROADMAP closure).

## Goals

- Verify `targetSdk = 36` / `compileSdk = 36` in the Android build configuration.
- Verify zero `announceForAccessibility`/`TYPE_ANNOUNCEMENT` (and no extra local announcement logic) in the Kotlin shell.
- Verify the frontend keeps `aria-live="polite"` and ARIA attributes.
- Record the compliance decision without creating migration code.

## Non-Goals

- No new accessibility/migration code (nothing to migrate).
- No manifest change (targetSdk is set via build.gradle.kts).
- No new bug record (no defect found); the spec is the verification record.
- No changes to the mirror-only Kotlin contract or Rust authority.

## Success Metrics

- `targetSdk = 36` confirmed in `build.gradle.kts` (measured).
- Zero legacy announcement references in Kotlin (measured).
- Frontend `aria-live` + ARIA preserved (measured).
- Gate 9 Android debug build with targetSdk 36 [NV] — gate owner.

## Cost of Inaction

ROADMAP item stays ambiguous; a future contributor might add migration code against a mirror-only contract, or an actual regression could go unnoticed.

## Open Questions

- Gate 9 result [NV]; no device TalkBack re-check in this verification.

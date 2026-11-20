# Validation (R4): 021 Android 16 Accessibility Compliance

- **Date**: 2026-11-20
- **Status**: static verification PASS (measured); gate 9 [NV] (gate owner)

| Check | Command / Evidence | Result |
|---|---|---|
| Spec quality | `specs/021-android-16-accessibility-compliance/checklists/requirements.md` | PASS |
| Assessment gate | `.specify/assessments/android-16-accessibility-compliance/decision.md` verdict `go` (verification-only) | PASS |
| Kotlin legacy APIs | `grep -rnE 'announceForAccessibility|TYPE_ANNOUNCEMENT|AccessibilityPaneTitle|AccessibilityLiveRegion' android-shell/src/main/kotlin/app/mdreader/` → no matches | PASS (measured) |
| Build config | `src-tauri/gen/android/app/build.gradle.kts:12` `compileSdk = 36`; `:18` `targetSdk = 36` (`:17` `minSdk = 29`) | PASS (measured) |
| Frontend a11y | `src-tauri/src/status_view.html:504` `role="status" aria-live="polite" aria-atomic="true"`; other ARIA attributes present | PASS (measured) |
| TalkBack | 10/10 attested in Spec 005 (recorded, not re-run) | PASS (recorded) |
| No new permission | Manifest: no `INTERNET` (per spec §2.2) | PASS (spec documented) |
| No code change | verificação apenas; spec/plan/tasks/checklist/assessment/validate/release | PASS |
| Gate 9 (Android debug build) | `scripts/verify.sh` gate 9 | [NV] — gate owner |
| Host gates 1–8 | no production change | N/A |

## Success criteria

- targetSdk = 36 confirmado em `build.gradle.kts` — PASS (measured).
- Zero referências a `announceForAccessibility`/`TYPE_ANNOUNCEMENT` no shell Kotlin — PASS (measured).
- Frontend mantém `aria-live` e atributos ARIA — PASS (measured).
- Gate 9 passa com targetSdk 36 — [NV].

## [NV]

- Gate 9 Android debug build and any on-device TalkBack re-check; no result asserted here.

# Idea Intake: Android 16 Accessibility Compliance (021)

- **Slug**: android-16-accessibility-compliance
- **Created**: 2026-11-20
- **Source**: `specs/021-android-16-accessibility-compliance/spec.md`, `ROADMAP.md` (fase NEXT, item 1), `specs/021-android-16-accessibility-compliance/plan.md`
- **Type**: verification/compliance record (no code change expected)

## Idea (as captured)

> "Verificar a conformidade do shell Android do Ressoa com as exigências de acessibilidade do Android 16 (API 36), especificamente a migração de chamadas residuais de `announceForAccessibility`/`TYPE_ANNOUNCEMENT` para `setAccessibilityPaneTitle`/`setAccessibilityLiveRegion` (conforme ROADMAP.md, fase NEXT)." — `specs/021-android-16-accessibility-compliance/spec.md:5`

> "**Não há chamadas `announceForAccessibility`/`TYPE_ANNOUNCEMENT` para migrar.** O requisito do ROADMAP.md (fase NEXT, item 1) já está satisfeito" — `spec.md` §3

## Restated

Audit the Android shell, build configuration and frontend surface to confirm Android 16 (API 36) accessibility compliance, rather than implementing a migration. Expected outcome is a verification record confirming `targetSdk = 36`, zero legacy announcement calls in Kotlin, and an accessible `aria-live`/ARIA frontend surface (TalkBack already attested in Spec 005).

## Origin & Context

- **Raised by**: ROADMAP.md phase NEXT item 1 (accessibility API migration).
- **Trigger**: need to confirm whether any legacy announcement API remains before planning migration work.
- **Preliminary evidence**: all 10 Kotlin shell files read; none contain the target APIs (spec §2.1).

## First-Glance Unknowns

- [NEEDS CLARIFICATION: whether `targetSdk` is set in the generated manifest or in `build.gradle.kts` — resolved in research: `build.gradle.kts` (compileSdk 36, targetSdk 36)]
- [NEEDS CLARIFICATION: whether gate 9 Android debug build passes with targetSdk 36 — left as [NV] for the gate owner]

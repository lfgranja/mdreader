# Decision: Android 16 Accessibility Compliance (021)

- **Slug**: android-16-accessibility-compliance
- **Decided**: 2026-11-20
- **Verdict**: go (verification-only; requirement already satisfied, no code change)
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | ROADMAP item needs closure; assumption vs measurement risk |
| Evidence strength | strong | Direct static audit: no legacy APIs in 10 Kotlin files; `targetSdk = 36`; `aria-live` present |
| Value vs. inaction | strong | Closes ROADMAP item without churn; protects mirror-only contract |
| Feasibility / appetite | strong | Documentation only |
| Strategic fit | strong | No SC-035/Rust-authority drift; no new permissions |
| Risk posture | strong | Reversible; gate 9 explicitly [NV] |

## Verdict & Rationale

`go` for Option A: produce the verification record and close the ROADMAP item. No `announceForAccessibility`/`TYPE_ANNOUNCEMENT` calls exist to migrate; `build.gradle.kts` sets compileSdk/targetSdk 36; the frontend keeps `aria-live="polite"` and ARIA attributes; TalkBack 10/10 was already attested in Spec 005. Do **not** add migration code — doing so would violate the mirror-only contract.

## If go — Handoff to `/speckit-specify`

- **Problem**: ROADMAP API-36 accessibility item unverified.
- **Chosen approach**: Option A — static verification record; no code changes.
- **In scope**: spec/plan/tasks/checklist/assessment/validate/release artifacts; measured evidence.
- **Out of scope**: migration code, manifest edits, new bug, device re-certification.
- **Success metrics**: targetSdk 36 ✅, zero legacy calls ✅, aria-live/ARIA ✅, gate 9 [NV].
- **Carried-forward questions**: gate 9 execution belongs to the gate owner.

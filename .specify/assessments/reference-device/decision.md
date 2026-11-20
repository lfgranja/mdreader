# Decision: Reference Device Realignment

- **Slug**: reference-device
- **Decided**: 2026-11-20 (backfilled; original maintainer decision 2026-10-11)
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Two specs assert two reference devices; gates unmeasurable |
| Evidence strength | strong | File/line contracts A–D, commit `d5d6b45`, grep-verifiable RED→GREEN |
| Value vs. inaction | strong | Restores single authority for every downstream on-device gate |
| Feasibility / appetite | strong | Option A docs-only, 4 sites, T001–T012 all `[X]` |
| Strategic fit | strong | Matches constitution single-authority rule and accepted 002 decision |
| Risk posture | strong | Reversible; history preserved by append-only discipline |

## Verdict & Rationale

`go`. Option A is the minimal, evidence-backed correction with no functional risk.

## If go — Handoff to `/speckit-specify`

- **Problem**: A78-class text in 001 drifts from the amended 002 Redmi/≤10 s authority.
- **Chosen approach**: Site A/B amendment + Site C closure note + Site D amendment commit.
- **In scope**: FR-001..004, SC-001..003, contract Sites A–D, docs-only (T001–T012).
- **Out of scope**: code/gates, rewriting historical measurements, new device classes.
- **Success metrics**: SC-001 one-device grep; SC-002 re-judgment recorded; SC-003 no hypothetical class; Site D commit message check.
- **Carried-forward questions**: none blocking; device trials [NV].

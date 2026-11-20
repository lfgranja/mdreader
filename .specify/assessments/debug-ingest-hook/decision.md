# Decision: Debug-Only Ingest Validation Hook

- **Slug**: debug-ingest-hook
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Manual SAF-based T184/T226 validation is fragile; gaps in evidence/durability/release proof are concrete and tracked (problem.md, converge.md). |
| Evidence strength | adequate | Code paths, host contract suites and partial device runs are recorded; official external guidance was recorded earlier but not live re-fetched; FR-010 `mode=full` still pending. |
| Value vs. inaction | strong | Repeatable production-path ingest evidence at low cost while leaving UI residuals explicit. |
| Feasibility / appetite | strong | Option A already implemented/exercised; remaining work is hardening and verification, not speculative design. |
| Strategic fit | adequate | Fits evidence-backed validation priorities; must respect Android release security and no-overclaim rules. |
| Risk posture | adequate | Risks (debug backdoor, gen drift, overclaim) have named mitigations: debug guards, canonical overlays, split residuals; residual verification tasks remain open. |

## Verdict & Rationale

`go` for Option A, narrowly scoped. This records the same decision as the earlier `debug-ingest-hook-automation` assessment for the spec-named slug. It does not authorize release exposure, a parallel ingest path, or full T184/T226 completion from debug-hook evidence. Pending FR-010 `mode=full` and device-wiring tasks are tracked verification work, not design changes.

## If go — Handoff to `/speckit-specify`

- **Problem**: T184/T226 ingest validation is manual/fragile and programmatic evidence lacks durability and release-proof boundaries.
- **Chosen approach**: Option A — debug-only seam through production `DocIngest`/JNI with FR-008 evidence, durable canonical overlays, combined release-exposure evidence, and split residual tracking.
- **In scope**: FR-001..FR-013, SC-001..006; overlay apply/check, release-exposure check, evidence grammar/channels, T184/T226 split.
- **Out of scope**: release hook exposure, parallel ingest path, SAF picker UI automation, playback authority changes, gate 9 parity claims.
- **Success metrics**: SC-001..006; 54/69 tasks already complete; T050 (`mode=full`) and T049/T067 wiring to be finished where environment permits.
- **Carried-forward questions**: FR-010 full evidence inputs; complete FR-008 device-run classification; residual UI/a11y statuses remain independently tracked in specs 009/010.

# Decision: Android Media Shell

- **Slug**: android-media-shell
- **Decided**: 2026-11-20 (backfilled; original feature started 2026-10-01)
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | No playback substrate blocks specs 002/004/005 and all driving trials |
| Evidence strength | strong | FR/SC contracts + host gates 1–8, SC-035 authority census, SC-057 closed-set enumeration |
| Value vs. inaction | strong | Enables the entire driving-first product |
| Feasibility / appetite | strong | Option A bounded to shell-only (FR-024/FR-025 excluded); host-verifiable |
| Strategic fit | strong | Single-authority rule, offline-only, closed sets match the constitution |
| Risk posture | medium | Device-only trials [NV]; Android 14+ FGS restrictions require device confirmation |

## Verdict & Rationale

`go`. Option A is the only design consistent with FR-002/FR-003 and SC-035; residual risk is confined to device trials and is named rather than assumed.

## If go — Handoff to `/speckit-specify`

- **Problem**: no single-authority PCM playback shell for locked, screen-off driving use.
- **Chosen approach**: Rust player owns playback/position/buffering; Kotlin MediaSession mirror; FGS + always-resident component; focus/device/resume closed sets.
- **In scope**: FR-001..045 (shell only), SC-001..057, gates 1–8, host census.
- **Out of scope**: synthesis (002), Android Auto (FR-025), network (FR-026), active engine fallback (SC-044 DEFERRED).
- **Success metrics**: SC-001/SC-002/SC-003/SC-004 10/10; SC-006 automated FGS check; SC-007/SC-011/SC-012 recovery; SC-035 zero surface playback logic; SC-057 closed sets enumerated.
- **Carried-forward questions**: device trials [NV]; 11 open follow-up tasks (incl. T232/T233) tracked in `tasks.md`.

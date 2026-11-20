# Problem Definition: Reference Device Realignment

- **Slug**: reference-device
- **Created**: 2026-11-20 (backfilled)
- **Inputs used**: intake.md | research.md

## Problem Statement

`001/plan.md` still defines the reference device as a hypothetical 4× Cortex-A78 / 6 GB class, while `002/spec.md` (amendment `d5d6b45`) names the maintainer's Redmi 2201117TY with a ≤10 s cold-start bound. The repo therefore asserts two authorities; on-device verdicts against the A78 class are unmeasurable or dismissible, and the 002 divergence note remains open.

## Affected Users & Stakeholders

- **Users**: maintainer running 10/10-trial gates — needs a verdict that counts without a device-class asterisk.
- **Stakeholders**: all downstream specs (004–021) that inherit the reference-device definition; constitution's single-authority rule.

## Goals

- FR-001: `001/plan.md` Reference-device paragraph names Redmi 2201117TY (Snapdragon 680, 4× A73 + 4× A53, ~5.8 GB, API 33) for FR-027/FR-019c and all 10/10 criteria.
- FR-002: both device-coupled bounds → ≤10 s cold-start; device-independent bounds untouched.
- FR-003: amendment recorded with acceptance, date, rationale and preserved history.
- FR-004: 002 "Divergence noted" annotation closed by follow-up note, history intact.

## Non-Goals

- No re-judgment of unrelated past measurements; no editing of `SC-009-GATE-RECORD.md` rows.
- No code, gate script, dependency or constitution change (T012).

## Success Metrics

- SC-001: one grep-confirmed device story across both specs.
- SC-002: 9 s × 5 result re-judged against ≤10 s and recorded un-asterisked.
- SC-003: no hypothetical device class remains in the reference-device definition.
- Contract Sites A–D verified, commit carries acceptance/date/rationale.

## Cost of Inaction

Two drifting authorities; every on-device gate remains decorative and cross-spec comparisons invalid.

## Open Questions

- None blocking. Device trials [NV] belong to the gate owner.

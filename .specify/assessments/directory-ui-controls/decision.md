# Decision: Directory UI Controls

- **Slug**: directory-ui-controls
- **Decided**: 2026-11-18
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Open follow-ups T229/T230/T234 in spec 010; bridge actions tested but unreachable |
| Evidence strength | strong | File/line evidence for buttons, listener, tests, bundle hashes — all measured |
| Value vs. inaction | strong | Makes P1 directory reading usable from the surface |
| Feasibility / appetite | strong | Option A is small, one HTML file + copies + structural tests |
| Strategic fit | strong | Rust authority untouched; mirrors-only UI; matches constitution |
| Risk posture | strong | Reversible; a11y invariants pinned by deterministic tests |

## Verdict & Rationale

`go`. Option A satisfies the 80/20 choice with high evidence and low residual risk.

## If go — Handoff to `/speckit-specify`

- **Problem**: bridge actions exist but the surface has no directory/chapter controls.
- **Chosen approach**: Option A — wire existing actions, add `directory-selected` listener, toggle chapter nav via `hidden`.
- **In scope**: five HTML bundles, structural tests, spec 010 follow-up status.
- **Out of scope**: T232/T233, Rust changes, new tokens/aria labels.
- **Success metrics**: structural tests pass; bundles byte-identical; gates/CI green.
- **Carried-forward questions**: none blocking.

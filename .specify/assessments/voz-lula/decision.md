# Decision: 016-voz-lula

- **Slug**: voz-lula
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Explicit product requirement 2026-11-08; spec/plan/tasks exist |
| Evidence strength | medium | Clear seam/format evidence; model/legal path unresolved (research.md) |
| Value vs. inaction | medium | Cultural differentiator; core UVP independent of it |
| Feasibility / appetite | medium | Adapter code small; model acquisition blocked externally |
| Strategic fit | mixed | Offline-first fit, but conflicts with single-voice constitution gate (needs amendment) |
| Risk posture | weak | HIGH LGPD/STJ/TSE/store-policy exposure for a real sitting public figure |

## Verdict & Rationale

`go` **for specification** — the idea merits a complete SDD record and a designed, fail-closed selection seam (Option A). This verdict authorizes `/speckit-specify`, which produced `specs/016-voz-lula/{spec,plan,tasks}.md`.

**Scope of this go**: design artifacts only. Implementation is a **separate, deferred decision** recorded in `specs/016-voz-lula/decision.md` (2026-11-10, verdict `defer`) pending: (1) documented consent or a fictional CC0 voice, (2) licence + provenance + legal review before bundling any model, (3) a constitution amendment + `test_noscope` gate update, and (4) synthetic-content store disclosure. No code or asset changes are authorized by this assessment.

## If go — Handoff to `/speckit-specify`

- **Problem**: single default voice; no selectable pt-BR homage voice with fail-closed init.
- **Chosen approach**: Option A — `Voice` enum, `--voice`/`set_voice`, canonical 22 050 Hz mono f32 handoff, `InitFailed` when `.onnx` absent.
- **In scope**: FR-01..05, SC-001..005; design artifacts only.
- **Out of scope**: model training/donation, consent, constitution/gate change, asset bundling.
- **Success metrics**: SC-001..004 build/test/clippy/fmt; SC-005 fail-closed `[NV]` (implementation deferred).
- **Carried-forward questions**: Q1 model provenance, Q2 espeak-ng tree reuse; legal/constitution prerequisites.

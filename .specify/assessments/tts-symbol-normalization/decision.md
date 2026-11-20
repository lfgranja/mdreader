# Decision: TTS Symbol Normalization

- **Slug**: tts-symbol-normalization
- **Decided**: 2026-11-20
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Listening-fatigue report 2026-11-07; existing `tts-symbol-fatigue` assessment = go |
| Evidence strength | strong | File/test evidence: `synth.rs` boundary, 17 normalize + 8 invariant tests, 170-pass adapter run — all measured |
| Value vs. inaction | strong | Directly improves the app's core TTS function |
| Feasibility / appetite | strong | Option A small–medium, pure Rust, already implemented/hardened |
| Strategic fit | strong | Offline-first, single engine, metadata/fingerprint contracts preserved |
| Risk posture | strong | Reversible; protected zones and invariants pinned by deterministic tests |

## Verdict & Rationale

`go`. Option A delivers deterministic, host-verifiable marker removal with no engine or metadata changes; residual risk is bounded by invariant/property tests.

## If go — Handoff to `/speckit-specify`

- **Problem**: verbatim Markdown markers spoken literally by eSpeak/Piper cause listener fatigue.
- **Chosen approach**: Option A — `normalize_for_speech` at the `synth.rs::synthesize` boundary, Fluid default with protected zones.
- **In scope**: FR-SYM-001..009, SC-SYM-001..005; `normalize.rs` + `synth.rs` wiring + invariant tests.
- **Out of scope**: SSML, engine swap, AST parser, full math/HTML, `.txt` full normalization, UI toggle.
- **Success metrics**: SC-SYM-001..004 host PASS; SC-SYM-005 device listening `[NV]`.
- **Carried-forward questions**: Literal-mode UI toggle (follow-up).

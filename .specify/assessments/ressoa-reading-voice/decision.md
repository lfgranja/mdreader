# Decision: Ressoa Reading Voice (018)

- **Slug**: ressoa-reading-voice
- **Decided**: 2026-11-10 (artifact backfilled 2026-11-20)
- **Verdict**: go (conditioned MVP — requester Option B)
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Spec §1 records single hardcoded voice and identity mismatch; shell already has cycle verbs with no catalog |
| Evidence strength | strong | Repo-declared constraints (pinned sherpa, handoff, CC0 upstream, gates) with file/section citations |
| Value vs. inaction | strong | Expressions the Ressoa identity in the audio actually heard; adds durable voice extension seam |
| Feasibility / appetite | medium | Multi-crate additive change; faber-only contingency bounds the risk |
| Strategic fit | strong | Rust authority, fail-closed, offline/zero-INTERNET, closed sets all preserved |
| Risk posture | strong | Reversible additive change; named-error fail-closed, deterministic catalog, host gates |

## Verdict & Rationale

`go`, conditioned on the MVP scope: ship a closed curated catalog through the same Rust path, keep `pt_BR-faber-medium` as default, and fall back to a documented tuned-faber-only catalog if no alternative passes quality/runtime/packaging validation. Non-imitative identity and provenance are hard constraints, not stretch goals.

## If go — Handoff to `/speckit-specify`

- **Problem**: no curated offline reading voice choice or catalog; identity unexpressed in audio.
- **Chosen approach**: Option A — curated closed catalog + additive multi-voice plumbing (FR-018-001..011), with Option C contingency.
- **In scope**: catalog assets/provenance, `adapter-synthesis` runtime/build/CLI, `crates/session` persistence, `src-tauri` diagnose/inventory/get/set/cycle with safe playback restart, tests and gates.
- **Out of scope**: imitation/cloning, network/downloaded voices, SSML/tier changes, major UI redesign.
- **Success metrics**: SC-018-001..005; host gates 1–8 green, expected-tests consistent; device evidence when available.
- **Carried-forward questions**: alternative-voice qualification and engine instance strategy — resolved during implementation with recorded evidence.

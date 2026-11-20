# Idea Intake: Ressoa Reading Voice (018)

- **Slug**: ressoa-reading-voice
- **Created**: 2026-11-10 (artifact backfilled 2026-11-20)
- **Source**: `specs/018-ressoa-reading-voice/spec.md`, `specs/018-ressoa-reading-voice/plan.md`, `DESIGN-RESSOA/`
- **Type**: feature — curated offline reading-voice catalog

## Idea (as captured)

> "Ressoa/mdreader's main design system (`DESIGN-RESSOA/`) defines a Ressoa, serene and musical 'Engine Ressoa' identity ... but reading TTS remains a single hardcoded bundled voice (`pt_BR-faber-medium`, Piper VITS via sherpa-onnx). There is no curated reading-voice profile aligned to that identity, no closed voice catalog, and no deterministic selection/cycle/persistence across the synthesis runtime, CLI, session store, and Tauri bridge." — `specs/018-ressoa-reading-voice/spec.md:9-12`

> "This feature delivers **requester Option (B)**: a curated/offline pt-BR TTS reading voice selected and tuned to evoke serenity/musicality with a **harmonious, melodic, calm, soft reading voice** aligned to `DESIGN-RESSOA/` ... implemented as a bundled alternative voice + tuning with evolution-ready multi-voice plumbing. It is explicitly **non-imitative**" — `specs/018-ressoa-reading-voice/spec.md:14`

## Restated

Ship a small, closed, offline pt-BR voice catalog (default `pt_BR-faber-medium` + at least one curated, CC0-provenanced 22050 Hz alternative), with deterministic list/select/cycle semantics wired through `adapter-synthesis`, CLI, session persistence and the Tauri bridge, while preserving the mono f32 @ 22,050 Hz handoff, fail-closed behavior, Rust authority and zero-`INTERNET` posture. Identity is thematic curation only — no third-party character/performer imitation.

## Origin & Context

- **Raised by**: requester Option (B) of the 018 design discussion, 2026-11-10.
- **Trigger**: the DESIGN-RESSOA "Engine Ressoa" identity has no corresponding reading-voice curation; the runtime has a single hardcoded voice.
- **Prior assessment reference**: `specs/018-ressoa-reading-voice/spec.md` records `.specify/assessments/ressoa-reading-voice/decision.md` (Verdict: GO, conditioned MVP) as its input; this artifact set reconstructs that assessed decision.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: whether a CC0 22050 Hz alternative actually passes quality/runtime/packaging validation — resolved in research: keep `faber`-only contingency with multi-voice-ready API if no alternative qualifies]
- [NEEDS CLARIFICATION: whether multiple `sherpa-onnx = "=1.13.8"` engine instances can be cached safely — deferred to implementation validation per FR-018-005]

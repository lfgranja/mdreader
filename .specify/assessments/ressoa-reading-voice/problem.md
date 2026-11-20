# Problem Definition: Ressoa Reading Voice (018)

- **Slug**: ressoa-reading-voice
- **Created**: 2026-11-10 (artifact backfilled 2026-11-20)
- **Inputs used**: intake.md | research.md

## Problem Statement

Ressoa/mdreader synthesizes long-form pt-BR text with a single hardcoded bundled voice (`pt_BR-faber-medium`) despite a design system that defines a serene, musical "Engine Ressoa" identity. There is no closed voice catalog, no deterministic/listable/selectable voice vocabulary shared by synthesis runtime, CLI, session store and Tauri bridge, and no provenance-gated path for adding a curated alternative. Users cannot choose or cycle a reading voice; maintainers cannot audit a catalog or safely evolve it.

## Affected Users & Stakeholders

- **Users**: pt-BR long-form readers who want a calm, melodic, soft reading voice and stable selection across restarts.
- **Stakeholders**: maintainer (audio contract, fail-closed and Rust authority invariants), reviewers/legal (provenance, licensing, non-imitative identity).

## Goals

- Define a closed, deterministic, bundled offline pt-BR voice catalog (default `pt_BR-faber-medium` + ≥1 validated CC0 22050 Hz alternative, or documented contingency).
- Expose voice list/select/cycle and persistence across `adapter-synthesis`, CLI, `crates/session` and `src-tauri` through the same Rust synthesis path.
- Preserve mono f32 @ 22,050 Hz handoff, fail-closed asset handling and zero `INTERNET`.
- Ship auditable per-voice provenance and a non-imitative Ressoa identity with recorded auditory-validation rationale.

## Non-Goals

- No third-party character/performer imitation, cloning, trademark or likeness use.
- No downloaded/cloud/marketplace/SAF-imported voices or any new network capability.
- No SSML/alignment capability expansion, no engine-tier expansion beyond `OfflineDefault`.
- No silent resample/downmix, no mixed-voice PCM splicing during playback.
- No major UI redesign beyond required voice controls/inventory plumbing.

## Success Metrics

- SC-018-001: catalog listable/selectable/cyclable deterministically via CLI and bridge with safe persistence/restore and default fallback.
- SC-018-002: every bundled voice conforms to mono f32 @ 22,050 Hz; missing/corrupt assets fail closed with named errors.
- SC-018-003: capabilities/tier/authority/position/resume/fingerprint invariants intact; host gates 1–8 green, expected-tests consistent.
- SC-018-004: auditable provenance/checksums for every voice; no third-party branding in shipped artifacts.
- SC-018-005: recorded auditory evidence for the calm/melodic/soft reading voice and default-choice rationale.

## Cost of Inaction

The Ressoa voice identity stays purely visual; readers keep a single uncurated voice and maintainers have no safe extension seam for offline voices.

## Open Questions

- Whether a CC0 alternative qualifies (contingency documented), and engine caching vs recreate-on-switch — resolved during implementation.

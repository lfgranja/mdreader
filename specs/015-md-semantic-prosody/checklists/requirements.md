# Specification Quality Checklist: MD Semantic Prosody

**Purpose**: Validate specification completeness and quality before planning/implementation
**Created**: 2026-11-20
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] Focused on user/system value (natural, human-like markdown reading without SSML)
- [x] Grounded in code/API evidence (only `GenerationConfig.speed` is per-segment in sherpa-onnx 1.13.8)
- [x] All mandatory sections completed (scenarios, FRs, non-goals, SCs)
- [x] Scope bounded to conservative 80/20 (speed map only; no pitch/volume/SSML/theatricality)

## Requirement Completeness

- [x] Requirements testable (prosody unit + invariant host tests; device listening optional)
- [x] Success criteria measurable (distinct heading/code speed factors; thematic-break pause; invariants)
- [x] Acceptance scenarios cover structural pauses, rate variation, protected zones
- [x] Edge cases identified (protected code monotone/slow; conservative defaults; `.txt` neutral)
- [x] Dependencies/assumptions identified (sherpa-onnx speed-only lever; coordination with 014 normalization)

## Feature Readiness

- [x] FR-PROS-001..008 map to trace evidence and assess decision (`go`, `.specify/assessments/md-semantic-prosody/`)
- [x] Pipeline order fixed: classify → plan → normalize → synthesize (structure derived before 014 normalization)
- [x] Risks/rollback constraints stated (conservative defaults; metadata invariants preserved)

## Implementation/Convergence Status (converge 2026-11-10, refined 2026-11-15, honest)

- [x] `prosody.rs` + wiring in `synth.rs` (per-segment `GenerationConfig.speed`) delivered.
- [x] 13 prosody unit tests + 4 invariant tests passing (host); fmt/clippy clean.
- [~] SC-PROS-001/002/003 host-evidenced; SC-PROS-004 device listening `[NV]`.
- [ ] Full host gates 1–8 / R4 / R7 not claimed at artifact creation.

## Notes

- Speed map: Normal/Question/Exclamation/ThematicBreak = 1.00; Emphasis/Quote/List = 0.95; Heading/Strong = 0.90; Code = 0.85.
- `prosody_invariants.rs` asserts deterministic plans and unchanged `source_range`/`segment_index`/`resume_point`.

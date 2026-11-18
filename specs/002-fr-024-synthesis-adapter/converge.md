# Converge: 002 FR-024 Synthesis Adapter

Status: **CONVERGED** — offline synthesis adapter (`sherpa-onnx` + Piper `pt_BR-faber-medium`) bound and attested; SC-044 remains deferred by design (80/89 tasks).

## Implementation Summary

Implemented the FR-024 synthesis adapter path: mono f32 @ 22050 Hz delivery contract, paragraph/heading chunking/segmentation alignment, closed-set failure mapping, and adapter capability declaration/registration for `offline_default`. SC-009 cold-start benchmark was executed on hardware with amended bound recorded; selector/invariant preservation (SC-051 mechanism) maintained with active multi-engine fallback deferred.

### Evidence (verified)
- `specs/002-fr-024-synthesis-adapter/tasks.md`: 80 checked / 9 unchecked (89 total) [verified in prior cycles].
- `specs/002-fr-024-synthesis-adapter/V3_ATTESTATION_2026-10-10.md`: adapter-scoped attestation — `cargo nextest -p adapter-synthesis` 110 passed, fmt/clippy/closed-set checks scoped green, checklists 148/148 [verified in prior cycles].
- `specs/002-fr-024-synthesis-adapter/SC-009-GATE-RECORD.md`: stage 2 benchmark executed on Redmi 2201117TY (9 s × 5) and recorded as PASS under amended ≤10 s bound per spec amendment history [verified in prior cycles].
- Git evidence: `830ec47a ... API_CONTRACT.md ...`, `42f8bfd5 docs(gate): SC-009 stage 2 executed on hardware, FAIL recorded` (historical pre-amendment context preserved in record), `16a933ba docs(002): record the independent final verification attestation ...` [verified in prior cycles].
- ROADMAP alignment: 80/89 tasks, CONVERGIDO (Attestation V3; benchmark SC-009).

### Gate 9 (Android build)
- Not separately re-run in this converge pass; Android integration/device packaging evidence is tracked at shell/integration feature level [unverified in this pass].

### Non-Changes
- No new benchmark, gate, or formatter runs performed in this converge pass (edit-only record).
- No changes to engine registry/tier policy beyond recorded spec/contract state.

### Unresolved / Deferred
- SC-044 (two real selectable engines / consent acquisition / active takeover): **DEFERRED** per spec/constitution records [verified in prior cycles].
- 9 unchecked tasks remain in `tasks.md` (consumer-wiring/verification residuals and 001/workspace-config-owned items such as T072–T075, T083–T085 as recorded) [unverified in this pass beyond task ledger].

### Acceptance Criteria Status
- [x] FR-024 adapter bound, format/segmentation/failure/capability declarations attested (adapter scope) [verified in prior cycles].
- [x] SC-009 benchmark evidence recorded under amended reference-device/bound context [verified in prior cycles].
- [ ] SC-044 multi-engine discharge (deferred).

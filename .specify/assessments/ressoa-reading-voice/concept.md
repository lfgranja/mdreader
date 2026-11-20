# Concept: Ressoa Reading Voice (018)

- **Slug**: ressoa-reading-voice
- **Created**: 2026-11-10 (artifact backfilled 2026-11-20)
- **Recommended option**: Option A (with documented faber-only contingency)

## Options

### Option A — Curated offline catalog, additive multi-voice-ready plumbing
- **Sketch**: Bundle a closed catalog (`pt_BR-faber-medium` default + validated CC0 alternative); generalize `adapter-synthesis` runtime/assets/build manifest/CLI, persist `last_voice` in `crates/session`, wire catalog-aware diagnose/inventory/get/set/cycle in `src-tauri`, and restart production at authoritative resume position on voice change. Per-voice provenance and tuning recorded.
- **Appetite**: medium (multiple crates + assets + tests)
- **Trade-offs**: Delivers the full user-facing value and a durable extension seam. Cost: APK/extraction/RAM growth and broader surface updates; mitigated by the fail-closed contract, deterministic catalog and host gates.
- **Rabbit holes**: runtime instance caching under pinned sherpa 1.13.8; per-voice phonemizer-data compatibility; mixed-voice PCM during cycle; provenance/license completeness.

### Option B — Cloud/streamed or downloaded voice
- **Sketch**: Fetch a premium/cloned voice at runtime.
- **Appetite**: large
- **Trade-offs**: Breaks offline/air-gapped posture (`INTERNET`), licensing and determinism; rejected outright.

### Option C — Tune the single `faber` voice only
- **Sketch**: Keep one voice, adjust inference defaults for a calmer reading style, no catalog/API work.
- **Appetite**: small
- **Trade-offs**: Cheap and low risk but no user choice and no evolution seam. Retained **only** as documented contingency if no alternative passes validation (FR-018-002), with the multi-voice-ready API still preserved.

## Recommendation

Option A, with Option C as the evidence-gated contingency. It matches the requester's Option (B), satisfies the assessment verdict (GO, conditioned MVP), and stays inside the constitution: Rust authority, unchanged handoff, fail-closed, zero network.

## Out of Scope (recommended option)

- Imitation/cloning of third-party voices; downloaded or cloud voices; new network permissions.
- SSML/alignment/tier changes; major UI redesign.

## Assumptions to Validate

- CC0 22050 Hz alternative available and compatible with `sherpa-onnx = "=1.13.8"` (validated in implementation; contingency if not).
- Multiple engine instances or safe recreate-on-switch feasible under pinned runtime (FR-018-005).
- Auditory comparison can be recorded as evidence for the calm/melodic/soft target (SC-018-005).

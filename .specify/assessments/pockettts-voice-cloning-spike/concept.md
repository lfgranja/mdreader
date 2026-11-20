# Concept: PocketTTS Voice Cloning Spike (020)

- **Slug**: pockettts-voice-cloning-spike
- **Created**: 2026-11-15 (artifact backfilled 2026-11-20)
- **Recommended option**: Option A (run the device-first measurement spike, then gate)

## Options

### Option A — Device-first measurement spike, then gate (recommended)
- **Sketch**: Execute PLAN-80-20 Phases 0–3: unblock device measurement, run a standalone C-API harness for PocketTTS pt with the Ressoa reference across steps/threads/cold-warm, record all metrics in `measurements.md`, then GO to Phase 2 integration only if TTFA/RTF/RSS pass; otherwise NO-GO with Piper fallbacks.
- **Appetite**: small–medium (harness + device time; no production code on the critical path)
- **Trade-offs**: Gives a defensible decision with minimal blast radius. Cost: device setup and a ~140 MB on-demand bundle; risk of bundle/runtime mismatch mitigated by the C-API harness. The measurement proves negative on SD680, so the outcome is a cheap, early NO-GO.
- **Rabbit holes**: comparing synthesis `first_chunk_ms` to end-to-end `play_to_audio_ms`; HWM vs RSS interpretation; termux-static CLI libc++ failure; license/consent assumptions.

### Option B — Integrate PocketTTS directly into `adapter-synthesis` first
- **Sketch**: Add the backend/embedding cache/streaming and measure in-app.
- **Appetite**: large
- **Trade-offs**: Highest cost before knowing feasibility; touches production contracts and risk of TTFA regression; rejected.

### Option C — Trust host/vendor benchmarks and decide without device data
- **Sketch**: Use the ~267–414 ms host figure and SD845 RTF ~1 as proof.
- **Appetite**: none
- **Trade-offs**: No evidence for SD680; explicitly rejected by the plan (`TTFA no SD680: NÃO VERIFICADO`).

## Recommendation

Option A. It is the cheapest way to answer the only question that matters (device feasibility under the 1551 ms / `RTF < 1` bar) and it keeps production untouched. The executed result is NO-GO, so Phase 2 stays closed and Phase 3 fallbacks are the actionable path.

## Out of Scope (recommended option)

- Shipping/bundling cloned voices; production adapter changes; downloaded/cloud voices; distribution without license/consent clearing.

## Assumptions to Validate

- Device measurement is reproducible with recorded commands/hashes (satisfied in `measurements.md`).
- Synthesis-side `first_chunk_ms` is a valid lower bound for audible TTFA (documented caveat, FR-PKT-006).
- Any future GO requires both improved on-device metrics and license/consent clearing.

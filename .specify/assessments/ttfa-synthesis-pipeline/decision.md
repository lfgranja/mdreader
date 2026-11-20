# Decision: TTFA da síntese/enfileiramento

- **Slug**: ttfa-synthesis-pipeline
- **Decided**: 2026-11-07
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md | converge-clarify.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Sintoma confirmado por trace e device: síntese sherpa-onnx/Piper síncrona no caminho de primeiro áudio e produção contínua acoplada a `playback_watch_tick` async. Diagnostic record (`2201117TY`, Android 13) mostra `play_to_audio_ms` comum 1.6–17.3 s e extremo 45.9–120.4 s, com casos de offset alto consistentes com síntese/trim até amostra útil. |
| Evidence strength | strong | Código (`lib.rs`, `synth.rs`, `backend.rs`), API sherpa-onnx 1.13.8 (Rust/C++: callback pós-batch/síncrono, **não** streaming intra-segmento VITS), device diagnostics, e constraints Tauri/Tokio/cpal/ORT documentadas. Fase-a-fase no device ainda pode ser instrumentada na implementação, mas evidência fecha a direção. |
| Value vs. inaction | strong | TTFA alto/degradado afeta UX central; extremos em resume/offset alto tornam a espera inaceitável. |
| Feasibility / appetite | adequate | Option A (segment-level scheduling: worker/produtor dedicado, primeiro segmento útil rápido, backpressure 60 s, watch sem síntese) é viável com risco controlável por TDD + device smoke. Option B excluída por falta de suporte comprovado na API em uso. |
| Strategic fit | strong | Preserva Rust authority, mono f32 22.050 Hz, pre-roll 60 s, fingerprint/resume/SourceRange, FR-001/005. |
| Risk posture | adequate | Riscos: races/cancelamento, `Playing` silencioso, regressão resume/fingerprint/pre-roll, underrun, chapter pre-roll (T217/T232/T233) hoje parcial. Mitigação: escopo mínimo, cancel tokens, medições, testes de contrato/invariantes e validação device. |

## Verdict & Rationale

**go** para especificação e implementação controlada da **Option A**. Evidência fecha o mecanismo e elimina Option B no escopo atual: sherpa-onnx 1.13.8 VITS/Piper não entrega streaming intra-segmento via progress callback (callback pós-batch, síncrono, samples efêmeros; `generate_with_config` retorna buffer completo). O ganho mínimo seguro é **scheduling**: mover CPU-bound para worker/produtor dedicado, produzir/enfileirar o **primeiro segmento útil** (inclusive em resumes com `samples_before_entry`) o quanto antes, manter backpressure 60 s, e remover `synthesize` do `playback_watch_tick`. Estado publicado deve preservar FR-001/005.

## Handoff to `/speckit.specify`

- **Problem**: TTFA alto/extremo por síntese CPU-bound no caminho síncrono de `play_document` e acoplamento de produção ao `playback_watch_tick` async; resumes com offset alto podem sintetizar/descartar muitos segmentos antes do áudio útil.
- **Chosen approach**: Option A — segment-level synthesis scheduler. (1) Fast deterministic path: validação/fingerprint/store/oferta/switch/prepare/seed/open backend. (2) Worker dedicado para síntese+primeiro handoff+produção contínua com cancelamento e `queue_room`/60 s bound. (3) `playback_watch_tick` só progresso/crossing/drain/end/pre-roll arm, sem síntese.
- **In scope**: `src-tauri` scheduling/estado/cancelamento; integração com `adapter_synthesis::synth::synthesize`, `CpalBackend`/`ResidentQueue`, `Production`/timeline/medições; `note_play_request`/`note_first_consumed_sample` preservados; resume/re-anchor/chapter transition sem regressão.
- **Out of scope**: streaming intra-modelo VITS/split encoder-decoder; mudança de voz/modelo; alteração de `MAX_CHUNK_BYTES`, `MAX_PREROLL_SECS`, formato 22.050 Hz mono f32, fingerprint/blake3, contratos `SourceRange`/CLI; completar gapless chapter pre-roll além de não-regressão.
- **Success metrics**: TTFA (`note_play_request`→primeiro sample consumido) reduzido no device 2201117TY; sem underrun regressivo; sem `Playing` silencioso; `playback_watch_tick` sem CPU-bound síntese; resume/fingerprint/pre-roll/SourceRange preservados; testes host + smoke device.
- **Carried-forward constraints/questions**: FR-001/005 state transition exata (preparando vs. Playing após 1º enqueue/start); cancelamento em novo doc/stop/skip/chapter/re-anchor; T217/T232/T233 não devem regredir (T232 hoje arma token sem síntese real observada; T233 parcial); instrumentação por fase recomendada para provar ganho.
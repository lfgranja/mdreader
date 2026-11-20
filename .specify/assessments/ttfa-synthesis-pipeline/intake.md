# Idea Intake: TTFA da síntese/enfileiramento (pipeline `play_document`)

- **Slug**: ttfa-synthesis-pipeline
- **Created**: 2026-11-07
- **Source**: pasted text (solicitação do usuário, 2026-11-07) + trace de código em `src-tauri/src/lib.rs`
- **Type**: improvement

## Idea (as captured)

> "Rastreie o caminho exato de síntese/enfileiramento em src-tauri/commands/play_document e me devolva um mapa objetivo: “onde sintetiza, em qual thread/runtime, quantos chunks antes do start, e qual mudança mínima daria time-to-first-audio melhor sem quebrar pre-roll/fingerprint/resume”. O ganho real depende de o pipeline produzir os primeiros chunks logo, enfileirar o suficiente para o backend não underrun, e continuar sintetizando em segundo plano sem bloquear. Se hoje algum caminho sintetiza demais antes de tocar, ou sintetiza no lugar errado, aparece exatamente o sintoma de lentidão/início demorado."

## Restated

Reduzir o tempo até o primeiro áudio (TTFA) no caminho de abertura/reprodução do mdreader (`play_document`), movendo o trabalho CPU-bound de síntese sherpa-onnx/Piper para fora do fluxo bloqueante e desacoplando o produtor contínuo do tick de playback, sem quebrar pre-roll 60 s, fingerprint/blake3, resume/re-anchor, ordem/medição de unidades e invariantes de não reportar `Playing` silencioso.

## Origin & Context

- **Raised by**: usuário (investigação de lentidão/início demorado na reprodução)
- **Trigger**: sintoma percebido de início demorado; suspeita de síntese excessiva antes do start ou síntese no lugar errado.
- **Related artifacts**: `src-tauri/src/lib.rs` (`open_document_bytes_named`, `prepare_session_document`, `play_document`, `stream_synthesis`, `produce_next_unit`, `advance_production`, `playback_watch_tick`, `enqueue_chunk`, `open_backend`, `start_backend`); `crates/adapter-synthesis/src/synth.rs` (Piper VITS pt-BR faber, `OnceLock`, ONNX `num_threads:1`); `crates/adapter-synthesis/src/chunk.rs` (`MAX_CHUNK_BYTES=5000`); `crates/player/src/backend.rs` (`ResidentQueue`, `MAX_PREROLL_SECS=60s`, cpal). Scouts concluídos: `DryParakeet` (trace) e `RapidReindeer` (pesquisa externa, `agent://RapidReindeer`).

## First-Glance Unknowns

- [NEEDS CLARIFICATION: Qual o TTFA real (`note_play_request` → primeiro sample consumido) no device de referência (Android 13 arm64, 2201117TY) para documento típico, e qual parcela é síntese do 1º segmento vs. chunking/prepare/backend/open?]
- [NEEDS CLARIFICATION: `playback_watch_tick` executa em contexto Tokio async (500 ms); chamar `advance_production`/`synthesize` síncrono ali bloqueia tarefa/runtime em Android/Tauri? Medir com tracing/instrumentação.]
- [NEEDS CLARIFICATION: Qual transição de estado/UX é segura para não violar FR-001/005 quando a síntese do primeiro segmento for movida para worker (manter `Playing` só após primeiro enqueue+start, ou estado de preparação explícito já suportado)?]
- [NEEDS CLARIFICATION: Sherpa-onnx Rust/C API em uso expõe callback/chunk streaming VITS/Piper equivalente a split encoder/decoder? (RapidReindeer: não confirmado.)]
- [NEEDS CLARIFICATION: `num_threads>1` no ONNX melhora TTFA sem contention/underrun/bateria no device alvo?]
- [NEEDS CLARIFICATION: Pre-roll de capítulo N+1 (T217/T232) sintetiza/enfileira áudio real hoje ou apenas arma token? Impacto em gapless.]
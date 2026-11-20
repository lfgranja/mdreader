# Idea Research: TTFA da síntese/enfileiramento (`play_document`)

- **Slug**: ttfa-synthesis-pipeline
- **Created**: 2026-11-07
- **Evidence confidence (overall)**: medium (trace estático forte no código; comportamento em device/latências reais não medidos; disponibilidade de streaming VITS na API Rust sherpa-onnx em uso não confirmada)

## Users & Demand

- Usuários do Ressoa/mdreader percebem “lentidão/início demorado” ao iniciar reprodução (relato do solicitante, 2026-11-07). — [source: intake; ASSUMPTION quanto à frequência/população] (confidence: low)
- Produto é driving-first/mobile: TTFA alto degrada a percepção de resposta ao toque/Play e pode conflitar com expectativas de mídia/lock-screen. — [source: contexto do produto em `AGENTS.md`, constituição/princípios do repo; ASSUMPTION qualitativa] (confidence: medium)
- Não há métrica de TTFA agregada no repo observada nesta investigação. — [source: ausência observada durante trace; ASSUMPTION] (confidence: medium)

## Prior Art (interno, evidência de código)

- **Entrada única de síntese:** `commands::play_document` (`src-tauri/src/lib.rs:5767-5899`) é o caminho que prepara sessão, abre backend, faz landing `Playing`, chama `stream_synthesis`, e publica. Wrappers: `commands::play()` `5591-5601`; `open_document_bytes_named` chama `play_document(...,0.0)` em `ColdStart::StartNow` (`3366+,3482`); `AwaitOffer/AwaitExplicitResume` apenas `prepare_session_document` + `park_cold_start` (sem PCM). `start_parked_reading` → `play_document` (`2251`). JNI: `shell_jni.rs:1063,1420,1431,1440,1445`. — [source: código lido/trace DryParakeet] (confidence: high)
- **Preparação sem PCM:** `prepare_session_document` (`5603-5765`) guarda `LoadedDocument`, `seed_session_rate`, `adapter_synthesis::chunk::split_document(document,"")`, calcula H1/seções, `total_units=segments.len()`, cria/instala `SurfaceResolver` e headings (`position_secs=INFINITY`) — **antes da síntese**. — (confidence: high)
- **Backend antes de Playing:** `open_backend` (`5516-5529`, cria/reusa `CpalBackend`) roda antes do landing; recusa impede estado Playing silencioso. — (confidence: high)
- **Landing Playing antes do 1º chunk:** sessão executa `ShellVerb::Play`, seta `stream_anchor_secs`, reseta `playback_watch`, credita resume com `on_audible_progress(start_offset_secs)` se >0, `after_execute/publish_current_state` (`5814-5863`). Erro de síntese chama `abandon_playing_landing` (`5865-5872`). — (confidence: high)
- **Síntese síncrona por segmento:** `produce_next_unit` (`5286-5373`) faz `adapter_synthesis::synth::synthesize(&segment)` para 1 `DocumentSegment`, mede timeline/section, aplica `samples_before_entry` (`sample_index_of`, `2318-2326`), coloca PCM pós-corte em `production.pending`. — (confidence: high)
- **Primeiro áudio:** `stream_synthesis` (`5041-5100`) instala `Production` (`358-385`), faz `while pending_samples==0 && produce_next_unit()? {}`, pega `take_piece(..., queue_room())`, `enqueue_chunk`, se não started `start_backend()` (`device.play`) + `mark_production_started`, `note_first_consumed_sample`, depois `advance_production` 1x. **Start ocorre após primeiro piece com samples > 0 enfileirado**; tipicamente 1 segmento, podendo ser >1 se resume consome segmentos inteiros. — (confidence: high)
- **Produção contínua demand-driven:** `advance_production` (`5120-5153`) só produz próximo segmento se `pending` vazio e `queue_room()>0`; sempre tenta `handover_pending`. `queue_room` usa `cap_samples_of(buffer.cap_secs, sample_rate)` com cap 60 s e `producer::admits`. `finish_production` quando todos produzidos e pending vazio. — (confidence: high)
- **Runtime de produção contínua:** `playback_watch_tick` (Tokio async task, `spawn_playback_watch_task`, interval 500 ms, `2337-2354,2455-2515`) credita progresso audível, pode sinalizar fim de documento (respeitando `production_running`), republica crossing, **chama `commands::advance_production(state)`** (`2512`). Busca não encontrou `spawn_blocking`/thread dedicada para síntese em `src-tauri`/`player`. — (confidence: high para ausência observada; comportamento de bloqueio sob runtime específico requer medição)
- **Pre-roll/bound:** `ResidentQueue` 60 s em `crates/player/src/backend.rs` (`91-124+`), mono f32 22.050 Hz; `push` preserva head contíguo e descarta excedente além da janela. Callback cpal consome FIFO. — (confidence: high)
- **Síntese engine:** `crates/adapter-synthesis/src/synth.rs` usa sherpa-onnx Piper VITS pt-BR faber, `OnceLock<OfflineTts>`, `num_threads:1`. — (confidence: high)
- **Chunking:** `crates/adapter-synthesis/src/chunk.rs`, `MAX_CHUNK_BYTES=5000`, heading/paragraph, `SourceRange`/`resume_point`/`segment_index`. — (confidence: high)
- **Gapless/pre-roll capítulo:** `maybe_start_chapter_preroll` arma `NextChapterPreRoll` (`begin/should_trigger`) mas síntese/enqueue real do próximo capítulo não observada no recorte; `advance_chapter_queue` reabre via `open_document_bytes_named`. — (confidence: medium; T232/T217 contexto indica incompletude)

## Market & Context (práticas externas, citadas)

- **TTFA em pipelines TTS:** reduzir TTFA exige produzir/tocar primeira unidade audível cedo e manter pipeline à frente do playback para evitar underrun; “tem som” ≠ buffer completo. — [source: [voice-agent-handbook latency budget](https://github.com/wananing/voice-agent-handbook/blob/master/01-foundations/latency-budget.md)] (confidence: medium; domínio voice-agent, princípios aplicáveis)
- **Streaming VITS/Piper:** modelos split encoder/decoder podem decodificar em chunks com overlap para TTFA menor; requer suporte/modelo split e cuidado com equivalência numérica. Piper1/OHF documenta `piper_create` + `piper_synthesize_next` com primeiro chunk pequeno e chunks posteriores dimensionados por velocidade medida. — [source: [OHF-Voice piper1-gpl `docs/STREAMING.md`](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/STREAMING.md), [Piper `infer_onnx_streaming.py` docs](https://tderflinger.github.io/piper-docs/codereading/python/piper_train/infer_onnx_streaming/), [`streaming-vits` PyPI](https://pypi.org/project/streaming-vits/)] (confidence: medium; `streaming-vits` é terceiro; não prova API Rust sherpa-onnx em uso)
- **Sherpa-onnx API Rust:** exemplos Rust VITS listados (`vits_tts_lexicon`, `vits_tts_phonemize`); callback/chunk streaming VITS equivalente não confirmado nas fontes consultadas. — [source: [sherpa-onnx rust-api-examples](https://github.com/k2-fsa/sherpa-onnx/blob/master/rust-api-examples/README.md), [docs.rs `sherpa_onnx::OfflineTts`](https://docs.rs/sherpa-onnx/latest/struct.OfflineTts.html), RapidReindeer] (confidence: medium; **unverified for mdreader’s exact bindings**)
- **Tauri/CPU-bound:** comandos bloqueantes/trabalho pesado devem evitar bloquear UI/runtime; `tauri::async_runtime::spawn_blocking` / padrões async são recomendados para CPU-bound em handlers. — [source: [Tauri async_runtime](https://docs.rs/tauri/latest/tauri/async_runtime/index.html), [spawn_blocking](https://docs.rs/tauri/latest/tauri/async_runtime/fn.spawn_blocking.html), [Tauri async commands discussion](https://github.com/orgs/tauri-apps/discussions/10329)] (confidence: high para padrão; integração exata com `play_document`/AppState locks requer design)
- **Tokio blocking:** trabalho bloqueante/CPU-bound em contexto async pode prejudicar executor; usar `spawn_blocking`/threads dedicadas. — [source: [Tokio `spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)] (confidence: high)
- **cpal/AAudio:** `BufferSize::Fixed` é pedido (pode ser arredondado/variar); callback não deve bloquear em síntese/modelo; fila produtor/consumidor com backpressure e underrun controlado. — [source: [docs.rs `cpal::BufferSize` 0.18.2](https://docs.rs/cpal/latest/cpal/enum.BufferSize.html), [RustAudio/cpal #902](https://github.com/RustAudio/cpal/issues/902)] (confidence: medium para comportamento Android específico; princípios sólidos)
- **ORT threading/contention:** `intra_op_num_threads`, spinning e contention afetam latência/CPU em mobile; mais threads nem sempre melhora; desabilitar/limitar spinning pode reduzir contenção. — [source: [ONNX Runtime thread management](https://onnxruntime.ai/docs/performance/tune-performance/threading.html), [XNNPACK EP](https://onnxruntime.ai/docs/execution-providers/Xnnpack-ExecutionProvider.html), [sherpa-onnx #1337](https://github.com/k2-fsa/sherpa-onnx/issues/1337)] (confidence: medium; device/model-specific)

## Data & Constraints

- `DECLARED_RATE_HZ`/handoff mono f32 22.050 Hz; `MAX_PREROLL_SECS=60`; `MAX_CHUNK_BYTES=5000`. — [source: código/contracts citados] (confidence: high)
- Invariantes: fingerprint blake3, resume/`sample_index_of`/`samples_before_entry`/`SourceRange`/`segment_index`/CLI metadata onde aplicável, FR-001/002/003/005/007/009/013/017 (segundo comentários/código). — (confidence: high no nível de contratos observados)
- `playback_watch` tick 500 ms; síntese de 1 segmento pode exceder budget de tick em mid-range — **não medido**. — (confidence: low/unknown)
- Device referência conhecido do contexto: Android 13 arm64, 2201117TY. — [source: handoff/contexto do projeto] (confidence: medium)

## Evidence Against the Idea

- **Complexidade de concorrência:** mover síntese para worker exige coordenação com `AppState` locks, cancelamento (novo documento/skip/stop/chapter), e não quebrar ordem/medição de unidades/timeline. — [source: estrutura atual de locks e `Production`/watch] (confidence: high)
- **Risco de Playing silencioso/regressão FR-001/005:** landing atual ocorre antes do 1º chunk com abandono em erro; mudar timing pode criar janela sem áudio ou estados incoerentes se mal desenhado. — (confidence: high)
- **Streaming intra-modelo pode ser inviável/inseguro agora:** API Rust/sherpa-onnx em uso não confirmada para chunk callback VITS; split encoder/decoder exigiria assets/pipeline diferentes e validação de equivalência. — (confidence: medium)
- **Tuning de threads ORT é empírico:** `num_threads>1` pode melhorar ou piorar por contenção térmica/big.LITTLE/callback áudio. — (confidence: medium)
- **Benefício real depende de medição:** sem baseline TTFA por fase no device, “mudança mínima” pode otimizar parte errada. — (confidence: high)

## Gaps & Open Questions

- [NEEDS CLARIFICATION: baseline TTFA/device (`note_play_request`→primeiro sample consumido) e tempo por `synthesize(segment)`/prepare/open.]
- [NEEDS CLARIFICATION: runtime de `playback_watch_tick`/Tauri async em Android — bloqueio real por `advance_production` síncrono?]
- [NEEDS CLARIFICATION: desenho seguro de estado/UX para “primeiro áudio em worker” preservando FR-001/005.]
- [NEEDS CLARIFICATION: API exata `sherpa_onnx`/bindings no `Cargo.lock`/código: suporta geração incremental/callback para VITS Piper?]
- [NEEDS CLARIFICATION: chapter N+1 pre-roll real (T217/T232/T233) — estado funcional e risco de regressão.]

## Sources

- Internal: `src-tauri/src/lib.rs`, `src-tauri/src/shell_jni.rs`, `crates/adapter-synthesis/src/{synth,chunk}.rs`, `crates/player/src/backend.rs` (trace citado por linha acima).
- Internal artifacts: `.specify/assessments/ttfa-synthesis-pipeline/intake.md`, scout outputs `agent://DryParakeet`, `agent://RapidReindeer` / session md file.
- External: OHF-Voice Piper streaming docs; Piper `infer_onnx_streaming.py` docs; `streaming-vits` PyPI; voice-agent-handbook latency budget; sherpa-onnx rust-api-examples/docs.rs/Dart/C# callback refs; Tauri async runtime docs/discussion; Tokio `spawn_blocking`; cpal `BufferSize`/issue #902; ONNX Runtime threading/XNNPACK docs; sherpa-onnx #1337. (URLs in text; hosts: github.com, docs.rs, pypi.org, onnxruntime.ai, tauri.app/docs.rs, pub.dev — high-reliability/primary where applicable; third-party noted.)
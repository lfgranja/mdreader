# Concept: Scheduling do pipeline de síntese para TTFA

- **Slug**: ttfa-synthesis-pipeline
- **Created**: 2026-11-07
- **Recommended option**: Option A — Scheduling/worker mínimo (sem streaming intra-modelo)

## Options

### Option A — Scheduling/worker mínimo (recomendado)
- **Sketch**: manter segmentação, formato, fingerprint/resume e bound 60 s; extrair síntese CPU-bound para worker dedicado (`spawn_blocking`/thread produtor com coordenação segura sobre `AppState`/`Production`/backend). O caminho de abertura faz trabalho rápido e determinístico (validação, fingerprint/store/oferta, `switch_to_document`, `prepare_session_document`, `seed_session_rate`, `open_backend`) e delega a produção: o worker sintetiza até obter o **primeiro segmento útil**, enfileira piece, inicia backend, marca started e continua produzindo sob backpressure (`queue_room`, cancelamento por novo documento/stop/skip). `playback_watch_tick` deixa de chamar `synthesize` — fica com progresso audível, crossing, drain/end (`production_running`), armar pre-roll.
- **Appetite**: `medium` (dias a ~2 semanas, incluindo instrumentação e validação em device; risco concentrado em concorrência/cancelamento/estado)
- **Trade-offs**: ganha TTFA e reduz bloqueio de runtime/comando com mudança alinhada ao desenho atual (segment-level). Custo: complexidade de sincronização/cancelamento, necessidade de testes para races, resume, troca de documento e erro de device. Não depende de API de streaming VITS não confirmada.
- **Rabbit holes**: (1) manter `Playing`/publish coerente sem áudio falso (FR-001/005) durante worker; (2) cancelamento limpo quando novo doc/stop chega antes do 1º chunk; (3) `Production`/locks/poisoning/deadlock; (4) síntese chamada de múltiplos pontos (chapter/re-anchor) divergindo do novo scheduler; (5) benchmark mostrando gargalo fora da síntese (I/O, modelo, backend open).

### Option B — Streaming intra-segmento VITS/Piper (split encoder/decoder ou callback)
- **Sketch**: após confirmar suporte na API sherpa-onnx/C/Rust e assets compatíveis, sintetizar um segmento em chunks menores (primeiro chunk pequeno) para emitir áudio antes do fim do segmento, preservando equivalência numérica/overlap/contexto.
- **Appetite**: `large` (incerteza alta: binding, assets split, validação acústica/equivalência, fallback)
- **Trade-offs**: potencial TTFA ainda menor dentro de segmentos longos. Riscos altos: API Rust não confirmada; mudanças de modelo/packaging; difícil garantir equivalência; maior superfície de regressão.
- **Rabbit holes**: suporte inexistente/parcial em `sherpa_onnx` crate usado; diferença Piper1 vs sherpa-onnx packaging; sample-accurate resume/medição por chunk; regressões em pt-BR faber.

### Option C — Ajustes táticos sem mover produtor
- **Sketch**: micro-otimizações (pré-carregar engine, reduzir trabalho de prepare, ajustar `num_threads`/spinning ORT, mudar ordem menor) mantendo síntese no fluxo atual/watch.
- **Appetite**: `small`
- **Trade-offs**: baixo risco inicial, mas provável ganho limitado se o gargalo principal for scheduling bloqueante do 1º segmento/documento. `num_threads` é empírico e pode piorar contenção.
- **Rabbit holes**: tratar sintoma sem corrigir causa; medições inconclusivas por variância de device/thermal.

### Option D — Não mudar agora (só medir)
- **Sketch**: instrumentar TTFA por fase no device alvo e adiar refactor.
- **Appetite**: `small`
- **Trade-offs**: reduz risco de código, mas não resolve sintoma relatado. Útil como pré-requisito de implementação, insuficiente como decisão final se baseline confirmar bloqueio.
- **Rabbit holes**: instrumentação acoplada demais; perder janela de correção.

## Recommendation

**Option A**, precedida de instrumentação mínima: o trace mostra síntese síncrona CPU-bound no caminho de `play_document`/primeiro áudio e `advance_production` no `playback_watch_tick` sem offload observado; as fontes externas suportam offload de CPU-bound + produtor/consumidor com backpressure para áudio. Streaming intra-modelo (B) fica condicionado à confirmação de API/assets. Qualquer implementação deve ser medida no device alvo e preservar invariantes.

## Out of Scope (for recommended option)

- Streaming VITS/Piper intra-segmento; mudança de voz/modelo/packaging.
- Mudança de chunking semântico/prosódia/symbol-fatigue.
- Reescrever backend cpal/AAudio ou contratos de `SourceRange`/bookmark.
- Completar gapless chapter pre-roll (exceto não-regressão).

## Assumptions to Validate

- `AppState`/`Production` podem ser coordenados de forma segura entre worker de síntese, playback watch e comandos (cancelamento, documento novo, resume, erro). **Validação**: testes host + revisão de locks; smoke em device.
- Primeiro segmento útil pode ser produzido+enfileirado significativamente antes do caminho atual sem underrun inicial. **Validação**: benchmark TTFA/fase e underrun.
- Estado publicado durante preparação mantém FR-001/005 e UX aceitável. **Validação**: testes de erro/superfície + device.
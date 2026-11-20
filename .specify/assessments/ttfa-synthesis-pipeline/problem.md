# Problem Definition: TTFA da síntese/enfileiramento

- **Slug**: ttfa-synthesis-pipeline
- **Created**: 2026-11-07
- **Inputs used**: intake.md | research.md | scout trace (DryParakeet) | external research (RapidReindeer)

## Problem Statement

No mdreader, o caminho que inicia a reprodução (`open_document_bytes_named` → `play_document` → `stream_synthesis`) executa síntese sherpa-onnx/Piper VITS **síncrona e CPU-bound** no fluxo chamador para produzir o primeiro áudio, e a produção contínua (`advance_production` → `produce_next_unit` → `synthesize`) também é acionada pelo `playback_watch_tick` (Tokio async, 500 ms). Em dispositivos Android mid-range, isso pode atrasar o primeiro áudio (TTFA), bloquear handlers/IPC/JNI e competir com o callback de áudio/UI, mesmo havendo janela de pre-roll de 60 s e produção demand-driven — exatamente o sintoma de “lentidão/início demorado” quando o primeiro chunk útil demora ou quando a síntese ocorre no lugar/runtime errado.

## Affected Users & Stakeholders

- **Users**: leitores ouvindo Markdown/TTS no Android, especialmente ao tocar Play/abrir documento; percebem atraso até o som começar ou resposta lenta da UI. — [evidence: intake] (confidence: medium)
- **Stakeholders (produto/manutenção)**: mantenedores precisam preservar driving-first, resume/fingerprint, pre-roll 60 s, medições FR-002/003/005/007/009/013/017 e estabilidade do Tauri/JNI. — (confidence: high)
- **Stakeholders (plataforma)**: Android/AAudio/cpal, ONNX Runtime/sherpa-onnx, Tauri 2; comportamento depende de device big.LITTLE, runtime async e backend de áudio. — (confidence: high)

## Goals

- Reduzir TTFA percebido/medido (press/play request → primeiro sample consumido) produzindo e enfileirando o **primeiro segmento útil** o quanto antes.
- Manter áudio contínuo após o start: enfileirar o suficiente para evitar underrun e continuar síntese em background com backpressure sobre a janela de 60 s.
- Remover CPU-bound de síntese do caminho bloqueante inadequado (comando/IPC/JNI/callback sensível) e do tick que não deve executar trabalho pesado.
- Preservar: pre-roll `MAX_PREROLL_SECS=60`, fingerprint/blake3, oferta/`switch_to_document`/`park_cold_start`, resume/re-anchor (`start_offset_secs`, `sample_index_of`/`samples_before_entry`, `SourceRange`/`resume_point`/`segment_index`), medição de unidades/timeline, e invariantes FR-001/005 (nunca reportar Playing silencioso sem abandono correto).

## Non-Goals

- Não redesenhar formato de áudio (mono f32 22.050 Hz), `ResidentQueue` bound, contratos CLI/IPC/JNI, ou semântica de bookmark/fingerprint.
- Não adotar streaming intra-modelo Piper/sherpa-onnx (split encoder/decoder/callback VITS) sem confirmação de API/bindings e validação de equivalência.
- Não alterar `MAX_CHUNK_BYTES`, prosódia, symbol-fatigue, tradução, ou UI, exceto estado de carregamento/preparação estritamente necessário e já compatível.
- Não otimizar cegamente `ONNX num_threads` sem benchmark no device alvo.
- Não resolver chapter N+1 pre-roll gapless completo (T217/T232/T233) neste escopo, exceto evitar regressão.

## Success Metrics

- **TTFA (primário):** mediano/p95 de `note_play_request` → primeiro sample consumido reduzido em documento representativo no device de referência (Android 13 arm64 2201117TY), com baseline registrado antes/depois. (baseline: **unknown**)
- **Sem underrun regressivo:** sem aumento material de underruns/gaps após start em leitura contínua de 5+ min; fila mantém avanço sob backpressure 60 s. (baseline: **unknown**)
- **Sem bloqueio observável:** comando/JNI de abertura retorna ou transiciona para estado seguro sem segurar a thread de UI/comando durante síntese completa do documento; `playback_watch_tick` não executa `synthesize` CPU-bound. (baseline: síntese síncrona observada no trace)
- **Invariantes preservadas:** testes relevantes passam; resume/fingerprint/re-anchor/pre-roll e cenários de erro mantêm comportamento (FR-001/002/003/005/007/009/013/017 conforme aplicável). (baseline: suíte atual; alvo: sem regressão)
- **Recurso:** sem regressão material de RSS/PSS/VmHWM atribuída ao novo pipeline em cenário equivalente. (baseline: **unknown**)

## Cost of Inaction

- TTFA permanece alto/variável, reforçando percepção de lentidão e risco de abandono ao iniciar leitura.
- CPU-bound em caminhos errados pode competir com áudio/UI/JNI, aumentando risco de jank, stalls ou underruns em mid-range.
- Evoluções futuras (gapless chapter, symbol-fatigue, prosody) herdam um pipeline difícil de compor/cancelar.

## Open Questions

- [NEEDS CLARIFICATION: baseline TTFA por fase e tempo por segmento no device de referência.]
- [NEEDS CLARIFICATION: bloqueio real do runtime async/Tauri pelo `advance_production` no `playback_watch_tick` em Android.]
- [NEEDS CLARIFICATION: estado de UI/sessão seguro durante worker de primeiro áudio (FR-001/005).]
- [NEEDS CLARIFICATION: suporte a streaming/callback VITS na API sherpa-onnx Rust efetivamente usada.]
- [NEEDS CLARIFICATION: estado de chapter pre-roll real e critérios de não-regressão gapless.]
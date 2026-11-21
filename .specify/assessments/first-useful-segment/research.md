# Idea Research: Primeiro segmento útil para reduzir TTFA

- **Slug**: first-useful-segment
- **Created**: 2026-11-21
- **Evidence confidence (overall)**: medium (mecanismo de host bem ancorado; prova de dispositivo `[NV]`)

## Users & Demand

- Usuário relatou ~183 s entre abrir "Hamlet trecho.md" e ouvir áudio; medição
  histórica `play_to_audio_ms=183089` em 2201117TY — [source: `.specify/memory/STATE.md`, entrada "Bug ACTION_VIEW Hamlet"] (confidence: high, cited).
- A solicitação não é hipotética: o mesmo relato motivou o ciclo 025, que
  corrigiu apenas o título e deixou a latência explicitamente fora de escopo —
  [source: `.specify/memory/STATE.md`, fechamento do ciclo 025] (confidence: high, cited).
- Nenhuma entrevista ou telemetria de múltiplos usuários foi coletada neste
  ciclo: a demanda é um relato único corroborado por evidência ADB —
  [source: `.specify/memory/STATE.md`] (confidence: medium, cited).

## Prior Art

- 013 TTFA já moveu a síntese do tick de watch para um produtor limitado e
  priorizou "first useful audio", mas declarou **chunking** fora de escopo —
  [source: `specs/013-ttfa-synthesis-pipeline/spec.md` §Out of Scope e SC-TTFA-001] (confidence: high, cited). Este ciclo é o complemento natural, não uma reescrita.
- O mecanismo de handover já existe: `handover_pending_for_generation`
  (`src-tauri/src/lib.rs:5894-5922`) pega uma peça, enfileira e inicia o backend
  na primeira peça; `stream_synthesis` (:6085-6154) produz até
  `production_started_for_generation` antes de devolver. Falta o primeiro
  segmento ser pequeno — [source: leitura estrutural em host] (confidence: high, cited).
- `adapter_synthesis::chunk::split_document` usa `MAX_CHUNK_BYTES = 5_000`
  (`crates/adapter-synthesis/src/chunk.rs:29,163-165`) e
  `split_document_within` já aceita um orçamento explícito (:173-177); um
  documento de ~3,9 KB cabe integralmente em um segmento — [source: leitura em host] (confidence: high, cited).
- T232 (novo worker de pré-síntese, 200-350 LOC, dependente de dispositivo)
  foi avaliado e mantido DEFER em ciclos anteriores — [source: `.specify/memory/STATE.md`] (confidence: high, cited).

## Market & Context

- Alternativa atual do usuário é esperar a síntese integral do documento; não
  há atalho de UI nem configuração que reduza o tamanho do primeiro segmento
  — [source: leitura estrutural de `lib.rs`/`chunk.rs`] (confidence: high, cited).
- Custo de não agir: cada início fresco de documento curto repete o mesmo
  pré-roll longo; o diagnóstico de 013 (TTFA relativo) permanece apenas parcial
  — [source: `.specify/validations/ttfa-synthesis-pipeline/validate.md`] (confidence: medium, cited).

## Data & Constraints

- Contrato de áudio inalterado: PCM `f32` mono @ 22050 Hz consumido pelo
  backend; a mudança é apenas de fronteira de segmento — [source: documentos de
  arquitetura e `crates/adapter-synthesis`] (confidence: high, cited).
- `DocumentSegment` (`chunk.rs:123-142`) carrega `content`, `source_range`,
  `boundary_kind`, `segment_index` (estritamente +1, regra 9) e `resume_point`
  (regra 11); dividir o primeiro segmento exige herdar esses metadados —
  [source: leitura em host] (confidence: high, cited).
- Gate autoritativo `scripts/verify.sh`, com os 4 checks SDD PASS observados
  no fechamento do ciclo 025 (26 specs/0 gaps, 867 tasks/0 cycles, 0 duplicatas,
  25 converge baselined); Factory HEAD `b1deab1` é somente leitura —
  [source: `.specify/memory/STATE.md` e briefing do orquestrador] (confidence: high, cited).
- Gate 2 de diff bidirecional impede novo target de teste; testes devem entrar
  no módulo inline `ttfa_013_tests` (`src-tauri/src/lib.rs:10657`) e
  `scripts/expected-tests.txt` não pode mudar — [source: briefing do orquestrador] (confidence: high, cited).
- Nenhuma captura pareada de `play_to_audio_ms` antes/depois está disponível
  neste checkout; qualquer afirmação de melhoria em dispositivo é `[NV]` —
  [source: ausência observada; ASSUMPTION] (confidence: medium, assumption).

## Evidence Against the Idea

- Dividir no whitespace pode produzir um primário muito curto (ex.: "HAMLET")
  e aumentar o número total de unidades, afetando progresso e contagem de
  seções — [source: leitura de `prepare_session_document`, `lib.rs:6716-6742`] (confidence: medium, cited).
- Sem whitespace antes do limite, forçar divisão quebraria palavras; a
  alternativa é não dividir nesse caso, reduzindo a cobertura do mecanismo —
  [source: raciocínio sobre `forbid` implícito em `chunk.rs`] (confidence: medium, assumption).
- A melhoria real de TTFA depende do custo fixo de inicialização do modelo,
  que pode dominar o ganho de síntese do primário; sem captura pareada, o
  benefício em dispositivo não está provado — [source: ASSUMPTION] (confidence: low, assumption).
- Retomada e cold-start parked usam o mesmo preparador; uma mudança descuidada
  pode alterar offsets de retomada ou síntese indevida ao estacionar —
  [source: `lib.rs:3985` e `:6774`] (confidence: high, cited).

## Gaps & Open Questions

- [NEEDS CLARIFICATION: revisar a métrica de sucesso para ser estritamente relativa, sem meta numérica absoluta de TTFA; resolvido na spec como "menor que o baseline pareado".]
- [NEEDS CLARIFICATION: confirmar se o orquestrador executará gates/builds/testes após o merge das mudanças; documentos não os executam.]

## Sources

- `src-tauri/src/lib.rs` (host, leitura estrutural; line ranges citadas acima).
- `crates/adapter-synthesis/src/chunk.rs` (host, leitura estrutural).
- `specs/013-ttfa-synthesis-pipeline/spec.md` e `.specify/validations/ttfa-synthesis-pipeline/validate.md` (repo local).
- `.specify/memory/STATE.md` e bug `playback-not-starting-after-file-selection` (repo local).
- `scripts/verify.sh` e `scripts/expected-tests.txt` (repo local, não modificados).

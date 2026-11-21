# Problem Definition: Primeiro segmento útil para reduzir TTFA

- **Slug**: first-useful-segment
- **Created**: 2026-11-21
- **Inputs used**: intake.md | research.md | user input

## Problem Statement

Em um início fresco de reprodução, o ouvinte espera a síntese integral do
primeiro segmento — que, para documentos curtos, é o documento inteiro — antes
de ouvir qualquer áudio; em 2201117TY isso foi observado como ~183 s com
"Hamlet trecho.md". O problema importa agora porque o ciclo 025 fechou a
titulação, deixando esta como a última barreira conhecida para um início
utilizável, e porque o mecanismo de handover de 013 já está presente.

## Affected Users & Stakeholders

- **Users**: ouvintes que abrem um documento Markdown e pressionam play do
  início, especialmente em dispositivos Android com síntese local lenta.
- **Stakeholders**: mantenedores do motor de síntese (dono de 013/T232), o
  orquestrador do ciclo (gate `scripts/verify.sh`) e o usuário que reportou o
  bug — cada um com interesse em um diff pequeno, verificável e reversível.

## Goals

- Fazer o primeiro áudio chegar após a síntese de uma unidade pequena, em vez
  da unidade integral do documento.
- Manter a retomada com posição armazenada (`start_offset_secs > 0`) com a
  segmentação e o timeline atuais.
- Preservar metadados de segmento, ordenação de índices e o contrato de PCM
  mono f32 22050 Hz.
- Tornar o mecanismo verificável em host, sem novo target de teste.

## Non-Goals

- Meta numérica absoluta de TTFA (nem "menos de X segundos"): o critério é
  relativo ao baseline pareado.
- Prova de comportamento em dispositivo nesta cadeia documental (`[NV]`).
- Reescrita do sintetizador, mudança de voz/modelo, ou conclusão de T232.
- Criação de novo target de teste, edição de `scripts/expected-tests.txt` ou
  mudança em `synth.rs`, `chunk.rs`, `Cargo.toml`, HTML, capabilities e workflow.
- Edição de specs, bugs ou `converge.md` de outros ciclos.

## Success Metrics

- Tempo até o primeiro áudio menor que o do baseline na mesma condição
  pareada (baseline histórico: 183089 ms; comparação controlada em dispositivo
  `[NV]`) — métrica estritamente relativa.
- 100% dos inícios com `start_offset_secs == 0.0` e primeiro segmento maior que
  160 bytes produzem um primário ≤160 bytes terminando em whitespace
  (baseline host: 0%, segmento único de ~3,9 KB).
- 100% das retomadas com `start_offset_secs > 0` preservam a segmentação
  anterior, com zero primários adicionais.
- 100% dos segmentos têm `segment_index` sequencial e metadados herdados; zero
  mudanças no contrato PCM (baseline: contrato estável).
- 100% dos novos testes no módulo `ttfa_013_tests` do target já pinado, com
  zero arquivos de teste novos.

## Cost of Inaction

Cada abertura de documento curto repete o pré-roll longo; a melhoria relativa
de 013 permanece parcial em dispositivo e o relato do usuário continua sem
solução, apesar de existir um mecanismo de handover pronto para ser alimentado
por uma unidade menor.

## Open Questions

- Nenhuma bloqueante. As dúvidas de fronteira (whitespace, retomada, índices,
  alocação de testes, natureza relativa do TTFA) foram respondidas na seção
  `## Clarifications` da spec 026.

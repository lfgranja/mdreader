# Decision: Primeiro segmento útil para reduzir TTFA

- **Slug**: first-useful-segment
- **Decided**: 2026-11-21
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Relato real com `play_to_audio_ms=183089` em 2201117TY e causa localizada em host (`lib.rs:6706`, `chunk.rs:29`). |
| Evidence strength | adequate | Mecanismo de chunking/handover lido em host com alto detalhe; melhoria final em dispositivo é `[NV]`, por isso não é `strong`. |
| Value vs. inaction | strong | Único bloqueio conhecido para início útil após o ciclo 025; alternativa T232 é large e dependente de device. |
| Feasibility / appetite | strong | Option A é small: helper privado + flag no preparador + dois callers + testes no módulo existente. |
| Strategic fit | strong | Complementa 013 (que excluiu chunking) e preserva as fronteiras adapter/engine e o contrato PCM. |
| Risk posture | adequate | Riscos de whitespace, reindexação e retomada identificados com mitigações concretas; risco residual de dispositivo declarado `[NV]`. |

## Verdict & Rationale

**go**. O scorecard satisfaz os requisitos do gate: problema válido, evidência
`adequate` (não `weak`/`unknown`), conceito shaping com opção recomendada e
riscos compreendidos. O mecanismo de handover já existe; a mudança entrega uma
unidade menor exatamente no início fresco, sem tocar no sintetizador nem nos
contratos de segmento. A ausência de captura pareada em dispositivo impede
`strong` em evidência, mas não bloqueia a especificação: a SC de TTFA é
relativa e a medição device fica marcada `[NV]`.

## If needs-clarification

Não aplicável — nenhuma pergunta bloqueante permanece.

## If go — Handoff to `/speckit-specify`

- **Problem**: início fresco espera a síntese de um segmento de ~3,9 KB antes do
  primeiro áudio; retomada e estacionamento não têm esse problema.
- **Chosen approach**: Option A — primário ≤160 bytes em whitespace apenas
  quando `start_offset_secs == 0.0`, gerado no preparador de sessão.
- **In scope / out of scope**: inclui helper privado, flag `primer`, chamadores
  start/parked, reindexação e testes em `ttfa_013_tests`; exclui `synth.rs`,
  `chunk.rs`, `Cargo.toml`, `expected-tests.txt`, HTML, capabilities, workflow,
  specs de outros ciclos e qualquer meta absoluta de TTFA.
- **Success metrics**: TTFA relativo menor que o baseline pareado (`[NV]` em
  device); primário em 100% dos inícios frescos elegíveis; 100% das retomadas
  sem primário; sequência de índices e metadados preservados; PCM inalterado;
  testes no target pinado.
- **Carried-forward open questions**: confirmar se há whitespace antes de 160
  bytes nos documentos representativos (caso contrário, não dividir) e se o
  ganho real em dispositivo é material (captura pareada `[NV]`).

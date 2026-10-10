# Decision: FR-024 Adaptador Mínimo de Síntese Offline

- **Slug**: fr-024-synthesis-adapter
- **Decided**: 2026-10-06
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | `problem.md`: shell (`001`) é prova de mecanismo mas não produto; P1 (`US1-US3`) impossível sem áudio real; `SC-044`/`gate-8-full` bloqueados; princípio VIII (driving-first) exige lock-screen playback contínuo. |
| Evidence strength | adequate | `research.md`: arquitetura interna sólida (`engine-core` trait, `standin_a`/`standin_b`, `player` com mono f32 @ 22050 Hz, `redb`/`blake3`); motor (`sherpa-onnx` + `ort` + `NNAPI`) e vozes (`Piper` pt-BR CC0) confirmados via fetch + `constitution.md`; mas latência real (`SC-009`) e boundary events (`SentenceBoundary`) não verificados no hardware de referência — marcados `ASSUMPTION`. |
| Value vs. inaction | strong | `problem.md` “Cost of Inaction”: sem áudio, todo investimento em `player`/`session`/`diag` não retorna valor externo; `SC-051` passa, mas `SC-044` e `gate-8-full` permanecem `DEFERRED`. |
| Feasibility / appetite | strong | `concept.md` Option A (`small`–`medium`, 1–3 semanas) usa trait existente, motor já escolhido pela constituição; não requer redesign de `001`, cloud, ou segmentação perfeita. |
| Strategic fit | strong | Alinha com `constitution.md` §Engine Registry (`offline_default` = `sherpa-onnx` + `Piper`), princípio VIII, e contrato `engine-core` neutro; não introduz novo provider sem consentimento. |
| Risk posture | adequate | Riscos identificados (`research.md` Evidence Against): latência não medida (`SC-009`), segmentação básica pode não satisfazer `FR-036`/`FR-023a`, regressão no `player` ao trocar stand-in pelo real. Mitigação: benchmark no dispositivo de referência antes de considerar pronto; preservar `SC-051`; manter falha nomeada (`FR-035`) no contrato. Nenhum risco é oculto; todos estão nos 5 `First-Glance Unknowns` / `Gaps`. |

## Verdict & Rationale

**GO — Option A (Adaptador mínimo).**

O problema é real, o conceito é viável dentro de um appetite saneável, e a evidência interna é robusta. A única barreira que impede `go` imediato é a falta de benchmark de latência (`SC-009`) e a incerteza sobre fronteiras de sentença do `Piper` — ambos são validações, não bloqueios arquiteturais. A recomendação de `concept.md` (Option A) respeita todos os não-objetivos (`problem.md`) e depende apenas de validar as suposições listadas (`concept.md` §Assumptions to Validate). Não há razão para `kill` (o valor do shell depende disso) nem `needs-clarification` como veredito final — as dúvidas são claras, têm estágio definido (`/speckit.clarify` após `/speckit.specify`), e não impedem a entrega de uma primeira versão funcional.

O veredito está condicionado a: o adaptador deve ser integrado com falha nomeada (`FR-035`) no primeiro ciclo, o benchmark de latência deve ser executado no dispositivo de referência antes da declaração de `SC-009` PASS, e `SC-051` (stand-in mecanism) deve continuar passando durante a transição.

## If go — Handoff to `/speckit.specify`

- **Problem**: O shell `mdreader` (`001-android-media-shell`) tem playback, focus, notificação, resume e diagnóstico completos, mas depende apenas de stand-ins (`standin_a`/`standin_b`). Sem síntese real (`FR-024`), os casos de uso P1 (`US1-US3`) nunca podem ser demonstrados; `SC-044` e `gate-8-full` permanecem `DEFERRED`.
- **Chosen approach**: Option A — Adaptador mínimo (`sherpa-onnx` + `Piper` pt-BR `faber`, segmentação básica `pulldown-cmark` + guarda pt-BR, mono f32 @ 22050 Hz, falha nomeada `FR-035` se formato errar). Sem cloud, sem segunda voz, sem export de diagnóstico, sem tela in-app.
- **In scope / out of scope**: In scope = adapter trait (`engine-core`), output format (mono f32 @ 22050 Hz), basic split, named failure on mismatch. Out of scope = redesign do `player`/`session`/`diag`; cloud synthesis; perfect segmentation (`SentenceBoundary`); byte-budget rigoroso inicial; diagnostic viewer; document picker; second voice.
- **Success metrics**: `SC-009` (≤3 s play-to-audio), `SC-044` (dois motores reais comprovados no hardware), `SC-051` preservado (failover não quebrado), P1 `US1-US3` demonstrável com áudio real e tela apagada.
- **Carried-forward open questions (para `/speckit.clarify` após `/speckit.specify`)**:
  - [NEEDS CLARIFICATION] Qual a latência real do `sherpa-onnx` + `ort` (`NNAPI`) no dispositivo de referência (`mid-range arm64`, 6 GB RAM) para o primeiro chunk a 22050 Hz mono (`SC-009`)?
  - [NEEDS CLARIFICATION] `Piper` expõe eventos de limite de sentença (`SentenceBoundary` / `WordBoundary`)? Se não, a segmentação básica (`pulldown-cmark`) é suficiente para `FR-036` / `FR-023a` sem quebra de resume?
  - [NEEDS CLARIFICATION] Como o byte-budget de 5.000 bytes (`constitution.md` linha 353) interage com chunks de Markdown maiores que o limite — split antes da síntese impacta alinhamento de posição?
  - [NEEDS CLARIFICATION] A ordem de integração preserva `SC-051` (stand-in mechanism) durante a transição para adaptador real? Qual a sequência exata de testes?
  - [NEEDS CLARIFICATION] Se o adaptador entregará falha nomeada (`FR-035`) desde o primeiro ciclo — o `player` já declara `FR-035`, mas o adapter precisa respeitar; como é verificado?

## Clarify After Specify

As 5 dúvidas acima (`problem.md` + `concept.md`) devem ser resolvidas via `/speckit.clarify` **depois** de `/speckit.specify`, quando a especificação do adaptador (interface do trait, formato de saída, testes de conformidade) estiver definida. Não devem bloquear a definição (`/speckit.specify`), mas devem ser validadas antes da declaração de `SC-009` PASS e `SC-044` PASS.

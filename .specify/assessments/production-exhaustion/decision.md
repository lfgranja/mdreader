# Decision: Exaustão da produção não finaliza o documento

- **Slug**: production-exhaustion
- **Decided**: 2026-11-21
- **Verdict**: go
- **Artifacts reviewed**: intake.md | research.md | problem.md | concept.md
- **Origin**: bug `playback-stall-at-end-of-document` (achado no smoke de longa
  duração do ciclo 026 em `2201117TY`), com causa refinada por diagnóstico de
  host (`ScoutStallCode`)

## Scorecard

| Criterion | Rating | Justification |
|-----------|--------|---------------|
| Problem validity | strong | Bug registrado com posição congelada em 199,203 s por >100 s, sem `session_end`/`action=complete`, MediaSession `state=3`; causa localizada em `produce_bounded:5817-5819`. |
| Evidence strength | adequate | Causa e helper de finalização lidos em host com alto detalhe; reprodução limpa e A/B em dispositivo ainda `[NV]`, por isso não `strong`. |
| Value vs. inaction | strong | Fecha o ciclo iniciado por 026: sem a correção, o ganho de TTFA termina em uma sessão que nunca conclui; alternativa C é correção de sintoma. |
| Feasibility / appetite | strong | Option A é small: uma checagem de exaustão no topo do loop, uma chamada ao helper existente e um teste no módulo já pinado. |
| Strategic fit | strong | Reusa a arquitetura de 013/T158 (produção por geração + gate de drenagem) e a timeline medida; nenhum enum, nenhum subsistema novo, fronteiras preservadas. |
| Risk posture | adequate | Riscos de deadlock (guard), geração obsoleta e `complete` antecipado têm mitigações concretas; risco residual de dispositivo declarado `[NV]`. |

## Verdict & Rationale

**go**. O scorecard satisfaz o gate: problema válido e origem documentada no bug
`playback-stall-at-end-of-document`, evidência `adequate` (não `weak`/`unknown`),
conceito shaping com opção recomendada (A) e riscos compreendidos. A causa é
mecânica e verificável em host: o ramo de `segments.get() == None` retorna sem
finalizar, deixando `state.production = Some` e, portanto,
`production_running == true`, o que bloqueia `signal_document_end` no watch.
A correção reutiliza `finish_production_if_current`, que já checa geração e já
instala a timeline medida, e não antecipa a conclusão — o `complete` continua
exigindo drenagem real. A ausência de captura pareada pós-correção impede
`strong` em evidência, mas não bloqueia a especificação: a validação device
fica marcada `[NV]` e a reprodução limpa permanece como dívida explícita.

## If needs-clarification

Não aplicável — nenhuma pergunta bloqueante permanece. As sete dúvidas de
fronteira (local da checagem, soltura de guard, `started`, ausência de
`complete`, enum, testes e natureza da evidência) foram resolvidas na spec.

## If go — Handoff to `/speckit-specify`

- **Problem**: produção exaurida (`next_unit >= len()`, `pending` vazio) retorna
  sem remover `state.production`; o gate `drained && !production_running` nunca
  abre, sem `session_end` e com transporte preso em PLAYING.
- **Chosen approach**: Option A — detecção de exaustão no topo do `while` de
  `produce_bounded`, sob lock curto, com soltura do guard e chamada a
  `finish_production_if_current` seguida de `break`; manter a checagem de
  `done` pós-commit para o tick da última unidade.
- **In scope / out of scope**: inclui a checagem, a chamada ao helper e o teste
  em `ttfa_013_tests`; exclui `signal_document_end`/`dispatch_document_end`/
  `ShellVerb::Complete` no produtor, mudança em `production_running`, enum novo,
  `synth.rs`, `chunk.rs`, `Cargo.toml`, `expected-tests.txt`, HTML, capabilities,
  workflow, specs/bugs de outros ciclos e `converge.md` alheio.
- **Success metrics**: 100% dos estados exauridos com geração atual terminam em
  `production == None` após no máximo um passo; zero finalizações de geração
  obsoleta; zero chamadas de `complete` no produtor; teste no target pinado;
  checklist 23/23; rastreabilidade 12/12; dispositivo `[NV]`.
- **Carried-forward open questions**: reprodução limpa com diag zerado e
  observação >220 s em `2201117TY`, e A/B com/sem primário — ambas `[NV]`,
  owner natural: ciclo de validação em device, não esta cadeia documental.

# Idea Research: Exaustão da produção não finaliza o documento

- **Slug**: production-exhaustion
- **Created**: 2026-11-21
- **Evidence confidence (overall)**: high para a causa em host; `[NV]` para a confirmação em dispositivo (sem captura pareada pós-correção neste checkout)

## Users & Demand

- Ouvinte do documento "HAMLET trecho.md" em `2201117TY`: reprodução contínua
  até 199,203 s e então parada sem `session_end` / `action=complete`, com
  MediaSession `state=3` (PLAYING) — [source: `.specify/memory/STATE.md`, entrada "Validação visual/espelhamento no device + achado de stall"; `.specify/bugs/playback-stall-at-end-of-document/assessment.md`] (confidence: high, cited).
- O bug foi registrado com severidade medium-high e veredito "likely valid,
  needs reproduction in clean session" — [source: bug `assessment.md`] (confidence: high, cited).
- Nenhuma telemetria de múltiplos usuários foi coletada: a demanda é um relato
  único de smoke em dispositivo, corroborado por leitura estrutural do motor —
  [source: ausência observada; ASSUMPTION] (confidence: medium, assumption).

## Prior Art

- 013 TTFA/T158 introduziu `produce_bounded` (:5773-5863), o handover por
  geração (:5943-5971) e o gate de drenagem em `playback_watch_tick` (:2669):
  "fila vazia enquanto o produtor ainda tem áudio é o produtor esperando o
  dispositivo, nunca fim de documento" — [source: leitura estrutural em host] (confidence: high, cited).
- O helper `finish_production_if_current` (:6004-6018) **já** é generation-safe:
  só faz `guard.take()` quando `p.generation == generation`, e passa
  `total_samples` medido para `install_session_timeline` (:6473-6492) — [source: leitura em host] (confidence: high, cited).
- `finish_production` (:6390-6405) é o caminho irmão, sem checagem de geração,
  usado por `advance_production` (:6305-6327) quando a produção não está mais
  rodando para a geração atual. Ele é chamado no ramo `!production_running_for_generation`
  — justamente o ramo que a exaustão não alcança, porque `state.production`
  permanece `Some` — [source: leitura em host] (confidence: high, cited).
- 026 criou o primário de 160 bytes e o teste
  `t007_produce_bounded_commits_first_unit_without_draining_all_segments`
  (:11007-11061), que já constrói uma `Production` de teste com `next_unit`,
  `pending`, `total_samples` e `started` — é o ponto natural para a regressão —
  [source: `src-tauri/src/lib.rs`, módulo `ttfa_013_tests`] (confidence: high, cited).
- O bug de latência extrema (183 s) foi fechado por 026 com redução pareada para
  7506 ms; a continuidade de longa duração ficou `[NV]` e produziu o achado
  atual — [source: `.specify/memory/STATE.md`] (confidence: high, cited).

## Market & Context

- Alternativa atual do usuário: não há caminho de UI para concluir a sessão — o
  transporte permanece PLAYING e o áudio não avança; a única saída é interromper
  manualmente — [source: captura `stall_199s.png` e `dumpsys media_session`] (confidence: high, cited).
- Custo de não agir: todo documento que termina com a produção exaurida no
  mesmo tick em que o handover esvazia `pending` pode repetir o travamento; o
  ganho de TTFA de 026 fica incompleto porque o fim da leitura não é confiável —
  [source: raciocínio sobre `produce_bounded` + STATE.md] (confidence: medium, cited).

## Data & Constraints

- Causa localizada: `segments.get(production.next_unit)` retornando `None`
  (:5817-5819) executa `return Ok(())` dentro do bloco que mantém o
  `MutexGuard`; não há chamada a `finish_production_if_current` nesse caminho.
  O mesmo vale para o `break` de backpressure em :5803-5805 quando a produção
  já está exaurida e `room == 0` — [source: leitura em host] (confidence: high, cited).
- `production_running` (:6360-6365) é um simples `guard.is_some()`; enquanto a
  estrutura não for removida, o gate `drained && !production_running` (:2669)
  permanece falso e `signal_document_end` (:3215-3226) nunca é chamado — [source: leitura em host] (confidence: high, cited).
- `Production` já carrega `generation`, `segments`, `next_unit`, `pending`,
  `total_samples` e `started`; a condição de exaustão usa campos existentes.
  **Nenhum enum novo** é necessário nem desejado — [source: leitura em host + briefing do orquestrador] (confidence: high, cited).
- O helper a reutilizar faz `guard.take()` sob o mesmo lock; detectar a exaustão
  sob `MutexGuard` e chamar o helper sem soltar o guard causaria deadlock — a
  checagem deve ser feita em bloco curto e o helper chamado depois — [source: leitura de :6004-6018] (confidence: high, cited).
- Gate autoritativo `scripts/verify.sh`; Factory HEAD `b1deab1` somente leitura;
  gate 2 de diff bidirecional impede novo target e edição de
  `scripts/expected-tests.txt` — [source: briefing do orquestrador e `.specify/memory/constitution.md`] (confidence: high, cited).
- Nenhuma captura pareada antes/depois da correção existe neste checkout;
  qualquer afirmação de comportamento em dispositivo é `[NV]` — [source: ausência observada; ASSUMPTION] (confidence: high, assumption).

## Evidence Against the Idea

- O bug pode ser um problema de drenagem do backend, não de exaustão da
  produção: a hipótese registrada no bug fala em "dangling handover". A leitura
  de host mostra que o handover esvazia `pending` corretamente e que o ponto
  morto é o estado de produção não removido; ainda assim, a confirmação em
  dispositivo falta — [source: bug `assessment.md` vs. leitura de host] (confidence: medium, cited).
- O cenário exato pode exigir que o último `handover` e a drenagem ocorram no
  mesmo tick; em execuções onde sobra `pending`, o ramo de commit já finaliza e
  o bug não aparece — explica por que o smoke de 026 percorreu 199 s antes de
  travar — [source: leitura de :5842-5860] (confidence: medium, cited).
- Finalizar a produção cedo demais poderia liberar o gate de fim enquanto ainda
  existe áudio não drenado; por isso a correção NÃO DEVE chamar
  `signal_document_end`, `dispatch_document_end` ou `ShellVerb::Complete`: a
  conclusão continua condicionada a `backend.is_drained()` — [source: leitura de :2665-2677] (confidence: high, cited).
- Uma alternativa tentadora — fazer `production_running` devolver `false`
  quando exaurido — esconderia o estado preso e deixaria de instalar a timeline
  medida pelo `finish_production_if_current`; é correção de sintoma — [source: raciocínio sobre :6360-6365 e :6004-6018] (confidence: high, cited).

## Gaps & Open Questions

- [NEEDS CLARIFICATION: confirmar se a exaustão sem `started` (nenhum sample
  entregue) deve finalizar — resolvido na spec: sim, desde que a geração seja a
  atual; nenhum áudio foi produzido, mas o gate de conclusão continua exigindo
  drenagem pelo backend.]
- [NEEDS CLARIFICATION: confirmar se o orquestrador executará gates/builds/testes
  após o merge — resolvido: documentos não executam; marcado `[NV]`.]
- [NEEDS CLARIFICATION: reprodução limpa com diag zerado e observação >220 s em
  `2201117TY` permanece `[NV]` —Owner natural: ciclo de validação em device.]

## Sources

- `src-tauri/src/lib.rs` (:2628-2692, :3215-3226, :5708-5726, :5773-5863,
  :5943-5971, :6004-6026, :6305-6365, :6390-6405, :6473-6492, :11007-11061)
  — leitura estrutural em host.
- `.specify/bugs/playback-stall-at-end-of-document/{assessment,fix,test}.md` (repo local).
- `.specify/memory/STATE.md` (repo local).
- `specs/013-ttfa-synthesis-pipeline/`, `specs/026-first-useful-segment/` (repo local, não editados).
- `scripts/verify.sh` e `scripts/expected-tests.txt` (repo local, não modificados).

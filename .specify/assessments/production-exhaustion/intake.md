# Idea Intake: Exaustão da produção não finaliza o documento

- **Slug**: production-exhaustion
- **Created**: 2026-11-21
- **Source**: bug `.specify/bugs/playback-stall-at-end-of-document/` (assessment.md, fix.md, test.md) + diagnóstico de host `ScoutStallCode` + `.specify/memory/STATE.md` (entradas "Validação visual/espelhamento no device + achado de stall")
- **Type**: fix

## Idea (as captured)

> "Ciclo SDD 027: `produce_bounded` (`src-tauri/src/lib.rs:5773-5863`) chama
> `finish_production_if_current` **somente depois** de `commit_produced_unit`
> (:5834-5841) e do `handover_pending_for_generation` (:5842). Quando
> `production.segments.get(production.next_unit)` devolve `None` (:5817-5819)
> com `pending` vazio, a função retorna `Ok(())` sem finalizar:
> `state.production` fica `Some` para sempre, `production_running` (:6360-6365)
> continua `true` e `playback_watch_tick` (:2669) nunca chama
> `signal_document_end` (:3215). Resultado observado: sem `action=complete`,
> sem `session_end`, MediaSession preso em `state=3` (PLAYING) com posição
> congelada em 199,203 s. Correção proposta: detectar a exaustão no topo do
> `while` (next_unit >= segments.len() && pending.is_empty()), chamar o helper
> já existente `finish_production_if_current` (:6004-6018), que é generation-safe,
> e dar `break`. Sem enum novo, sem `complete` antecipado, teste no target
> já pinado (`ttfa_013_tests` / `t007`)."

## Restated

Quando a produção termina de entregar todas as unidades de um documento e não
há áudio pendente, o estado de produção deve ser encerrado para que o caminho
já existente de fim de documento possa ser acionado pelo backend drenado. Hoje
o encerramento só acontece no ramo que acabou de commitar uma unidade; o ramo
que encontra a produção já exaurida retorna silenciosamente e deixa o transporte
preso em PLAYING. A correção é local, reusa o helper generation-safe existente
e não antecipa o sinal de conclusão — ela apenas remove o bloqueio do gate.

## Origin & Context

- **Raised by**: smoke de longa duração do ciclo 026 no dispositivo `2201117TY`
  (APK com o primário de 160 bytes). A reprodução correu de 0 s a 199,203 s com
  787 entradas de diagnóstico, sem FATAL/ANR/underrun; a posição então parou por
  mais de 100 s. [source: `.specify/memory/STATE.md`, entrada "Validação visual/espelhamento no device + achado de stall"] (confidence: high, cited).
- **Trigger**: o bug `playback-stall-at-end-of-document` foi registrado como
  novo bug, com veredito "likely valid, needs reproduction in clean session" e
  severidade medium-high; a hipótese inicial (dangling handover do primário) foi
  refinada por leitura estrutural em host para a causa de exaustão em
  `produce_bounded`. [source: `.specify/bugs/playback-stall-at-end-of-document/assessment.md` e diagnóstico `ScoutStallCode`] (confidence: high, cited).
- **Custo observado**: sessão que termina o áudio mas nunca emite
  `session_end`, deixando o ouvinte diante de um controle de pause ativo e de um
  waveform animado, sem conclusão. [source: `stall_199s.png`, `dumpsys media_session state=3`] (confidence: high, cited).

## First-Glance Unknowns

- [NEEDS CLARIFICATION: onde detectar a exaustão — no topo do `while` ou apenas no `else` de `segments.get()`? O ramo de `room == 0` (:5803-5805) também pode dar `break` antes do `get()`, então a detecção no `else` cobre menos casos.]
- [NEEDS CLARIFICATION: como chamar `finish_production_if_current` sem manter o `MutexGuard` de `state.production` vivo? O helper precisa tomar o mesmo lock.]
- [NEEDS CLARIFICATION: a finalização deve ser condicionada a `p.started`? A exaustão sem start (nenhum áudio entregue) também precisa liberar o gate?]
- [NEEDS CLARIFICATION: o que garante que nenhum `action=complete`/`session_end` seja antecipado? A conclusão continua exigindo `backend.is_drained()` em `playback_watch_tick:2669`?]
- [NEEDS CLARIFICATION: nenhum enum novo — como expressar o estado "exaurido" sem adicionar variante a `Production` ou a outro enum?]
- [NEEDS CLARIFICATION: onde alocar o teste de regressão sem novo target e sem alterar `scripts/expected-tests.txt`?]
- [NEEDS CLARIFICATION: que evidência de dispositivo pode ser reivindicada neste checkout, dado que o bug foi visto em `2201117TY` mas a causa é verificável em host?]

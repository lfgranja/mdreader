# Problem Definition: Exaustão da produção não finaliza o documento

- **Slug**: production-exhaustion
- **Created**: 2026-11-21
- **Inputs used**: intake.md | research.md | diagnóstico de host | user input (bug registrado)

## Problem Statement

Quando `produce_bounded` (`src-tauri/src/lib.rs:5773-5863`) encontra a produção
já exaurida — `next_unit >= segments.len()` e `pending` vazio — ele retorna
`Ok(())` sem remover `state.production`. Como `production_running` (:6360-6365)
é apenas `guard.is_some()`, o gate de fim de documento em `playback_watch_tick`
(:2669, `drained && !production_running`) permanece falso mesmo depois de o
backend drenar: `signal_document_end` (:3215) nunca é chamado, não há
`action=complete` nem `session_end`, e o transporte fica em PLAYING com a
posição congelada. Isso importa agora porque o ciclo 026 entregou o primário de
160 bytes e reduziu o TTFA pareado de 183089 ms para 7506 ms, mas o smoke de
longa duração revelou que o fim da leitura não é confiável — o ganho de início
não fecha o ciclo de reprodução.

## Affected Users & Stakeholders

- **Users**: ouvintes que reproduzem um documento até o fim, especialmente em
  Android (`2201117TY`), que veem pause ativo e waveform animado após o áudio
  acabar, sem conclusão da sessão.
- **Stakeholders**: mantenedor do motor de produção/síntese (013/T158), o autor
  do bug `playback-stall-at-end-of-document`, o orquestrador do ciclo (gate
  `scripts/verify.sh`) e o usuário que fez o smoke — todos interessados numa
  correção pequena, verificável em host e reversível.

## Goals

- Garantir que toda exaustão da produção para a geração atual seja detectada
  antes de qualquer `break`/`return` do loop de `produce_bounded`, com a
  finalização feita pelo helper `finish_production_if_current` (:6004-6018).
- Manter a finalização generation-safe: nenhuma geração obsoleta pode ser
  removida; nenhum lock pode ser mantido ao chamar o helper (evitar deadlock).
- Não antecipar `complete`: `action=complete` e `session_end` continuam
  condicionados a `backend.is_drained()` no watch; a correção apenas libera o
  gate removendo a produção exaurida.
- Não introduzir enum novo nem mudar o contrato de áudio, o primário de 026, o
  handover, o backpressure ou a semântica incremental do produtor.
- Cobrir a regressão no módulo inline `ttfa_013_tests` do target já pinado e
  manter toda evidência de dispositivo marcada `[NV]`.

## Non-Goals

- Reescrita de `produce_bounded`, `handover_pending_for_generation`,
  `stream_synthesis`, `synth.rs`, `chunk.rs` ou do contrato PCM mono f32 22050 Hz.
- Mudança em `production_running` como forma de esconder a exaustão (correção
  de sintoma), emissão direta de `signal_document_end` / `dispatch_document_end`
  / `ShellVerb::Complete` a partir do produtor, ou novo enum de estado.
- Reexecução da reprodução em dispositivo ou A/B com/sem primário nesta cadeia
  documental (`[NV]`); reprodução limpa com diag zerado é pré-requisito do bug.
- Edição de `scripts/verify.sh`, `scripts/expected-tests.txt`, `Cargo.toml`,
  HTML, capabilities, workflow, specs/bugs de outros ciclos ou `converge.md`
  de outras specs.
- Execução de gates, builds, testes, formatadores, `git add` ou `git commit`
  por subagentes.

## Success Metrics

- 100% dos estados de exaustão detectados na entrada do loop
  (`next_unit >= segments.len() && pending.is_empty()`, geração atual) resultam
  em `state.production == None` após, no máximo, um passo de `produce_bounded`
  (baseline host: 0% — estrutura permanece `Some`).
- 100% dos casos com geração divergente preservam a produção (zero `take()` de
  geração obsoleta); zero deadlocks por guard retido na chamada ao helper.
- Zero chamadas a `signal_document_end` / `dispatch_document_end` /
  `ShellVerb::Complete` originadas de `produce_bounded`; conclusão em device
  (`action=complete` + `session_end`) permanece `[NV]`.
- 100% dos testes de regressão no módulo `ttfa_013_tests` do target pinado;
  nenhum target novo, nenhuma alteração em `expected-tests.txt`, nenhum enum novo.
- Checklist de requisitos 23/23 `[x]`, 6/6 FR e 6/6 SC rastreados a tarefas e
  símbolos reais em `analyze.md`; nenhum marcador de ambiguidade remanescente.

## Cost of Inaction

Cada documento que esgota a produção no mesmo instante em que o handover esvazia
a fila deixa a sessão presa em PLAYING: sem `session_end`, sem limpeza de
bookmark de conclusão e sem avanço de capítulo em leitura de diretório. O
primário de 026 melhora o início, mas a experiência termina sem fechamento — e
o bug `playback-stall-at-end-of-document` continua aberto, exigindo reprodução
limpa em device para ser dado como resolvido.

## Open Questions

- Nenhuma bloqueante. As dúvidas de local da checagem, soltura de guard,
  `started`, ausência de `complete` antecipado, enum, alocação de testes e
  natureza da evidência foram respondidas em `## Clarifications` da spec 027.
- Permanecem `[NV]`: reprodução limpa (>220 s, diag zerado) e A/B com/sem
  primário em `2201117TY` — fora do alcance desta cadeia documental.

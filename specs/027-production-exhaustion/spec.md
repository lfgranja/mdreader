# Feature Specification: Exaustão da produção não finaliza o documento (Ciclo 027)

**Feature Branch**: `027-production-exhaustion`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "Ciclo SDD 027: corrigir o bug `playback-stall-at-end-of-document`, origem da cadeia documental. `produce_bounded` (`src-tauri/src/lib.rs:5773-5863`) só chama `finish_production_if_current` depois de `commit_produced_unit`. Quando `segments.get(next_unit)` retorna `None` (:5817-5819) com `pending` vazio, a função retorna sem finalizar: `state.production` fica `Some`, `production_running` (:6360-6365) continua `true` e `playback_watch_tick` (:2669) nunca chama `signal_document_end` — sem `action=complete` nem `session_end`, PLAYING travado em 199,203 s. Detectar a exaustão antes do break, reutilizar o helper generation-safe, não antecipar `complete`, testar no target pinado, sem enum novo. Dispositivo `[NV]`."

## Clarifications

### Session 2026-11-21

- Q: Onde a exaustão deve ser detectada? → A: no topo do `while` de `produce_bounded` (:5782), antes de `queue_room`, do ramo `has_pending` (:5784-5802), do `break` de backpressure (:5803-5805) e de qualquer `return`. A checagem de `done` já existente após o commit (:5847-5860) é mantida para finalizar no mesmo tick da última unidade.
- Q: Como chamar o helper sem deadlock? → A: a checagem de exaustão roda em bloco curto com o `MutexGuard` de `state.production`; o guard é solto antes de chamar `finish_production_if_current` (:6004-6018), que toma o mesmo lock. Lock envenenado não dispara finalização.
- Q: A finalização exige `started == true`? → A: não. A condição é `next_unit >= segments.len() && pending.is_empty()` com a geração atual. Exaustão sem start (documento vazio ou falha anterior) também libera o gate; a conclusão real continua dependendo da drenagem do backend.
- Q: O produtor pode emitir `complete`? → A: não. `produce_bounded` NÃO DEVE chamar `signal_document_end`, `dispatch_document_end` nem `ShellVerb::Complete`. O gate `drained && !production_running` (:2669) permanece a única decisão de fim; `action=complete` e `session_end` continuam exigindo `backend.is_drained()`.
- Q: É necessário um enum novo para representar "exaurido"? → A: não. A condição usa apenas campos já presentes em `Production` (`generation`, `segments`, `next_unit`, `pending`); nenhum enum ou tipo público novo é introduzido.
- Q: Onde ficam os testes? → A: no módulo inline `ttfa_013_tests` do target já pinado (`src-tauri/src/lib.rs`, ao lado de `t007_produce_bounded_commits_first_unit_without_draining_all_segments` :11007-11061). Nenhum novo target e nenhuma alteração em `scripts/expected-tests.txt`.
- Q: Que evidência de dispositivo é reivindicada? → A: nenhuma nesta cadeia. A causa é mensurável em host; `action=complete`/`session_end` observados em `2201117TY` são `[NV]` e dependem de reprodução limpa (diag zerado, >220 s). Gates, builds e testes são executados pelo orquestrador, não por esta especificação.

## User Scenarios & Testing *(mandatory)*

<!--
  Histórias priorizadas pela jornada do ouvinte. Cada uma é testável de forma
  independente em host por inspeção estrutural ou teste no target pinado;
  nenhuma afirmação de comportamento em dispositivo é feita aqui.
-->

### User Story 1 - Fim de documento conclui quando a produção se esgota (Priority: P1)

Quando o ouvinte reproduce um documento até o fim, a sessão deve concluir com
`session_end` em vez de ficar presa em PLAYING com o progresso congelado. Hoje,
se a última unidade já foi entregue e `pending` esvaziou no mesmo tick em que o
loop verifica o trabalho restante, `produce_bounded` retorna sem remover a
produção e o gate de fim nunca abre.

**Why this priority**: é a falha relatada no bug — sem conclusão, o ouvinte vê
pause ativo e waveform animado muito depois de o áudio ter terminado (199,203 s
no smoke do ciclo 026).

**Independent Test**: em host, construir uma `Production` com `next_unit ==
segments.len()`, `pending` vazio e geração atual, chamar `produce_bounded` uma
vez e verificar que `state.production` passou a `None`; no baseline, permanece
`Some`.

**Acceptance Scenarios**:

1. **Given** uma produção exaurida para a geração atual, **When** o loop de
   `produce_bounded` inicia uma iteração, **Then** a exaustão é detectada antes
   de qualquer `return`/`break` e o helper de finalização é chamado.
2. **Given** a exaustão detectada, **When** o guard é solto e o helper atua,
   **Then** `state.production == None` e a timeline medida é instalada, sem
   chamada a `signal_document_end` pelo produtor.
3. **Given** o backend drenado e nenhuma produção pendente, **When** o
   `playback_watch_tick` roda, **Then** o gate `drained && !production_running`
   abre e o caminho de fim existente é seguido; a observação em dispositivo é
   `[NV]`.

---

### User Story 2 - Finalização generation-safe e sem deadlock (Priority: P1)

A detecção de exaustão não pode encerrar a produção de uma geração obsoleta nem
concorrer com o lock usado pelo próprio helper de finalização.

**Why this priority**: a invalidação de geração é o mecanismo que protege troca
de documento, cancelamento e retomada; uma finalização cruzada corromperia a
sessão, e um guard retido travaria o produtor.

**Independent Test**: em host, com `next_unit == len()` e `pending` vazio mas
geração divergente, verificar que a produção não é removida; com geração
atual, verificar que a chamada ao helper ocorre após a soltura do guard.

**Acceptance Scenarios**:

1. **Given** exaustão com `production.generation != generation`, **When**
   `produce_bounded` executa, **Then** nenhuma finalização ocorre e a estrutura
   permanece intacta para a geração correta.
2. **Given** lock envenenado em `state.production`, **When** a checagem de
   exaustão falha em obter o guard, **Then** a função retorna sem finalizar —
   nenhum `take()` é feito com estado inválido.
3. **Given** exaustão com geração atual, **When** a checagem termina, **Then**
   o `MutexGuard` já foi solto antes de `finish_production_if_current` tomar o
   mesmo lock; zero deadlocks por reentrância.

---

### User Story 3 - Sem `complete` antecipado e sem enum novo (Priority: P2)

A correção apenas libera o gate de fim; ela não antecipa a conclusão nem cria
um novo estado de produção.

**Why this priority**: `action=complete` e `session_end` são contratos do
backend e do shell; emiti-los com áudio ainda não drenado fecharia a sessão
cedo. Um enum novo, além de desnecessário, criaria uma segunda origem de
verdade para o estado da produção.

**Independent Test**: em host, inspecionar o diff e confirmar que
`produce_bounded` não contém chamadas a `signal_document_end`,
`dispatch_document_end` ou `ShellVerb::Complete`, que nenhum enum/variante nova
aparece e que `playback_watch_tick:2669` permanece como único gate de fim.

**Acceptance Scenarios**:

1. **Given** a produção recém-finalizada, **When** o backend ainda tem áudio
   na fila, **Then** nenhum `action=complete` é emitido; a conclusão espera
   `backend.is_drained()`.
2. **Given** o diff do ciclo, **When** os símbolos adicionados são enumerados,
   **Then** não há enum novo, nem mudança em `production_running`,
   `handover_pending_for_generation`, `commit_produced_unit` ou `queue_room`.
3. **Given** o primário de 160 bytes do ciclo 026, **When** a exaustão é
   tratada, **Then** a segmentação, o handover e o contrato PCM mono `f32`
   @ 22050 Hz permanecem intactos.

---

### User Story 4 - Evidência em host e limites do ciclo (Priority: P2)

O mecanismo deve ser verificável no target de testes existente, com os limites
de verificação declarados explicitamente.

**Why this priority**: o sintoma foi visto em dispositivo, mas a causa e o
contrato são verificáveis em host; declarar `[NV]` evita reivindicar prova que
não foi obtida neste checkout.

**Independent Test**: em host, percorrer `analyze.md` e confirmar que cada FR
tem tarefa e símbolo correspondentes, que o teste está em `ttfa_013_tests`, que
`expected-tests.txt` está intacto e que toda afirmação de dispositivo está
marcada `[NV]`.

**Acceptance Scenarios**:

1. **Given** os requisitos FR-027-001 a FR-027-006, **When** o relatório de
   análise é lido, **Then** todos estão mapeados a tarefas T001..T014 e aos
   símbolos reais do código.
2. **Given** o teste de regressão, **When** o target é enumerado, **Then** ele
   aparece no módulo inline `ttfa_013_tests` e nenhum arquivo de teste novo ou
   entrada em `scripts/expected-tests.txt` é criado.
3. **Given** a evidência disponível, **When** o resultado é relatado, **Then**
   toda afirmação dependente de dispositivo é `[NV]`, e nenhum gate, build,
   teste ou formatador é atribuído a esta cadeia documental.

---

### Edge Cases

- **Exaustão no topo do loop**: `next_unit == len()` e `pending` vazio na
  primeira iteração — deve finalizar imediatamente, antes de `queue_room`.
- **Exaustão com `room == 0`**: o `break` de backpressure (:5803-5805) não pode
  ser atingido com a produção exaurida sem que a finalização tenha ocorrido —
  é o motivo da checagem no topo.
- **Última unidade commitada no mesmo tick**: a checagem de `done` pós-commit
  (:5847-5860) continua finalizando; a nova checagem não a substitui nem duplica
  efeito (o helper é idempotente por `take()`).
- **`pending` não vazio**: o produtor NÃO está exaurido; o handover existente
  (:5943-5971) continua drenando a fila antes da finalização.
- **Geração divergente**: nenhuma finalização; a produção da geração corrente
  não é tocada.
- **Geração zero / `max_units == 0`**: `produce_bounded` já retorna cedo
  (:5778-5780); nenhuma mudança nesse comportamento.
- **Lock envenenado**: nenhuma finalização e nenhuma escalada de erro — a
  função retorna `Ok(())` como nos demais ramos de falha de lock.
- **`started == false` com exaustão**: libera o gate, mas a conclusão real
  permanece subordinada à drenagem do backend.
- **Documento vazio / nenhum segmento**: mesma condição de exaustão; nenhum
  priming, nenhum commit, nenhuma antecipação de `complete`.
- **Leitura de diretório**: `advance_chapter_queue` (:2863) continua sendo
  decidido no watch após a drenagem; o produtor não escolhe capítulo.
- **Reabertura do documento**: nova geração produz nova estrutura; a checagem
  generation-safe impede que a exaustão antiga encerre a nova sessão.
- **Primário do ciclo 026**: a unidade curta e a reindexação não são
  reprocessadas; a correção atua somente no encerramento da produção.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-027-001**: O sistema DEVE detectar a exaustão da produção no topo do
  loop de `produce_bounded` (`src-tauri/src/lib.rs:5782`), antes de
  `queue_room`, do ramo `has_pending`, do `break` de backpressure e de qualquer
  `return` antecipado. A condição DEVE ser `next_unit >= segments.len()` e
  `pending.is_empty()` para a geração atual. A checagem de `done` já existente
  após o commit (:5847-5860) DEVE ser preservada.

- **FR-027-002**: Quando a exaustão for detectada, o sistema DEVE soltar o
  `MutexGuard` de `state.production` e DEVE chamar
  `finish_production_if_current` (:6004-6018), seguido de `break`. A finalização
  DEVE ocorrer somente se `p.generation == generation`; gerações divergentes,
  lock envenenado e estrutura ausente NÃO DEVEM ser finalizados. NENHUM novo
  mecanismo de travamento ou estado DEVE ser introduzido.

- **FR-027-003**: O sistema NÃO DEVE antecipar a conclusão da sessão:
  `produce_bounded` NÃO DEVE chamar `signal_document_end`,
  `dispatch_document_end` ou `ShellVerb::Complete`. O gate
  `drained && !production_running` em `playback_watch_tick` (:2669) DEVE
  permanecer como a única decisão de fim, de modo que `action=complete` e
  `session_end` continuem exigindo `backend.is_drained()`.

- **FR-027-004**: NENHUM enum, variante ou tipo público novo DEVE ser criado;
  a condição de exaustão DEVE usar somente campos já existentes de
  `Production`. `production_running` (:6360-6365), `advance_production`
  (:6305-6327), `handover_pending_for_generation` (:5943-5971),
  `commit_produced_unit`, `queue_room` (:6331-6341), o primário de 026 e o
  contrato PCM mono `f32` @ 22050 Hz NÃO DEVEM ser alterados.

- **FR-027-005**: Os testes do ciclo DEVEM residir no módulo inline
  `ttfa_013_tests` do target já pinado e DEVEM cobrir, no mínimo: (a) exaustão
  com geração atual e `started == true` resultando em `production == None`;
  (b) geração divergente preservando a produção; (c) `pending` não vazio não
  finalizando; (d) nenhuma emissão de `complete` pelo produtor. NENHUM novo
  target e NENHUMA alteração em `scripts/expected-tests.txt` DEVEM ocorrer.

- **FR-027-006**: O diff de código DEVE ficar confinado a
  `src-tauri/src/lib.rs`. NENHUMA mudança DEVE ser feita em `synth.rs`,
  `chunk.rs`, `Cargo.toml`, HTML, capabilities, workflow, `scripts/verify.sh`,
  `scripts/expected-tests.txt`, specs/bugs de outros ciclos ou `converge.md`
  alheio. Gates, builds, testes e formatadores são executados pelo
  orquestrador; toda evidência de comportamento em dispositivo DEVE ser
  marcada `[NV]`.

### Key Entities *(include if feature involves data)*

- **Produção exaurida**: estado em que `next_unit >= segments.len()` e
  `pending.is_empty()` para a geração atual — todas as unidades já foram
  produzidas e entregues ao handover.
- **`finish_production_if_current`**: helper já existente (:6004-6018) que
  remove a estrutura sob checagem de geração e instala a timeline medida via
  `install_session_timeline` (:6473-6492).
- **Gate de drenagem**: condição `backend.is_drained() &&
  !commands::production_running(state)` em `playback_watch_tick` (:2669),
  único caminho para `signal_document_end`.
- **Geração**: identificador `u64` da sessão de reprodução; protege contra
  finalização cruzada quando o documento muda.
- **Teste de regressão**: caso adicionado ao módulo `ttfa_013_tests` do target
  pinado, usando a mesma construção de `Production` de `t007` (:11007-11061).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-027-001**: 100% dos estados de exaustão com geração atual detectados na
  entrada do loop resultam em `state.production == None` após, no máximo, um
  passo de `produce_bounded`, com a timeline medida instalada (baseline host:
  0% — a estrutura permanece `Some` e o gate nunca abre). Mensurável no teste
  do target pinado.

- **SC-027-002**: 100% dos casos de geração divergente, lock envenenado ou
  estrutura ausente preservam a produção (zero `take()` indevido); zero
  deadlocks na chamada ao helper, por inspeção da ordem de soltura do guard.

- **SC-027-003**: zero chamadas a `signal_document_end`,
  `dispatch_document_end` ou `ShellVerb::Complete` originadas de
  `produce_bounded`; 100% das conclusões permanecem no gate
  `drained && !production_running`. A emissão de `action=complete` e
  `session_end` em `2201117TY` é `[NV]` e exige reprodução limpa (>220 s).

- **SC-027-004**: 100% dos comportamentos existentes preservados — contagem de
  unidades de `t007`, handover por geração, backpressure de `queue_room`,
  primário de 026 e contrato PCM inalterados; zero novos enums e zero mudanças
  em `production_running`, `advance_production`, `synth.rs` ou `chunk.rs`.

- **SC-027-005**: 100% dos testes novos no módulo `ttfa_013_tests`; zero novos
  targets; `scripts/expected-tests.txt` byte-idêntico; diff de código restrito
  a `src-tauri/src/lib.rs`.

- **SC-027-006**: 100% dos 6 requisitos funcionais e 100% dos 6 critérios de
  sucesso rastreados a tarefas e símbolos reais em `analyze.md`; checklist de
  requisitos 23/23 itens `[x]`; veredito `go` no assessment; nenhum marcador de
  ambiguidade remanescente e `[NV]` explícito para toda afirmação de dispositivo.

> Todas as SCs são mensuráveis em host por inspeção estrutural ou por testes no
> target pinado. O comportamento efetivamente observado em dispositivo é `[NV]`
> e não é reivindicado como verificado nesta cadeia documental.

## Assumptions

- A causa diagnosticada em host — retorno antecipado em `segments.get() ==
  None` sem finalização — é suficiente para especificar a correção; a
  confirmação em dispositivo depende de reprodução limpa com diag zerado e
  observação além de 220 s (`[NV]`).
- `finish_production_if_current` continua generation-safe e idempotente por
  `take()`; chamá-lo duas vezes no mesmo tick (topo do loop e pós-commit) não
  duplica efeito.
- Exaustão com `started == false` é legítima e deve liberar o gate; nenhum
  áudio foi entregue, mas a drenagem do backend continua decidindo a conclusão.
- O módulo `ttfa_013_tests` aceita mais um caso sem alterar a lista canônica de
  testes, respeitando o gate 2 de diff bidirecional.
- `specs/013`, `specs/026`, `.specify/bugs/playback-stall-at-end-of-document`
  e `converge.md` de outros ciclos são dados de contexto; nenhum deles é
  editado por este ciclo. O bug permanece registrado como origem, sem reescrita.
- Gates, builds, testes e formatadores são executados pelo agente principal;
  nenhuma afirmação sobre o resultado de `scripts/verify.sh` é feita por esta
  cadeia documental.

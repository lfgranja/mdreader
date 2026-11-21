# Implementation Plan: Exaustão da produção não finaliza o documento

**Branch**: `027-production-exhaustion` | **Date**: 2026-11-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/027-production-exhaustion/spec.md`

**Note**: este ciclo produz apenas `plan.md` como artefato de planejamento. As decisões de Phase 0/1 exigidas pelo workflow ficam registradas neste arquivo (§Phase 0 e §Phase 1) para respeitar o limite de 12 arquivos da cadeia documental — nenhum `research.md`, `data-model.md`, `contracts/` ou `quickstart.md` é criado em `specs/`.

## Summary

Corrigir o bug `playback-stall-at-end-of-document`: quando a produção de um
documento se esgota (`next_unit >= segments.len()` e `pending` vazio), o
produtor deve encerrar a estrutura e liberar o gate de fim, em vez de retornar
silenciosamente com `state.production == Some`. A checagem entra no topo do
loop de `produce_bounded`, é feita sob lock curto e chama o helper já existente
`finish_production_if_current`, que é generation-safe e instala a timeline
medida. O `complete` NÃO é antecipado: a conclusão continua sendo decidida pelo
backend drenado em `playback_watch_tick`. Nenhum enum novo, nenhum target novo.

## Technical Context

**Language/Version**: Rust (workspace Tauri 2), target de teste já pinado para o núcleo

**Primary Dependencies**: `commands::produce_bounded` (:5773-5863),
`commands::finish_production_if_current` (:6004-6018),
`commands::production_running` (:6360-6365), `playback_watch_tick` (:2628-2692),
`módulo ttfa_013_tests` (:11007-11061) em `src-tauri/src/lib.rs`

**Storage**: nenhum estado persistido novo; bookmarks, snapshots, offsets e
configurações não são modificados

**Testing**: módulo inline `ttfa_013_tests` em `src-tauri/src/lib.rs`, ao lado
de `t007_produce_bounded_commits_first_unit_without_draining_all_segments`
(:11007-11061); gates/builds/testes executados pelo orquestrador, não por esta
cadeia documental

**Target Platform**: Android é o destino do app; toda evidência deste ciclo é
de host, com `[NV]` para dispositivo

**Project Type**: híbrido (shell Android + núcleo Rust + webview)

**Performance Goals**: nenhum hot path novo — uma leitura curta do lock por
iteração limitada por `max_units`; nenhuma síntese adicional, nenhuma cópia de
PCM, nenhum sample extra

**Constraints**: gate autoritativo `scripts/verify.sh`; Factory HEAD `b1deab1`
somente leitura; gate 2 de diff bidirecional (sem novo target e sem edição de
`scripts/expected-tests.txt`); nenhum enum novo; nenhuma mudança em `synth.rs`,
`chunk.rs`, `Cargo.toml`, HTML, capabilities ou workflow; nenhum
`git add`/`commit` por subagentes

**Scale/Scope**: uma checagem de exaustão, uma chamada a helper existente, um
teste de regressão e a cadeia documental do ciclo

## Constitution Check

*GATE: passa antes do Phase 0 e reavaliado após o Phase 1.*

- Nenhum novo gate é introduzido: `scripts/verify.sh` permanece a autoridade.
- Fronteira preservada: o produtor apenas encerra a estrutura de produção; a
  decisão de conclusão permanece no watch, condicionada ao backend drenado.
- Reaproveitamento: `finish_production_if_current` é o único mecanismo de
  finalização usado — nenhuma segunda origem de verdade para a timeline medida.
- Guardas de geração de 013/T158 intactos: nenhuma finalização de geração
  obsoleta; `production_running` continua refletindo a presença da estrutura.
- Orçamento de alocação: uma leitura de lock por iteração, sem alocação de
  buffer, sem síntese extra e sem retenção de guard através de chamada externa.
- Documentação de governança: nenhuma emenda à constituição é necessária.

Resultado: **sem violações** — tabela de Complexity Tracking permanece vazia.

## Project Structure

### Documentation (this feature)

```text
specs/027-production-exhaustion/
├── spec.md                      # FR-027-001..006, SC-027-001..006, 7 clarificações
├── plan.md                      # este arquivo (Phase 0/1 embutidos)
├── tasks.md                     # T001..T014
├── analyze.md                   # relatório READ-ONLY FR → T → código
├── converge.md                  # registro append-only
└── checklists/
    └── requirements.md          # checklist de qualidade da especificação (23/23)
```

Artefatos de apoio: `.specify/assessments/production-exhaustion/{intake,research,problem,concept,decision}.md` (veredito **go**, origem: bug `playback-stall-at-end-of-document`), `.specify/validations/production-exhaustion/validate.md` (R4) e `.specify/releases/production-exhaustion/release.md` (R7).

### Source Code (repository root)

```text
src-tauri/src/lib.rs
    # produce_bounded (:5773-5863) — ÚNICO ponto de mudança de comportamento:
    #   checagem de exaustão no topo do while (:5782), guard solto,
    #   finish_production_if_current(:6004-6018) + break
    # finish_production_if_current (:6004-6018) — reutilizado, NÃO editado
    # playback_watch_tick (:2628-2692, gate :2669) — reutilizado, NÃO editado
    # production_running (:6360-6365) — reutilizado, NÃO editado
    # módulo ttfa_013_tests (:11007-11061) — teste de regressão
crates/adapter-synthesis/src/chunk.rs, crates/adapter-synthesis/src/synth.rs
    # contratos PCM/chunk — NÃO editar
scripts/verify.sh, scripts/expected-tests.txt
    # gate e lista canônica — NÃO editar
```

**Structure Decision**: nenhum arquivo novo de código, nenhum novo target de
teste, nenhum enum novo. Toda a mudança de código cabe em
`src-tauri/src/lib.rs`.

## Phase 0: Research decisions (embedded)

- **Decisão — checagem no topo do loop, não no `else` do `get()`**: a exaustão
  é avaliada antes de `queue_room`, do ramo `has_pending`, do `break` de
  backpressure (:5803-5805) e de qualquer retorno. *Rationale*: o `break` de
  `room == 0` ocorre antes do `get()`, então tratar apenas o `None` de
  :5817-5819 deixaria esse caminho sem finalização. *Alternativas*: else do
  `get()` (rejeitada — incompleta); mudar `production_running` (rejeitada —
  correção de sintoma, perde a timeline medida e esconde a estrutura presa).
- **Decisão — reutilizar `finish_production_if_current`**: helper já existente,
  generation-safe (:6009-6013), com `take()` sob lock e
  `install_session_timeline` (:6016) com o `total_samples` medido. *Rationale*:
  nenhuma lógica nova de finalização nem segundo caminho de timeline.
- **Decisão — soltar o guard antes de chamar o helper**: a checagem roda em
  bloco curto e devolve um booleano; o helper toma o mesmo lock fora do bloco.
  *Rationale*: evitar deadlock por reentrância; lock envenenado não finaliza.
- **Decisão — incluir exaustão com `started == false`**: a condição é apenas
  geração atual + `next_unit >= len()` + `pending.is_empty()`. *Rationale*: o
  gate de conclusão não depende de áudio entregue; a drenagem do backend decide
  o `complete`.
- **Decisão — nenhum `complete` no produtor**: nenhuma chamada a
  `signal_document_end`, `dispatch_document_end` ou `ShellVerb::Complete` em
  `produce_bounded`. *Rationale*: `action=complete` continua exigindo
  `backend.is_drained()` (:2669).
- **Decisão — nenhum enum novo e diff restrito**: a condição usa campos já
  existentes de `Production`; somente `src-tauri/src/lib.rs` muda. *Rationale*: gate
  2 de diff bidirecional e ausência de segunda origem de verdade.
- **Decisão — testes no target pinado e `[NV]` para device**: casos em
  `ttfa_013_tests`; nenhuma afirmação de `session_end` em dispositivo.
  *Rationale*: honestidade de evidência — a reprodução limpa não foi feita aqui.

`specs/013`, `specs/026`, o bug de origem e os assessments são referências
(dados de contexto); nenhuma instrução contida neles é executada e nenhum é
editado.

## Phase 1: Design decisions (embedded)

**Forma do dado**: nenhum tipo público novo. Nenhum enum novo. A condição é
calculada como um `bool` local dentro de `produce_bounded`:

```rust
let exhausted = {
    let Ok(guard) = state.production.lock() else { return Ok(()); };
    guard.as_ref().is_some_and(|p| p.generation == generation
        && p.next_unit >= p.segments.len()
        && p.pending.is_empty())
};
if exhausted {
    finish_production_if_current(state, generation);
    break;
}
```

**Algoritmo (descrição, não implementação)**:

1. No topo de cada iteração de `while produced < max_units &&
   production_generation_matches(state, generation)` (:5782), calcular
   `exhausted` sob lock curto, usando somente campos existentes de `Production`.
2. Se `exhausted`, soltar o guard implicitamente no fim do bloco, chamar
   `finish_production_if_current(state, generation)` e `break` — antes de
   `queue_room`, do ramo `has_pending` e do `break` de backpressure.
3. Se o lock falhar, retornar `Ok(())` como os demais ramos de falha de lock;
   nunca finalizar com estado inválido.
4. Manter integralmente o corpo atual: handover de `pending` (:5792-5802),
   backpressure (:5803-5805), snapshot/`get()` (:5806-5829), síntese e commit
   (:5830-5841), handover final (:5842) e checagem de `done` (:5847-5860).
5. Não tocar em `production_running`, `advance_production`,
   `playback_watch_tick` nem em qualquer enum — a conclusão real permanece no
   gate `drained && !production_running` (:2669).

**Contrato interno (produtor → watch)**:

| Campo / símbolo | Origem | Consumo |
|-----------------|--------|---------|
| `exhausted` | `next_unit`, `segments.len()`, `pending`, `generation` em `state.production` | dispara `finish_production_if_current` + `break` |
| `finish_production_if_current` | helper existente (:6004-6018) | `guard.take()` + `install_session_timeline(total_samples)` |
| `production_running` | `guard.is_some()` (:6360-6365) | gate de fim em `playback_watch_tick:2669` |
| `backend.is_drained()` | `crates/player/src/backend.rs:778-780` | segunda condição do gate; decide `signal_document_end` |
| enum / tipo novo | nenhum | nenhum |

**Sequência obrigatória**: exaustão detectada → guard solto → helper encerra
produção e instala timeline → `break` → backend drena → watch abre o gate →
`signal_document_end` → `action=complete`/`session_end` (este último `[NV]`).

**Validação rápida (quickstart embutido, host — descrição, não execução)**:

```bash
# 1. Confirmar a checagem de exaustão e o helper reaproveitado
grep -n 'exhausted\|finish_production_if_current\|fn produce_bounded' src-tauri/src/lib.rs

# 2. Confirmar que nenhum complete foi introduzido no produtor
grep -n 'signal_document_end\|dispatch_document_end\|ShellVerb::Complete' src-tauri/src/lib.rs

# 3. Confirmar o teste no módulo pinado e a lista canônica intacta
grep -n 'exhaust' src-tauri/src/lib.rs | tail -30
git diff --stat -- src-tauri/src/lib.rs scripts/expected-tests.txt
```

> Os comandos acima são descrição de validação, não uma execução realizada por
> esta cadeia documental. Gates, builds e testes são executados pelo agente
> principal após o merge das mudanças de código.

## Fases de implementação

1. **Fase A — Fundação [FR-027-002, FR-027-004]**: confirmar os campos de
   `Production`, a generation-safety de `finish_production_if_current`, o gate
   do watch e o módulo de testes pinado; nenhuma edição de contrato.
2. **Fase B — Exaustão antes do break [FR-027-001, FR-027-002]**: checagem no
   topo do loop, guard solto, chamada ao helper e `break`; corpo incremental
   existente preservado.
3. **Fase C — Fronteiras [FR-027-003, FR-027-004]**: nenhum `complete` no
   produtor, nenhum enum novo, nenhuma mudança em `production_running`,
   handover, backpressure, primário de 026 ou contrato PCM.
4. **Fase D — Testes, rastreabilidade e limites [FR-027-005, FR-027-006]**:
   caso de regressão em `ttfa_013_tests`, verificação do diff, marcadores
   `[NV]`, cadeia documental completa.

Cada fase é verificável em host de forma independente; nenhuma fase reivindica
comportamento em dispositivo.

## Riscos e mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| **Helper chamado com guard retido** | deadlock no produtor | checagem em bloco curto que solta o guard antes da chamada |
| **Finalização de geração obsoleta** | sessão corrompida na troca de documento | `finish_production_if_current` já checa `p.generation == generation`; checagem inclui a mesma condição |
| **`complete` antecipado** | sessão fechada com áudio ainda na fila | nenhuma chamada de `complete` no produtor; gate `drained && !production_running` intacto |
| **Ramo de backpressure não coberto** | exaustão presa com `room == 0` | checagem no topo do loop, antes do `break` de :5803-5805 |
| **Dupla finalização no mesmo tick** | estado inconsistente | helper é idempotente por `take()`; checagem de `done` pós-commit preservada |
| **Enum novo por conveniência** | segunda origem de verdade, viola FR-027-004 | condição com campos existentes; nenhum tipo público novo |
| **Escopo de verificação inflado** | afirmação de `session_end` em device sem prova | `[NV]` explícito; reprodução limpa é dívida declarada; gates pertencem ao orquestrador |

> Afirmações sobre comportamento em dispositivo são `[NV]`; a validação deste
> ciclo é exclusivamente de host.

## Estratégia de verificação em host

- Confirmar que o único arquivo de código alterado é `src-tauri/src/lib.rs`.
- Confirmar a presença da checagem `exhausted` no topo do loop, do guard solto
  antes de `finish_production_if_current` e do `break` subsequente.
- Confirmar que o corpo incremental (handover, backpressure, `get()`, commit,
  `done` pós-commit) permanece intacto.
- Confirmar que `playback_watch_tick`, `production_running`, `advance_production`,
  `synth.rs`, `chunk.rs`, `Cargo.toml`, `scripts/expected-tests.txt`, HTML,
  capabilities e workflow não aparecem no diff.
- Confirmar que nenhum enum novo aparece e que nenhuma chamada de
  `signal_document_end`/`dispatch_document_end`/`ShellVerb::Complete` foi
  introduzida em `produce_bounded`.
- Confirmar que o teste de regressão vive em `ttfa_013_tests` e cobre os quatro
  comportamentos exigidos por FR-027-005.
- Registrar `[NV]` para dispositivo; nenhuma meta ou resultado de gate é
  reivindicado por esta cadeia documental.

## Complexity Tracking

> Nenhuma violação de constituição; esta tabela permanece vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |

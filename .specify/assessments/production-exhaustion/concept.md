# Concept: Exaustão da produção não finaliza o documento

- **Slug**: production-exhaustion
- **Created**: 2026-11-21
- **Recommended option**: Option A — detecção de exaustão no topo do loop

## Options

### Option A — Detecção de exaustão no topo do `while`
- **Sketch**: no início de cada iteração de `produce_bounded` (:5782), sob um
  lock curto, verificar se a geração é a atual e se
  `next_unit >= segments.len() && pending.is_empty()`. Se sim, soltar o guard,
  chamar `finish_production_if_current` (:6004-6018) e dar `break`. O ramo de
  commit (:5834-5841) mantém a checagem de `done` já existente (:5847-5860) —
  ela continua necessária para finalizar no mesmo tick em que a última unidade
  é produzida.
- **Appetite**: small (uma checagem + uma chamada a helper existente + testes).
- **Trade-offs**: cobre todos os caminhos de saída do loop (inclusive o `break`
  de `room == 0` em :5803-5805), mas acrescenta uma leitura curta do lock por
  iteração; como o loop é limitado por `max_units` e a leitura não sintetiza, o
  custo é desprezível.
- **Rabbit holes**: transformar a checagem em novo enum de estado, mover a
  lógica para o watch, ou acionar `complete` diretamente.

### Option B — Tratar apenas o `else` de `segments.get()`
- **Sketch**: no ramo que hoje faz `return Ok(())` quando `get(next_unit)` é
  `None` (:5817-5819), soltar o `MutexGuard` e chamar
  `finish_production_if_current` antes de retornar.
- **Appetite**: small (menor que A).
- **Trade-offs**: cobre o caso relatado, mas **não** cobre a exaustão que
  encontra `room == 0` e `break` em :5803-5805 antes de chegar ao `get()`, nem
  outros retornos antecipados do loop. Correção incompleta por construção.
- **Rabbit holes**: multiplicar chamadas ao helper em cada `return`; risco de
  chamar com guard retido e deadlock.

### Option C — Fazer `production_running` ignorar produção exaurida
- **Sketch**: mudar `production_running` (:6360-6365) para devolver `false`
  quando `next_unit >= len() && pending.is_empty()`, abrindo o gate :2669 sem
  tocar em `produce_bounded`.
- **Appetite**: small, porém semanticamente errado.
- **Trade-offs**: esconde o estado preso em vez de removê-lo; deixa de instalar
  a timeline medida por `finish_production_if_current` → `install_session_timeline`
  (:6473-6492), e qualquer outro consumidor de `production_running` passa a ver
  um estado inconsistente. Correção de sintoma.
- **Rabbit holes**: criar enum `ProductionState { Active, Exhausted, Finished }`
  (proibido pelo requisito "sem enum novo") e espalhar a checagem pelos callers.

## Recommendation

**Option A**. É a única que cobre todos os caminhos de exaustão do loop com uma
mudança local, reutilizando exatamente o helper generation-safe que já existe e
já instala a timeline medida. Mantém intactos o gate de drenagem
(`drained && !production_running`), o `complete` do backend, o primário de 026,
o handover por geração e o contrato de áudio. Option B é incompleto; Option C
trata o sintoma e perde a timeline medida. Nenhuma opção introduz enum novo.

## Out of Scope (for the recommended option)

- Emissão de `signal_document_end`, `dispatch_document_end` ou
  `ShellVerb::Complete` a partir de `produce_bounded` — conclusão continua
  exigindo `backend.is_drained()` no watch (:2669).
- Mudança em `production_running`, `handover_pending_for_generation`,
  `commit_produced_unit`, `queue_room`, primário de 026, `synth.rs`,
  `chunk.rs`, `Cargo.toml`, HTML, capabilities ou workflow.
- Novo target de teste, edição de `scripts/expected-tests.txt` ou novo enum.
- Reprodução em dispositivo nesta cadeia documental; `action=complete` e
  `session_end` observados em device são `[NV]`.
- Edição de specs/bugs de outros ciclos ou de `converge.md` alheio.

## Assumptions to Validate

- A exaustão detectada no topo do loop precede todos os `return`/`break` que
  poderiam deixar a produção presa; a checagem de `done` pós-commit permanece
  válida para o tick da última unidade.
- `finish_production_if_current` é generation-safe e pode ser chamado com o
  guard de `state.production` solto; lock envenenado não dispara finalização.
- `started == false` com exaustão é um estado legítimo (documento vazio ou
  falha anterior) e deve liberar o gate — a conclusão real ainda depende da
  drenagem do backend.
- O módulo `ttfa_013_tests` aceita um caso de regressão sem novo target e sem
  alterar a lista canônica de testes, conforme gate 2 de diff bidirecional.
- A causa em host é suficiente para especificar a correção; a confirmação em
  dispositivo (`[NV]`) depende de reprodução limpa com diag zerado.

---

description: "Task list for feature implementation — Ciclo 024, retomada de diretório na reabertura"
---

# Tasks: Retomada de Diretório na Reabertura

**Input**: Design documents from `/specs/024-directory-resume-offer/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), checklists/requirements.md

**Tests**: o teste é exigido por FR-024-005, mas é REESCRITA no target já pinado — nenhum novo target é criado e `scripts/expected-tests.txt` não é alterado.

**Organization**: tarefas agrupadas por user story para permitir implementação e verificação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: qual user story a tarefa atende (US1, US2, US3)
- IDs são sequenciais T001..T015 e pertencem exclusivamente a este ciclo

## Path Conventions

- Caminhos relativos à raiz do repositório `mdreader`
- Alvo de teste pinado: `src-tauri/tests/test_directory_resume.rs` (pin em `scripts/expected-tests.txt:113`)

---

## Phase 1: Setup & Foundational (Shared Infrastructure)

**Purpose**: habilitar o uso do mecanismo existente sem avisos de código morto e fixar a regra de não-criação de target.

- [ ] T001 [P] Remover o atributo `#[expect(dead_code, reason = "directory reading not yet wired")]` de `park_directory_resume` (src-tauri/src/lib.rs:2155-2163) e de `DirectoryResume` (src-tauri/src/lib.rs:3116). [FR-024-001]
- [ ] T002 [P] Confirmar que `src-tauri/tests/test_directory_resume.rs` já está pinado como `src-tauri::test_directory_resume` em `scripts/expected-tests.txt:113`; NÃO criar novo target e NÃO editar esse arquivo. [FR-024-005]

**Checkpoint**: fundação pronta — nenhuma user story depende de arquivo novo.

---

## Phase 2: User Story 1 — Oferta de retomada alcançável na reabertura (Priority: P1) 🎯 MVP

**Goal**: a reabertura de pasta com bookmark `Paused` publica a rota do diálogo e estaciona a oferta até a resposta do usuário.

**Independent Test**: em host, com bookmark `Paused`, o caminho de reabertura publica `in_app_dialog`, mantém a janela de 3.000 ms e resolve a máquina uma única vez.

### Implementation for User Story 1

- [ ] T003 [US1] Estender `DirectoryResume` (src-tauri/src/lib.rs:3116) com `filename: String` e `offset_secs: f64`, ambos derivados do `CompositeBookmark` ativo, preservando a rota `InAppDialog` já calculada por `install_chapter_queue` (src-tauri/src/lib.rs:3133-3173, mapeamento em :3160-3168). [FR-024-001, FR-024-002]
- [ ] T004 [US1] Em `handle_directory_selected` (src-tauri/src/shell_jni.rs:1431-1451), após `install_chapter_queue`, construir `StoredPosition` a partir do bookmark e chamar `route_offer` (src-tauri/src/lib.rs:2082) EXATAMENTE UMA VEZ — sem re-derivar a rota no loop de poll. [FR-024-003]
- [ ] T005 [US1] Chamar `park_directory_resume` (src-tauri/src/lib.rs:2155-2163) com o `DirectoryResume` enriquecido no mesmo caminho de reabertura, estacionando a oferta até a resposta. [FR-024-001]
- [ ] T006 [US1] Publicar a rota via `publish_route` / `offer_route_wire` (src-tauri/src/lib.rs:2104, :2137) e enriquecer o evento `directory-selected` com `filename`/`offset_secs`, de modo que `offer_route` (src-tauri/src/lib.rs:7138) deixe de responder `none/pending_cold_start:false` e `applyOffer` (src-tauri/src/status_view.html:1093-1120) pare de fechar/no-op. [FR-024-001, FR-024-002]
- [ ] T007 [US1] Garantir que a transição `expiry → AutoStart` continue sendo resolvida pela máquina mutata `route_offer`, preservando `published_route` (src-tauri/src/lib.rs:188); zero re-derivações de rota em poll. [FR-024-003]

**Checkpoint**: US1 deve estar funcional e verificável em host de forma independente.

---

## Phase 3: User Story 2 — Contexto da retomada visível na interface (Priority: P1)

**Goal**: a oferta exibe nome de arquivo não vazio e offset salvo igual ao bookmark, com os cinco bundles HTML byte-idênticos.

**Independent Test**: em host, o tratador de `directory-selected` atribui os campos ao estado antes de atualizar a oferta, e as cinco superfícies HTML têm o mesmo hash.

### Implementation for User Story 2

- [ ] T008 [US2] No tratador do evento `directory-selected` (src-tauri/src/status_view.html:1405-1408), ler `filename`/`offset_secs` do payload e atribuir `state.chapterFilename` (lido em :1068 mas nunca escrito) e `state.saved_offset_secs` ANTES de chamar `refreshOffer()` (:1145). Nenhuma alteração de marcação HTML. [FR-024-004]
- [ ] T009 [P] [US2] Espelhar byte-a-byte `src-tauri/src/status_view.html` em `dist/index.html`. [FR-024-006]
- [ ] T010 [P] [US2] Espelhar byte-a-byte `src-tauri/src/status_view.html` em `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html` e `src-tauri/gen/android/app/src/main/assets/index.html`. [FR-024-006]
- [ ] T011 [US2] Confirmar que `renderOffer` (src-tauri/src/status_view.html:1067-1072) e os botões `resume_at_stored` / `start_over` (:1336-1343) consomem os novos valores sem exigir nova marcação, preservando a janela de 3.000 ms e o auto-continue em foreground como regras já existentes. [FR-024-004]

**Checkpoint**: US1 e US2 funcionam independentemente; os cinco HTML são idênticos byte a byte.

---

## Phase 4: User Story 3 — Evidência comportamental no target já pinado (Priority: P2)

**Goal**: o teste mede o comportamento observável em vez da presença literal de strings no fonte.

**Independent Test**: em host, o teste falha se a rota não for publicada ou se o payload não trouxer nome e offset; nenhum pin é alterado.

### Tests for User Story 3 (exigidos por FR-024-005)

- [ ] T012 [US3] Substituir as asserções literais `BRIDGE.contains(...)` / `STATUS_VIEW.contains(...)` em `reopen_paused_directory_routes_to_in_app_offer_dialog_for_3s` (src-tauri/tests/test_directory_resume.rs:95-117) por asserções comportamentais sobre: rota publicada `in_app_dialog`, `filename` não vazio, `offset_secs = 45.0` do bookmark, janela de 3.000 ms e payload entregue ao diálogo. [FR-024-005]
- [ ] T013 [US3] Acrescentar ao MESMO target uma asserção de resolução única da máquina de oferta (contagem = 1 por reabertura; zero re-derivações em poll), sem criar novo arquivo de teste nem novo `#[test]` em outro target. [FR-024-003, FR-024-005]

**Checkpoint**: a evidência comportamental cobre as três User Stories e o pin permanece intacto.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: verificação final de integridade e limites do ciclo.

- [ ] T014 [P] Verificar em host a byte-identity dos cinco HTML (`src-tauri/src/status_view.html`, `dist/index.html`, `src-tauri/src/assets/index.html`, `src-tauri/dist/index.html`, `src-tauri/gen/android/app/src/main/assets/index.html`) e a imutabilidade de `scripts/expected-tests.txt`. [FR-024-005, FR-024-006]
- [ ] T015 Revisar que todos os IDs deste arquivo são T001..T015, que nenhum ID da spec 010 foi reintroduzido e que nenhum arquivo fora do escopo do ciclo foi tocado. [Polish]

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup/Foundational)**: sem dependências — pode começar imediatamente.
- **Phase 2 (US1)**: depende de T001 (remoção do marcador de código morto).
- **Phase 3 (US2)**: depende de T006 (evento enriquecido); os espelhos T009/T010 podem rodar em paralelo após T008.
- **Phase 4 (US3)**: depende de T006 e T008; pode ser escrita primeiro como sinal vermelho (TDD), mas só passa após Fase 2/3.
- **Phase 5 (Polish)**: depende de todas as fases anteriores.

### User Story Dependencies

- **US1 (P1)**: depende apenas da Phase 1 — sem dependência de outras stories.
- **US2 (P1)**: consome o payload produzido por US1, mas é testável isoladamente injetando o payload no evento.
- **US3 (P2)**: verifica US1 e US2; não introduz comportamento de produto.

### Parallel Opportunities

- T001 e T002 podem rodar em paralelo.
- T009 e T010 podem rodar em paralelo entre si e após T008.
- T014 pode rodar em paralelo com a revisão final de T013.

---

## Parallel Example: User Story 2

```bash
# Após T008, espelhar o fonte nos quatro bundles em paralelo:
Task: "Espelhar src-tauri/src/status_view.html em dist/index.html (T009)"
Task: "Espelhar src-tauri/src/status_view.html em src-tauri/src/assets/index.html, src-tauri/dist/index.html e src-tauri/gen/android/app/src/main/assets/index.html (T010)"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Concluir Phase 1 (T001, T002).
2. Concluir Phase 2 (T003–T007).
3. **PARAR E VALIDAR**: rota publicada, janela de 3.000 ms e uma resolução de oferta em host.
4. Só então seguir para US2.

### Incremental Delivery

1. Setup + Foundational → fundação pronta.
2. US1 → oferta alcançável (MVP).
3. US2 → contexto visível e bundles idênticos.
4. US3 → evidência comportamental no target pinado.
5. Polish → verificação de limites e integridade.

### Never Do

- Criar novo target de teste ou editar `scripts/expected-tests.txt`.
- Editar `specs/010` ou renumerar tarefas existentes daquela spec.
- Introduzir segunda origem de posição na interface.
- Re-derivar a rota de oferta em poll.
- Reivindicar verificação em dispositivo: toda a evidência deste ciclo é de host (`[NV]` para device).

---

## Notes

- **[P]** = arquivos diferentes, sem dependências entre si.
- **[Story]** rastreia a tarefa até a user story correspondente em `spec.md`.
- Cada tarefa é específica o bastante para ser executada sem contexto adicional; os números de linha são os do mapa de evidência verificado em host e podem derivar após edições — use o símbolo como âncora.
- Gates, testes e formatadores NÃO são executados por subagentes; a verificação final cabe ao agente principal.

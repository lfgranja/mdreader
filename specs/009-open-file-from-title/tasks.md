# Tasks: Abrir Arquivo Pelo Título e Iniciar Reprodução

**Input**: Design documents from `/specs/009-open-file-from-title/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/ipc-and-jni.md`, `quickstart.md`
**Tests**: TDD is mandatory per project rules (red-green-refactor, property tests for invariants, tests written first and failing before implementation).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (disjoint files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`)
- File paths are exact and absolute/repo-relative.
- Subagent Rule: Use shared tooling only on one agent at a time. Do not interrupt running tools.
- Commit Hook: Atomic conventional commit upon editing each file.

---

## Phase 1: Setup & Foundational (Infrastructure & Types)

**Purpose**: Scaffolding actions, JNI signatures, and shared state tracking.

- [x] T158 [P] Declare `PickDocument` action variant in `BridgeAction` enum and register `"pick_document"` in `parse_action` in `src-tauri/src/lib.rs`.
- [x] T159 [P] Add `currentActivity` lifecycle tracker in `android-shell/src/main/kotlin/app/mdreader/ShellApplication.kt` and expose `ShellHost.currentActivity` accessor.
- [x] T160 Define `pick_document_from_shell` JNI dispatch calling `ShellHost.pickDocument()` in `src-tauri/src/shell_jni.rs` (with no-op fallback on non-Android).
- [x] T161 Add `DocIngest.pickDocument` trigger method in `android-shell/src/main/kotlin/app/mdreader/ShellHost.kt` delegating to `DocIngest.pickDocument(activity)`.

---

## Phase 2: User Story 1 - Selecionar Novo Documento e Iniciar Reprodução Imediata (Priority: P1) 🎯 MVP

**Goal**: Permitir ao usuário tocar na caixa de título (`.meta-box`) para abrir o seletor SAF (`ACTION_OPEN_DOCUMENT`), selecionar um arquivo Markdown/texto compatível e iniciar a síntese/reprodução imediatamente.

**Independent Test**: Com o app aberto, tocar na `.meta-box`, selecionar um `.md` inédito no SAF e verificar se o título atualiza e o áudio começa a tocar automaticamente (< 1s).

### Tests for User Story 1 (TDD - Write First, MUST FAIL initially) ⚠️

- [x] T162 [P] [US1] Integration/Contract test in `src-tauri/tests/test_pick_document_action.rs` verifying that dispatching `"pick_document"` invokes the platform picker trigger, leaves position unchanged, and handles valid UTF-8 payload ingest with automatic playback transition.
- [x] T163 [P] [US1] Property-based test in `crates/session/tests/prop_pick_document_open.rs` testing invariants: an unread document with valid bytes (`text/markdown`, size ≤ 93636) always transitions to `StartNow` (auto-play) with position starting at 0.0 and correct Blake3 fingerprint.

### Implementation for User Story 1

- [x] T164 [US1] Wire `BridgeAction::PickDocument` in `dispatch_action` / `run_mobile_plugin` in `src-tauri/src/lib.rs` to call `shell_jni::pick_document_from_shell()`.
- [x] T165 [US1] Forward `onActivityResult` in `src-tauri/gen/android/app/src/main/java/app/mdreader/MainActivity.kt` to `DocIngest.handlePickResult(this, requestCode, resultCode, data)`.
- [x] T166 [P] [US1] Update `.meta-box` in `src-tauri/src/status_view.html` with `role="button"`, `tabindex="0"`, `aria-label="Selecionar documento para leitura"`, minimum touch dimensions (`min-height: 48px`), CSS active scale (`0.985`), and `:focus-visible` styling (`var(--sona-etwahl)`).
- [x] T167 [US1] Add click, `Enter`, and `Space` event listeners on `.meta-box` with debounce in `src-tauri/src/status_view.html` invoking `sendAction('pick_document')` and restoring focus upon picker completion.

**Checkpoint**: At this point, User Story 1 delivers an end-to-end working MVP for opening new documents from the title box.

---

## Phase 3: User Story 2 - Respeito à Política de Retomada em Documentos Já Lidos (Priority: P2)

**Goal**: Garantir que reabrir um documento com histórico prévio de pausa deliberada preserve o progresso e apresente o diálogo de oferta (`offerDialog` / FR-009) por 3 segundos, enquanto documentos acidentais/kill retomam diretamente do heartbeat.

**Independent Test**: Reabrir documento pausado na metade e verificar que o `<dialog id="offerDialog">` abre sem tocar áudio antes da decisão.

### Tests for User Story 2 (TDD - Write First, MUST FAIL initially) ⚠️

- [ ] T168 [P] [US2] Contract/Integration test in `src-tauri/tests/test_pick_document_resume_offer.rs` asserting that picking a document with a saved deliberate-pause bookmark in `redb` routes through `AwaitOffer`, sets `offer_route: in_app_dialog`, `pending_cold_start: true`, and holds audio until resolved.
- [ ] T169 [P] [US2] Property test in `crates/session/tests/prop_resume_offer_invariants.rs` asserting that identical byte content with differing filenames retains the stored bookmark and resumes from the exact recorded offset.

### Implementation for User Story 2

- [ ] T170 [US2] Ensure `open_ingested_document` in `src-tauri/src/shell_jni.rs` preserves `open_document_bytes_named` return payload facts (`offer_route`, `pending_cold_start`, `offer_window_ms`) and notifies the webview publish channel.
- [ ] T171 [US2] Verify and refine `applyOffer` integration in `src-tauri/src/status_view.html` to trigger `openOffer(windowMs)` immediately when an ingested document returns an `in_app_dialog` route.

**Checkpoint**: User Stories 1 and 2 work seamlessly without bookmark corruption.

---

## Phase 4: User Story 3 - Cancelamento ou Recusa sem Efeito Colateral na Reprodução Atual (Priority: P3)

**Goal**: Assegurar que cancelamento no SAF (botão Voltar), arquivo vazio (0 bytes), arquivo excedendo 93.636 bytes (`FormatUndeliverable`) ou não-UTF8 não afetem a reprodução corrente e apresentem feedback discreto.

**Independent Test**: Com áudio tocando, abrir o seletor e cancelar: o áudio não deve engasgar. Selecionar arquivo > 93.636 bytes: a tela exibe o banner de aviso sem interromper a sessão.

### Tests for User Story 3 (TDD - Write First, MUST FAIL initially) ⚠️

- [ ] T172 [P] [US3] Unit & Invariant test in `src-tauri/tests/test_pick_document_cancellation.rs` asserting that `IngestOutcome::Cancelled` or `IngestOutcome::Refused` makes zero modifications to `AppState::session` or `PositionClock`.
- [ ] T173 [P] [US3] Contract test in `src-tauri/tests/test_pick_document_error_notification.rs` verifying that format refusals (>93.636 bytes / non-text) publish `reason: "FormatUndeliverable"` to `reasonBanner` and `liveRegion` without audio interruption.

### Implementation for User Story 3

- [ ] T174 [US3] Implement refusal callback / notification bridge from `DocIngest.handlePickResult` in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` to notify the UI of `IngestOutcome.Refused(FormatUndeliverable)` and `IngestOutcome.Unreadable`.
- [ ] T175 [US3] Handle `FormatUndeliverable` notification in `src-tauri/src/status_view.html` by invoking `renderReason("FormatUndeliverable")` to display the inline reason banner and announce via `liveRegion`.
- [ ] T176 [US3] Wrap SAF activity launch in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` in a try-catch block for `ActivityNotFoundException` and unexpected system exceptions, logging diagnostics safely.

---

## Phase 5: Verification & Quality Gates (Final Phase)

**Purpose**: Formal execution of the project verification gate. Only these tests attest to success.

- [ ] T177 Run formatting check: `cargo fmt --all -- --check`.
- [ ] T178 Run all unit, integration, and property tests via nextest: `cargo nextest run --workspace`.
- [ ] T179 Update and verify gate 2 test census against `scripts/expected-tests.txt`.
- [ ] T180 Run clippy lint gate: `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] T181 Run closed-set purity and shell-purity scans: `./scripts/check-shell-purity.sh` and gate 8 checks.
- [ ] T182 Execute full gate verification suite: `./scripts/verify.sh`.
- [ ] T183 Perform on-device validation against test scenarios documented in `specs/009-open-file-from-title/quickstart.md` using ADB.

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 (Setup & Foundational)**: Must be completed first to establish JNI bindings and action routing.
- **Phase 2 (User Story 1 - P1)**: Depends on Phase 1. Tests (T162, T163) MUST be written first and fail (RED). Implementation tasks (T164-T167) follow (GREEN).
- **Phase 3 (User Story 2 - P2)**: Depends on Phase 2. Tests (T168, T169) written first and failing. Implementation tasks (T170, T171) follow.
- **Phase 4 (User Story 3 - P3)**: Depends on Phase 2. Tests (T172, T173) written first and failing. Implementation tasks (T174-T176) follow.
- **Phase 5 (Verification)**: Depends on completion of all desired user stories.

### Parallel Execution Waves
- **Wave 1 (Setup)**: `T158` (lib.rs) and `T159` (ShellApplication.kt) can run in parallel (disjoint files).
- **Wave 2 (US1 TDD)**: `T162` (test_pick_document_action.rs) and `T163` (prop_pick_document_open.rs) run in parallel.
- **Wave 3 (US1 Implementation)**: `T165` (MainActivity.kt) and `T166` (status_view.html CSS/HTML) can run in parallel.
- **Wave 4 (US2 TDD)**: `T168` and `T169` run in parallel.
- **Wave 5 (US3 TDD)**: `T172` and `T173` run in parallel.
- **Wave 6 (Verification)**: Sequential execution of gates 1-8 via `verify.sh` and device test.

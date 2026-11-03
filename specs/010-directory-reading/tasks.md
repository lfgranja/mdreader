# Tasks: Leitura Contínua de Diretórios de Markdown (010-directory-reading)

**Input**: Design documents from `specs/010-directory-reading/` (`spec.md`, `plan.md`, `data-model.md`, `contracts/directory-playback.md`, `quickstart.md`, `research.md`).
**Prerequisites**: Constitution v2.0.0 (`.specify/memory/constitution.md`), Principle I (Rust sole playback authority), Principle II (one code path), Principle VII (closed sets everywhere), Principle VIII (driving-time safety), Principle IX (pure offline).
**Numbering**: Continuing sequentially from T185.
**Format**: Strict `- [ ] T### [P?] [Story?] Description with exact repo-relative file path`.
**TDD Mandate**: All test tasks (contract, property, integration) are written first, testing invariants, and BLOCKING implementation tasks.

---

## Phase 1: Setup & Infrastructure (Scaffolding Actions, JNI Signatures & Types)

**Purpose**: Scaffolding action variants, JNI signatures, and shared state tracking for directory picking and chapter flow.

- [x] T185 [P] Declare closed `ChapterStatus` enum (`Pending`, `Playing`, `Completed`, `Skipped`) and `ChapterItem` struct with serialization in `crates/player/src/queue.rs`.
- [x] T186 [P] Declare directory and chapter action variants (`PickDirectory`, `NextChapter`, `PreviousChapter`) in `BridgeAction` enum and register `"pick_directory"`, `"next_chapter"`, `"previous_chapter"` in `parse_action` in `src-tauri/src/lib.rs`.
- [x] T187 Declare `pick_directory_from_shell` JNI dispatch calling `ShellHost.pickDirectory()` with non-Android no-op in `src-tauri/src/shell_jni.rs`.
- [x] T188 [P] Add `DocIngest.pickDirectory(activity)` implementation using `ACTION_OPEN_DOCUMENT_TREE` with persistable read flags and single-active directory permission release in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`.
- [x] T189 [P] Expose static `ShellHost.pickDirectory()` dispatch delegating to `DocIngest.pickDirectory(currentActivity)` in `android-shell/src/main/kotlin/app/mdreader/ShellHost.kt`.

---

## Phase 2: Foundational (Core Data Structures & Storage)

**Purpose**: Core data structures, composite Blake3 hashing, and dedicated storage table for directory bookmarks.

- [ ] T190 [P] Implement `compute_collection_fingerprint(hashes: &[[u8; 32]]) -> [u8; 32]` computing `blake3(fp_1 || fp_2 || ... || fp_n)` in `crates/session/src/fingerprint.rs`.
- [ ] T191 [P] Declare `DIRECTORY_BOOKMARKS: TableDefinition<&[u8; 32], (u32, f64, u64, u8)>` table and define `CompositeBookmark` struct in `crates/session/src/store.rs`.
- [ ] T192 Implement `ChapterQueue` with index tracking, natural sort ordering, and transition state machine in `crates/player/src/queue.rs`.
- [ ] T193 Implement directory bookmark CRUD methods (`get_directory_bookmark`, `put_directory_bookmark`, `delete_directory_bookmark`) in `crates/session/src/store.rs`.
- [ ] T194 Define JNI entrypoint `Java_app_mdreader_DocIngest_onDirectorySelected` deserializing the directory manifest JSON and instantiating `ChapterQueue` in `src-tauri/src/shell_jni.rs`.

---

## Phase 3: User Story 1 - Selecionar Diretório e Iniciar Reprodução Contínua (Priority: P1) 🎯 MVP

**Goal**: Permitir ao usuário selecionar uma pasta com múltiplos arquivos `.md`/`.txt` via SAF, iniciar a reprodução audível do primeiro capítulo em < 3,0 segundos (TTFA), e transicionar automaticamente de forma gapless (< 1,0 s) para o próximo arquivo ordenado.

**Independent Test**: Selecionar pasta local com notas numeradas (`01-intro.md`, `02-arquitetura.md`). O sistema inicia o áudio da primeira nota em < 3.0s e, ao término, transiciona automaticamente para a segunda sem intervenção manual.

### Tests for User Story 1 (TDD - Write First, MUST FAIL initially) ⚠️

- [ ] T195 [P] [US1] Contract test in `src-tauri/tests/test_directory_playback_contract.rs` asserting that `pick_directory` action initiates directory scanning, emits chapter metadata, and starts audio within TTFA budget (< 3.0s).
- [ ] T196 [P] [US1] Property-based test in `crates/player/tests/prop_chapter_queue_natural_sort.rs` testing invariants of natural sort ordering across arbitrary filename patterns (`01-`, `02-`, `10-`, case-insensitive, README index precedence).
- [ ] T197 [P] [US1] Integration test in `crates/player/tests/test_gapless_chapter_transition.rs` asserting automatic gapless transition from chapter $N$ to $N+1$ with silence gap < 1.0s and PositionClock reset to 0.0s for the new chapter.

### Implementation for User Story 1

- [ ] T198 [US1] Implement background coroutine directory scanning (`Dispatchers.IO`) in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` collecting `.md`/`.txt` files from root and 1st-level subfolders with natural sort.
- [ ] T199 [US1] Dispatch immediate first chapter yield to Rust via JNI before full background scan completion in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt`.
- [ ] T200 [US1] Integrate `ChapterQueue` into `AppState` and handle `BridgeAction::Verb(ShellVerb::PickDirectory)` in `src-tauri/src/lib.rs`.
- [ ] T201 [US1] Implement automatic gapless chapter advance on `DocumentEnd` in `crates/player/src/lib.rs` without destroying or recreating the `cpal` audio stream.
- [ ] T202 [P] [US1] Update `status_view.html` `#sectionTitle` and `h1` to render active chapter title and tabular chapter counter (`"Capítulo X de Y • [Seção]"`) preserving 100dvh ceiling with zero CLS in `src-tauri/src/status_view.html`.
- [ ] T203 [US1] Announce chapter transitions politely to screen readers via `#liveRegion` (`aria-live="polite"`, `aria-atomic="true"`) in `src-tauri/src/status_view.html`.

**Checkpoint**: User Story 1 delivers an end-to-end working MVP for opening directories and playing sequentially.

---

## Phase 4: User Story 2 - Retomada Precisa por Arquivo e Posição Salva (Priority: P2)

**Goal**: Garantir que pausar ou fechar o app durante a leitura de uma pasta preserve o índice exato do arquivo e o offset em segundos, oferecendo diálogo de retomada de 3,0 segundos (`offerDialog`) na reabertura, e mantendo heartbeat a cada 20 segundos.

**Independent Test**: Tocar pasta com 5 notas, pausar no arquivo 3 aos 45s e fechar o app. Ao reabrir a mesma pasta, o sistema apresenta oferta de retomada indicando o arquivo 3 aos 45s e, ao confirmar, retoma exatamente desse ponto.

### Tests for User Story 2 (TDD - Write First, MUST FAIL initially) ⚠️

- [ ] T204 [P] [US2] Property-based test in `crates/session/tests/prop_directory_resume.rs` testing invariants: deliberate pause stores `(chapter_index, offset_secs, ts, Paused)` and routes to `AwaitOffer` (`in_app_dialog`); interrupted/kill stores `Interrupted` and resumes immediately without prompt.
- [ ] T205 [P] [US2] Integration test in `src-tauri/tests/test_directory_resume.rs` verifying that reopening a previously paused directory presents `offerDialog` for 3.0s, and confirming "Continuar" restores the active chapter and position clock.

### Implementation for User Story 2

- [ ] T206 [US2] Implement atomic write transaction for chapter completion and new chapter start persistence in `crates/session/src/store.rs`.
- [ ] T207 [US2] Add 20-second heartbeat write loop for active directory playback with `SessionEnd::Interrupted` in `src-tauri/src/lib.rs`.
- [ ] T208 [US2] Implement `lookup_directory_on_open` in `crates/session/src/store.rs` resolving composite Blake3 fingerprint against `DIRECTORY_BOOKMARKS`.
- [ ] T209 [US2] Connect directory resume offer flow to `offerDialog` in `src-tauri/src/status_view.html` displaying current chapter filename and saved position with 3.0s auto-dismiss window.
- [ ] T210 [US2] Implement "Recomeçar do início" handler resetting active chapter to 0, offset to 0.0s, and clearing previous bookmark in `src-tauri/src/lib.rs`.

**Checkpoint**: User Stories 1 and 2 operate seamlessly with crash-safe durability.

---

## Phase 5: User Story 3 - Navegação Entre Capítulos/Notas e Resiliência contra Arquivos Inválidos (Priority: P3)

**Goal**: Permitir avançar e retroceder capítulos contextualmente via fones Bluetooth, lock screen (`ACTION_SKIP_TO_NEXT`/`PREVIOUS`), e tratar arquivos corrompidos, vazios (0 bytes) ou maiores que 93.636 bytes pulando-os silenciosamente com telemetria diagnóstica.

**Independent Test**: Inserir arquivo vazio e arquivo > 93KB entre notas válidas. Durante o playback, o sistema pula os arquivos incompatíveis com aviso discreto e continua reproduzindo os arquivos válidos. Skip forward na última seção avança de capítulo.

### Tests for User Story 3 (TDD - Write First, MUST FAIL initially) ⚠️

- [ ] T211 [P] [US3] Contract test in `src-tauri/tests/test_directory_contextual_skip.rs` asserting that `ACTION_SKIP_TO_NEXT` on the last section jumps to chapter $N+1$, and `ACTION_SKIP_TO_PREVIOUS` within the first 3 seconds jumps to chapter $N-1$.
- [ ] T212 [P] [US3] Property-based test in `crates/player/tests/prop_invalid_chapter_skipping.rs` asserting that files > 93.636 bytes, 0 bytes, or non-UTF8 are marked `ChapterStatus::Skipped`, emit `DiagnosticKind::Failure(FormatUndeliverable)`, and do not crash or block subsequent valid chapters.
- [ ] T213 [P] [US3] Unit test in `crates/player/tests/test_pre_roll_cancellation.rs` asserting that pre-synthesized buffer for chapter $N+1$ (10s pre-roll trigger) is cooperatively cancelled and purged upon manual seek or skip.

### Implementation for User Story 3

- [ ] T214 [US3] Implement contextual skip logic in `crates/player/src/skip.rs` combining heading boundary check with `ChapterQueue::next()` and `ChapterQueue::previous()`.
- [ ] T215 [US3] Forward `ACTION_SKIP_TO_NEXT` and `ACTION_SKIP_TO_PREVIOUS` from `MediaMirror.kt` through shell JNI to contextual skip in `src-tauri/src/lib.rs`.
- [ ] T216 [US3] Implement individual chapter byte budget validation (`MAX_DOC_BYTES = 93.636`) and UTF-8 verification with diagnostic failure logging in `crates/player/src/queue.rs`.
- [ ] T217 [US3] Implement adaptive 10.0-second pre-roll synthesis trigger for chapter $N+1$ with cooperative cancellation token on manual skip/seek in `src-tauri/src/lib.rs`.
- [ ] T218 [US3] Implement graceful refusal banner rendering and TalkBack notification for skipped corrupted files without stopping transport in `src-tauri/src/status_view.html`.

**Checkpoint**: Full feature capability complete across all user scenarios and edge cases.

---

## Phase 6: Polish, Verification & Quality Gates (Final Phase)

**Purpose**: Formal execution of the project verification gate (Gates 1-8). Only these deterministic tests attest to success.

- [ ] T219 Run formatting check: `cargo fmt --all -- --check`.
- [ ] T220 Run all unit, integration, and property tests via nextest: `cargo nextest run --workspace`.
- [ ] T221 Update and verify gate 2 test census against `scripts/expected-tests.txt`.
- [ ] T222 Run clippy lint gate: `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] T223 Run closed-set purity and shell-purity scans: `./scripts/check-shell-purity.sh` and gate 7 checks.
- [ ] T224 Run supply chain security audit: `cargo deny check && cargo audit`.
- [ ] T225 Execute full gate verification suite: `./scripts/verify.sh`.
- [ ] T226 Perform manual and ADB verification of Scenarios 1–5 on Android target device per `specs/010-directory-reading/quickstart.md`.

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 (Setup & Infrastructure)**: Must be completed first to establish data types and JNI channels.
- **Phase 2 (Foundational)**: Depends on Phase 1. Establishes `ChapterQueue`, `DIRECTORY_BOOKMARKS`, and Blake3 composite hashing.
- **Phase 3 (User Story 1 - P1)**: Depends on Phase 2. Tests (T195, T196, T197) MUST be written first and FAIL before implementation tasks (T198-T203).
- **Phase 4 (User Story 2 - P2)**: Depends on Phase 3. Tests (T204, T205) MUST be written first and FAIL before implementation tasks (T206-T210).
- **Phase 5 (User Story 3 - P3)**: Depends on Phase 3 and Phase 4. Tests (T211, T212, T213) MUST be written first and FAIL before implementation tasks (T214-T218).
- **Phase 6 (Polish & Verification)**: Depends on completion of all implementation phases. Enforces strict zero-failure verification gate (Gates 1-8).

### Parallel Execution Waves
- **Wave 1 (Setup Scaffolding)**: `T185` (`queue.rs`), `T186` (`lib.rs`), `T188` (`DocIngest.kt`), and `T189` (`ShellHost.kt`) can run in parallel (disjoint files).
- **Wave 2 (Foundational Core)**: `T190` (`fingerprint.rs`) and `T191` (`store.rs`) can run in parallel.
- **Wave 3 (US1 TDD Tests)**: `T195` (`test_directory_playback_contract.rs`), `T196` (`prop_chapter_queue_natural_sort.rs`), and `T197` (`test_gapless_chapter_transition.rs`) run in parallel.
- **Wave 4 (US1 UI & Scanning)**: `T198` (`DocIngest.kt`) and `T202` (`status_view.html`) can run in parallel.
- **Wave 5 (US2 TDD Tests)**: `T204` (`prop_directory_resume.rs`) and `T205` (`test_directory_resume.rs`) run in parallel.
- **Wave 6 (US3 TDD Tests)**: `T211` (`test_directory_contextual_skip.rs`), `T212` (`prop_invalid_chapter_skipping.rs`), and `T213` (`test_pre_roll_cancellation.rs`) run in parallel.
- **Wave 7 (Verification Gates)**: Sequential execution of gates 1-8 via `verify.sh` and device validation.

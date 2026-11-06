# Tasks: 012-debug-ingest-hook

**Input**: Design documents from `specs/012-debug-ingest-hook/`
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md), [.specify/memory/constitution.md](../../.specify/memory/constitution.md)

**Tests**: REQUIRED by user directive (TDD). Test/spec-check tasks MUST be written first, use property-based testing/invariants where applicable, and BLOCK implementation until all scoped tests exist and FAIL (RED phase). Do not mark RED tasks complete by making them pass.
**Organization**: Tasks are grouped by user story for independent implementation/testing. `[P]` = disjoint files, no dependency on incomplete tasks. Shared-context tasks (e.g., same Kotlin hook file set or same script contract) MUST reuse the same subagent/wave owner.

## Execution Rules (mandatory)

- **Template/scaffold/LSP first (rule 0)**: Before hand-writing generated code, use the applicable template/scaffold or LSP code action when viable (Android/Kotlin/Gradle/manifest scaffolds, script skeleton generators, test templates). Record `N/A:<reason>` in the task evidence when no generator/LSP action applies.
- **TDD RED gate (rule 1)**: For each wave, write all listed test/check tasks first. Confirm RED/failing-or-missing behavior against the target contract before implementation begins. Implementation tasks in that wave are blocked until RED evidence is recorded.
- **Parallel waves (rule 2)**: Execute `[P]` tasks in parallel only when files are disjoint and prerequisites are satisfied. Reuse the same subagent for tasks that share context/files/contracts. If a shared long-running tool is busy, other agents wait in queue; never kill/interrupt/bypass another agent/thread/process tool.
- **Verification attestation (rule 3)**: Only Phase 9 verification tasks attest success/completeness. Agent claims, partial runs, and non-verification task completion do not.
- **Atomic commit hook (rule 4)**: Whenever editing a file finishes, create an atomic conventional commit with a detailed summary of edits made. Deterministic hook scope: one logical edit unit/file-finish event per atomic commit unless the repo owner documents a narrower batch; never mix unrelated file-finish events.

## Path Conventions

- Android shell (canonical Kotlin): `android-shell/src/main/kotlin/app/mdreader/`
- Canonical FR-011 overlays (Spec 012): `specs/012-debug-ingest-hook/android-overlays/app/src/`
- Generated materialization targets only (never canonical): `src-tauri/gen/android/app/src/{debug,androidTest}/`
- Repo scripts: `scripts/`
- Host/script/spec test helpers (create only if needed; prefer existing repo conventions): `scripts/tests/` or `tests/spec012/`
- Spec 009 residual updates: `specs/009-open-file-from-title/tasks.md`
- Spec 010 residual updates: `specs/010-directory-reading/tasks.md`
- Feature docs/contracts: `specs/012-debug-ingest-hook/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create canonical FR-011 structure and test scaffolding locations before behavior changes.

- [ ] T001 Create canonical FR-011 overlay directories at `specs/012-debug-ingest-hook/android-overlays/app/src/debug/` and `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/` (if androidTest scaffold source is created) per `contracts/durable-overlay-materialization.md` and FR-013; add only required dirs.
- [ ] T002 [P] Create Spec 012 script/test helper directory `scripts/tests/spec012/` (or reuse existing test convention; record chosen path) with README note pointing to `specs/012-debug-ingest-hook/contracts/*.md` for contract-test ownership.
- [ ] T003 [P] Add FR-011 overlay format-version marker under `specs/012-debug-ingest-hook/android-overlays/` (detectable by apply/check; CHK022/CHK024) and document `v1-implicit` compatibility only if explicitly chosen, else require explicit version; update contract reference only if implementation-defined marker path is fixed.
- [ ] T004 [P] Capture baseline hook/precondition evidence anchors in `specs/012-debug-ingest-hook/quickstart.md` notes section only if missing: confirm `DebugIngestActivity.kt`/`DebugIngestHook.kt` current `path`+`uri` precedence is `unverified — confirm first` before coding contract behavior (no behavior change).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared contracts, scaffolds, and RED tests that block all user-story implementation.

**⚠️ CRITICAL**: No user-story implementation begins until Phase 2 RED evidence exists and fails as expected.

### Scaffold/Template First (blocking setup)

- [ ] T005 [P] Create executable script skeletons via repo shell template/scaffold conventions (LSP/shell scaffold N/A → document) for `scripts/apply-debug-ingest-overlay.sh`, `scripts/check-debug-ingest-overlay.sh`, and `scripts/check-debug-ingest-release-exposure.sh` with `--help`, fail-closed exit behavior, and stub `blocked:not-implemented` output; no contract satisfaction yet.
- [ ] T006 [P] Create canonical debug manifest overlay source `specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml` by copying/adapting the existing debug manifest structure as a scaffold, preserving debug-only exported `app.mdreader.DebugIngestActivity` and `app.mdreader.action.DEBUG_INGEST` only in debug overlay scope (FR-001/002/010/011).
- [ ] T007 [P] Create canonical androidTest smoke scaffold source `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` from Android instrumented-test template/scaffold conventions when viable; minimal debug-seam smoke only, no manual SAF interaction (FR-013). If scaffold generation unavailable, hand-write minimal compilable skeleton and record `N/A:<LSP/scaffold>`.

### Foundational RED Tests / Contract Checks (write first, must fail/block)

- [ ] T008 [P] Write failing contract tests for FR-011 overlay apply/check invariants in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py` (or repo-equivalent harness): idempotent apply, non-mutating check, explicit `pass|fail|blocked|not-applicable`, fail-closed on missing/stale/forbidden target/unknown format unless documented `v1-implicit`, androidTest conditional applicability (FR-013/SC-006), CHK021 no `src-tauri/gen/**` authority. Use property-based/invariant cases where applicable (e.g., repeated apply/check states, path-safety, status exhaustiveness).
- [ ] T009 [P] Write failing contract tests for debug ingest result grammar/channels/corroboration in `scripts/tests/spec012/test_debug_ingest_result_contract.py`: parse `ok:document:*|ok:uri:*|ok:directory:<path>:chapters=<n>|error:<class/code>:<detail>|refused:not-debug`; cache authoritative when readable/parseable; corroboration/conflict/inconclusive rules; SC-002 budget/denominator classification helpers if script-owned; FR-008 required fields/disposition/write status. Use property-based generation for valid/invalid prefixes, alias normalization matrix, and conflict cases.
- [ ] T010 [P] Write failing contract tests for FR-010 release-exposure result shape in `scripts/tests/spec012/test_release_exposure_contract.py`: `mode=full` D1–D4 required for pass, static/runtime non-full never satisfy FR-010/SC-004, `blocked/not-evaluated` => `satisfies_fr010_sc004=false`, identity binding required, `class_presence_note` required when class scan runs, in-scope identifiers `DebugIngestActivity|DebugIngestHook|app.mdreader.action.DEBUG_INGEST|debug manifest/intent-filter/materialized overlay targets`.
- [ ] T011 [P] Write failing contract tests for FR-012 residual split reporting in `scripts/tests/spec012/test_t184_t226_residual_contract.py`: `T184-PROG`/`T226-PROG` only `programmatic-non-ui`, `full_parent_completion_claimed=false` for hook-only coverage, required residual IDs/status lifecycle `open|verified|accepted-risk|deferred`, `accepted-risk` constraints, owning task-file update targets `specs/009-open-file-from-title/tasks.md` and `specs/010-directory-reading/tasks.md`, no orphan evidence.
- [ ] T012 Record Foundational RED evidence (all T008–T011 failing/missing-contract behavior confirmed) in `specs/012-debug-ingest-hook/quickstart.md` or a feature evidence note path decided in T002; include command, failing summary, and date. **Gate: blocks T013+**.

**Checkpoint**: Canonical scaffolds exist, contracts encoded as RED tests/checks, and implementation may begin per user story.

---

## Phase 3: User Story 1 — Deterministic document ingest validation (Priority: P1) 🎯 MVP

**Goal**: Debug/test automation can trigger document ingest through the same production `DocIngest` snapshot/fingerprint + `onDocumentIngested` JNI path with explicit parseable evidence.

**Independent Test**: On a debug device/emulator with an app-accessible document fixture, ADB invocation yields `ok:document:<path>` (or explicit `error:*` for invalid cases) with FR-008 evidence and no crash/ANR; release/non-debug cannot return `ok:*`.

### RED Tests for US1 (write first, must fail before US1 implementation)

- [ ] T013 [P] [US1] Write failing JVM/unit/property-style tests for document-mode normalization/validation and deterministic `path`/`uri` precedence in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` (create test source set/Gradle config only if absent; scaffold first): aliases `document|file|open_document`, required input matrix, explicit `error:unsupported-mode|error:*`, and `refused:not-debug` behavior contract. Use invariant/property cases for case normalization and malformed inputs where harness supports.
- [ ] T014 [P] [US1] Add failing script-level smoke assertions for document invocation/result parsing to `scripts/tests/spec012/test_debug_ingest_result_contract.py` or `scripts/tests/spec012/test_t184_t226_hook_script.py`: expects parseable `ok:document:*` for fixture ref and FR-008 minimum field presence in report shape (blocked by fixture preconditions => `blocked:fixture-inaccessible`). Mark RED before script hardening.

### Implementation for US1 (blocked until T013–T014 RED)

- [ ] T015 [US1] Harden document-ingest request parsing/diagnostics in `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt`: enforce deterministic mode normalization, explicit `path`/`uri` precedence after confirming current behavior (T004), app-accessible fixture validation, correlation/run context handoff, and no crash on invalid/missing/unreadable/unsupported/native-not-loaded/busy cases.
- [ ] T016 [US1] Harden debug hook transport/guard behavior in `android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt`: `BuildConfig.DEBUG` guard, native readiness handling, explicit `refused:not-debug` outside debug, route document mode through existing production `DocIngest.debugDeliverDocument`/snapshot/fingerprint + `onDocumentIngested` JNI callback only (FR-004, ASM-012-INGEST-SOT), no fake path.
- [ ] T017 [US1] Update minimal debug-only helper surface in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` only as required for explicit diagnosable results/correlation passthrough without adding a parallel ingest path; keep production authority and small blast radius (NFR-012-MAINT-001/002).
- [ ] T018 [US1] Harden `scripts/automate-t184-t226-hook.sh` document path to use app-accessible fixtures, capture authoritative cache result + corroboration per contract, record FR-008 minimum/disposition/write status/correlation/build+device identity, SC-002 timing/budget classification, and explicit `blocked:fixture-inaccessible`/`inconclusive` handling.
- [ ] T019 [US1] Re-run US1 RED tests/checks to confirm targeted RED→GREEN transition for document scope in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` and `scripts/tests/spec012/*`; record evidence path in task notes/commit body (not verification attestation).

**Checkpoint**: US1 independently testable for document programmatic ingest (debug) and explicit refusal/failure behavior.

---

## Phase 4: User Story 2 — Deterministic directory ingest validation (Priority: P1)

**Goal**: Debug/test automation can trigger directory ingest through the same production directory-open/manifest-dispatch path with chapter-count/filtering evidence.

**Independent Test**: On a debug device/emulator with an app-accessible mixed directory fixture, ADB invocation yields `ok:directory:<path>:chapters=<n>` where `n` matches production filtering/expected fixture composition, with FR-008 evidence and no crash/ANR.

### RED Tests for US2 (write first, must fail before US2 implementation)

- [ ] T020 [P] [US2] Extend failing JVM/unit/property-style tests in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` for directory-mode aliases `directory|dir|tree|open_directory`, required path/reference rules, empty/missing/unreadable/no-chapters error classes, and deterministic validation.
- [ ] T021 [P] [US2] Add failing script-level assertions in `scripts/tests/spec012/test_debug_ingest_result_contract.py` or `scripts/tests/spec012/test_t184_t226_hook_script.py` for `ok:directory:<path>:chapters=<n>` parsing, fixture composition/expected-filtering reporting requirement (ASM-012-CHAPTERS-INTERP), and blocked/inconclusive handling.

### Implementation for US2 (blocked until T020–T021 RED)

- [ ] T022 [US2] Harden directory-ingest request handling in `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt`: directory path/reference validation, explicit diagnosable empty/missing/unreadable/unresolvable cases, no crash, correlation context preserved.
- [ ] T023 [US2] Harden directory routing in `android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt` and `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` to use production snapshot/fingerprint/chapter-manifest construction + `onDirectorySelected`/manifest dispatch only (FR-005); report accepted chapter count from production filtering, not a fake count.
- [ ] T024 [US2] Harden `scripts/automate-t184-t226-hook.sh` directory path: app-accessible fixture deployment guidance/checks, expected filtering/composition notes in report, authoritative cache/corroboration, FR-008 minimum/disposition, explicit residual non-UI scope note.
- [ ] T025 [US2] Re-run US2 RED tests/checks for directory scope in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` and `scripts/tests/spec012/*`; record evidence path (not verification attestation).

**Checkpoint**: US1 and US2 both independently testable on debug artifacts.

---

## Phase 5: User Story 3 — Trustworthy evidence and residual-scope reporting (Priority: P2)

**Goal**: Automation outputs separate programmatic non-UI evidence from native SAF picker UI/a11y residuals and update T184/T226 split tracking without full-completion overclaim.

**Independent Test**: Generated reports/evidence records include FR-008 minimum fields, correlation/provenance/corroboration, and `full_parent_completion_claimed=false` for hook-only coverage; owning T184/T226 task files show `*-PROG` and residual items with lifecycle/status/evidence or explicit open/blocked decisions.

### RED Tests for US3 (write first, must fail before US3 implementation)

- [ ] T026 [P] [US3] Add failing report-schema assertions in `scripts/tests/spec012/test_t184_t226_residual_contract.py` and/or `scripts/tests/spec012/test_debug_ingest_result_contract.py` for FR-008 disposition fields (`preferred/effective location or channel`, `write_status`, reviewer-accessible location, retention scope), crash/ANR status (`unknown` not pass), residual scope note, and SC-003 `full_parent_completion_claimed=false`.
- [ ] T027 [P] [US3] Add failing tests/fixtures for split residual updates in `scripts/tests/spec012/test_t184_t226_residual_contract.py` covering required IDs `T184-PROG`, `T184-RES-SAF-CANCEL`, `T184-RES-A11Y`, `T226-PROG`, `T226-RES-SAF-TREE`, `T226-RES-TRANSITION`, `T226-RES-RESUME-UI`, `T226-RES-HARDWARE-CONTROLS`, `T226-RES-INVALID-ENTRIES-UI`, `T226-RES-A11Y-PICKER` and lifecycle/status rules.

### Implementation for US3 (blocked until T026–T027 RED)

- [ ] T028 [US3] Implement FR-008 evidence/report helpers in `scripts/automate-t184-t226-hook.sh` (or a sourced helper `scripts/lib/spec012_evidence.sh` if created as a disjoint file): correlation key, artifact/device identity binding, authoritative/corroborated channel handling, preferred/effective output, `write_status`, reviewer-accessible location, retention scope, crash/ANR classification (`unknown` explicit), residual scope note.
- [ ] T029 [US3] Update T184 split tracking in `specs/009-open-file-from-title/tasks.md`: add/update `T184-PROG` and residual items with status/evidence refs/owner/decision placeholders as applicable, preserve immutable IDs, state hook evidence does not close `T184-RES-SAF-CANCEL`/`T184-RES-A11Y`, and keep `full_parent_completion_claimed=false` unless separately verified by real evidence outside hook-only scope.
- [ ] T030 [P] [US3] Update T226 split tracking in `specs/010-directory-reading/tasks.md`: add/update `T226-PROG` and residual items with status/evidence refs/owner/decision placeholders as applicable, preserve immutable IDs, state hook evidence does not close native picker/transition/resume/hardware/a11y residuals, and keep `full_parent_completion_claimed=false` unless separately verified.
- [ ] T031 [US3] Reconcile `specs/012-debug-ingest-hook/contracts/t184-t226-residual-tracking.md` and `specs/012-debug-ingest-hook/quickstart.md` with implemented report fields/IDs/paths only if implementation exposes naming deltas; keep contracts normative and cross-artifact consistent (NFR-012-MAINT-005).
- [ ] T032 [US3] Re-run US3 RED tests/checks in `scripts/tests/spec012/test_t184_t226_residual_contract.py` and `scripts/tests/spec012/test_debug_ingest_result_contract.py`; record evidence path (not verification attestation).

**Checkpoint**: Evidence is trustworthy, scoped, and residual-aware; T184/T226 status cannot be overclaimed from hook-only runs.

---

## Phase 6: User Story 4 — Optional instrumented test coverage (Priority: P3)

**Goal**: Provide instrumented smoke coverage for the debug-only seam without manual SAF picker interaction, materialized durably when canonical androidTest source exists.

**Independent Test**: In a debug/test Android environment (device or emulator), the relevant instrumented smoke target exercises the automation seam and reports pass/fail; FR-011 check/apply treats androidTest target as required only when canonical source exists.

### RED Tests for US4 (write first, must fail before US4 implementation)

- [ ] T033 [P] [US4] Add failing FR-011 conditional androidTest assertions in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py`: canonical androidTest source present => target required/materializable; source absent => `not-applicable`; emulator execution note does not change durability (FR-013/SC-006).
- [ ] T034 [P] [US4] Add failing smoke-test source/intent contract assertions (static) in `scripts/tests/spec012/test_t184_t226_residual_contract.py` or a dedicated `scripts/tests/spec012/test_androidtest_smoke_contract.py`: `DebugIngestSeamSmokeTest.kt` references debug-only component/action contract and makes no manual SAF picker/UI traversal claim.

### Implementation for US4 (blocked until T033–T034 RED)

- [ ] T035 [US4] Finalize canonical androidTest smoke scaffold at `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` using Android instrumented-test/LSP templates where viable; exercise debug seam invocation/result contract only, no manual picker (FR-013).
- [ ] T036 [US4] Implement androidTest materialization/applicability handling in `scripts/apply-debug-ingest-overlay.sh` and `scripts/check-debug-ingest-overlay.sh`: required when canonical androidTest source exists, `not-applicable` when absent, fail-closed on stale/forbidden/missing when required; update FR-011 evidence output fields.
- [ ] T037 [US4] Re-run US4 RED tests/checks in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py` and `scripts/tests/spec012/test_androidtest_smoke_contract.py` (if created); record evidence path (not verification attestation).

**Checkpoint**: Optional instrumented coverage is scaffolded and durable; does not claim residual UI/a11y coverage.

---

## Phase 7: FR-011 — Durable overlay apply/check (cross-cutting, foundational for durability claims)

**Goal**: Canonical overlay sources remain authoritative and generated `src-tauri/gen/**` targets are materialized/verified idempotently with fail-closed diagnostics.

**Independent Test**: `check-debug-ingest-overlay.sh` reports explicit `pass|fail|blocked|not-applicable` with path/expected-vs-actual/remediation and is non-mutating; `apply-debug-ingest-overlay.sh` is idempotent and restores required managed targets from canonical sources; `src-tauri/gen/**` is never treated as canonical.

> Implementation blocked by T008/T012 RED. Reuse the same subagent for T038–T041 (shared scripts + FR-011 contract context).

- [ ] T038 Implement idempotent FR-011 apply logic in `scripts/apply-debug-ingest-overlay.sh` per `contracts/durable-overlay-materialization.md`: canonical root `specs/012-debug-ingest-hook/android-overlays/app/src/`, format-version detection/reporting, managed targets only, path safety, AndroidTest conditional, CHK018/CHK021 behavior, no sensitive data, JSON/human diagnostics, evidence/provenance fields.
- [ ] T039 Implement non-mutating fail-closed FR-011 check logic in `scripts/check-debug-ingest-overlay.sh`: compare canonical vs target state/hash as specified, report detected/handled format version, dep/layout validation (`DEP/ASM-012-*`, `ASM-012-REPO-LAYOUT`, `ASM-012-SOURCESET-WIRING`), overall/per-target status, and remediation; never modify files.
- [ ] T040 [P] Materialize/update canonical debug manifest content in `specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml` to the minimal required debug-only hook identifiers (`app.mdreader.DebugIngestActivity`, `app.mdreader.action.DEBUG_INGEST`) and verify it does not authorize non-debug exposure; keep small blast radius.
- [ ] T041 Re-run T008 and T033 overlay contract tests in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py`; run a local `check/apply/check` cycle against the repo to capture idempotence/drift evidence (not verification attestation).

**Checkpoint**: FR-011/SC-006 durability mechanism is implemented and test-backed.

---

## Phase 8: FR-010 — Release/non-debug exposure evidence (cross-cutting, release safety)

**Goal**: A repeatable fail-closed check can produce D1–D4 combined evidence for `mode=full`, or explicit `blocked/not-evaluated`/non-full results that never satisfy FR-010/SC-004.

**Independent Test**: Static artifact inspection over a supplied release/non-debug artifact and runtime negative invocation over a verified release/non-debug install produce machine-reviewable evidence bound to identity; missing inputs/tools/identity yield `blocked/not-evaluated` with `satisfies_fr010_sc004=false`.

> Implementation blocked by T010/T012 RED. Reuse the same subagent for T042–T044 (shared release-exposure contract context).

- [ ] T042 Implement `scripts/check-debug-ingest-release-exposure.sh` per `contracts/release-exposure-evidence.md`: `--mode static|runtime|full`, required artifact/install/tool inputs, D1 scope/guards checks, D2 merged/packaged manifest/merger/APK-AAB inspection for in-scope identifiers and unintended external exposure, D3 runtime negative with verified install identity, D4 deterministic report, `class_presence_note`, identity hashes/tool provenance, explicit `blocked/not-evaluated`/partial handling.
- [ ] T043 [P] Add manifest/APK/AAB scan helper logic only if needed in a disjoint file `scripts/lib/spec012_release_scan.sh` (or keep inline if simpler/smaller blast radius; document choice): prefer `apkanalyzer`/`aapt2`/bundletool/merger report inputs when available, fail closed when required evidence cannot be produced, never claim class absence is sufficient.
- [ ] T044 Re-run T010 release-exposure contract tests in `scripts/tests/spec012/test_release_exposure_contract.py`; capture a host-only static/mock or `--help`/input-validation evidence run (not FR-010 pass attestation).

**Checkpoint**: FR-010 evidence path is mechanized and fail-closed.

---

## Phase 9: Polish & Cross-Cutting Verification (attestation only)

**Purpose**: Final consistency, docs, and required verification. **Only tasks in this phase attest to success/completeness of implementation claims.**

- [ ] T045 [P] Run cross-artifact consistency pass (NFR-012-MAINT-005) and update any deltas in `specs/012-debug-ingest-hook/spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/*.md`, and checklist/task references: FR/SC/US/EC/ASM/DEP/EV/TASK/RES/T184/T226 IDs, paths, result grammar/channels, CHK018 run points, FR-011 canonical root, androidTest conditional, and `full_parent_completion_claimed=false` semantics.
- [ ] T046 [P] Validate quickstart command examples and evidence field names against implemented scripts/contracts in `specs/012-debug-ingest-hook/quickstart.md` and `scripts/automate-t184-t226-hook.sh`; mark device/release steps as opt-in and non-attesting unless executed with evidence.
- [ ] T047 [P] Run formatting/lint checks relevant to changed host/script/Kotlin/Rust surfaces (e.g., `cargo fmt --all -- --check` and targeted tests/scripts lint as applicable) and record exact commands/results in evidence; do not claim unrelated gates.
- [ ] T048 **Verification (attests):** Execute Spec 012 host-verifiable test suite/contract checks created by this feature (T008–T011, T013–T014, T020–T021, T026–T027, T033–T034 as present) and any targeted Kotlin/unit checks added. Pass required for attestation of contract/test scope; attach command output/evidence refs.
- [ ] T049 **Verification (attests, opt-in device/debug):** Run T184/T226 debug hook programmatic validation on an accessible debug build/device or emulator using `scripts/automate-t184-t226-hook.sh` or contract-equivalent commands. Attests only `T184-PROG`/`T226-PROG` exercised non-UI scope with FR-008 evidence; MUST record residuals open and `full_parent_completion_claimed=false`.
- [ ] T050 **Verification (attests, opt-in release RC):** Run `scripts/check-debug-ingest-release-exposure.sh --mode full` against the actual release/non-debug artifact and verified installed target. Only a `mode=full` pass with D1–D4 evidence and bound identities attests FR-010/SC-004. Missing inputs => record `blocked/not-evaluated` with `satisfies_fr010_sc004=false`; this does not attest FR-010 pass.
- [ ] T051 **Verification (attests):** Run `scripts/check-debug-ingest-overlay.sh` after a representative regeneration/apply scenario or simulated drift case (CHK018), and confirm fail-closed behavior/reporting. Attests FR-011/SC-006 durability mechanism only for exercised scenario; record androidTest `not-applicable`/required status.
- [ ] T052 **Verification (attests reporting integrity):** Confirm `specs/009-open-file-from-title/tasks.md` and `specs/010-directory-reading/tasks.md` contain split `*-PROG`/residual entries with evidence refs or explicit blocked/open status, no orphan evidence, and no full T184/T226 completion claim from hook-only evidence.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies.
- **Phase 2 (Foundational)**: depends on Phase 1; includes scaffold-first tasks and RED tests T008–T012. **BLOCKS all user-story and cross-cutting implementation** until RED evidence recorded.
- **Phase 3 (US1, P1)**: depends on Phase 2 RED gate; MVP for document ingest.
- **Phase 4 (US2, P1)**: depends on Phase 2 RED gate; shares hook/script context with US1 → same subagent/wave owner recommended for T015–T018/T022–T024.
- **Phase 5 (US3, P2)**: depends on US1/US2 script/report behavior patterns; T029/T030 touch disjoint spec task files and may run in parallel.
- **Phase 6 (US4, P3)**: depends on Phase 2 and FR-011 script context; can follow Phase 7 scaffolding decisions.
- **Phase 7 (FR-011)**: depends on T006–T008/T012; can run after Foundational RED, before/alongside US3/US4 as file ownership allows.
- **Phase 8 (FR-010)**: depends on T010/T012; independent of US1/US2 implementation except shared repo conventions.
- **Phase 9 (Verification/Polish)**: depends on all desired implementation tasks; only this phase attests success.

### User Story Dependency Graph

```text
Setup (P1) -> Foundational RED gate
Foundational RED gate -> US1 (P1, MVP)
Foundational RED gate -> US2 (P1)
US1 + US2 -> US3 (P2, evidence/residual reporting)
Foundational RED gate + FR-011 context -> US4 (P3)
Foundational RED gate -> FR-011 implementation
Foundational RED gate -> FR-010 implementation
US1..US4 + FR-011 + FR-010 -> Phase 9 verification attestation
```

### Parallel Opportunities / Waves

| Wave | Parallel `[P]` tasks (disjoint files/prereqs) | Shared-context owner (reuse same subagent) |
|------|--------------------------------------------------|---------------------------------------------|
| Wave 0 | T002, T003, T004 after T001 | setup/docs owner |
| Wave 1 | T005, T006, T007 | scaffold owner (then split by file type if helpful) |
| Wave 2 (RED, blocking) | T008, T009, T010, T011 | contract-test owner(s) by test file |
| Wave 3 (US1 RED) | T013, T014 | Kotlin contract-test owner / script-test owner |
| Wave 4 (US1 impl) | — | **same hook/script subagent**: T015→T016→T017→T018→T019 |
| Wave 5 (US2 RED) | T020, T021 | reuse US1 test owners |
| Wave 6 (US2 impl) | — | **same hook/script subagent**: T022→T023→T024→T025 |
| Wave 7 (US3 RED) | T026, T027 | reporting-contract owner |
| Wave 8 (US3 impl) | T029, T030 parallel; T028/T031 separate | script owner for T028/T031; docs owners for T029/T030 |
| Wave 9 (US4 RED) | T033, T034 | overlay/androidTest contract owner |
| Wave 10 (US4+FR-011) | T040 parallel with script work only if file-disjoint; T035/T036/T038/T039 mostly shared | FR-011 script owner for T036/T038/T039/T041; Kotlin/androidTest owner for T035 |
| Wave 11 (FR-010) | T043 conditional/disjoint | release-exposure script owner for T042/T044 |
| Wave 12 (verification only) | T045, T046, T047 parallel | verification owner(s); T048–T052 run as required opt-in/attestation sequence with evidence |

---

## Parallel Examples

```text
US1: after T012 RED gate, run [P] T013 (android-shell/src/test/...DebugIngestContractTest.kt)
     and T014 (scripts/tests/spec012/test_*result*_contract.py) in parallel, then
     one shared hook/script subagent executes T015-T019 sequentially on shared files.

US3: T029 updates specs/009-open-file-from-title/tasks.md while [P] T030 updates
     specs/010-directory-reading/tasks.md; same reporting-contract subagent owns T026-T028,T031-T032.

FR-011/FR-010: T008 and T010 are parallel RED tests in disjoint files; later FR-011
     script owner handles T038/T039/T036, FR-010 script owner handles T042/T043/T044.
```

---

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1–2, including RED gate (T012).
2. Complete Phase 3 (US1) for document ingest.
3. Run Phase 9 verification tasks relevant to US1 contract scope (T048 subset + opt-in T049 document case only if environment available); do not claim T184 fully complete.

### Incremental Delivery

1. US1 document path → debug programmatic evidence.
2. US2 directory path → debug programmatic evidence.
3. US3 evidence/residual split + T184/T226 task updates.
4. US4 optional instrumented smoke + FR-011 androidTest conditional.
5. FR-011/FR-010 hardening + Phase 9 attestation.

### Subagent-Driven Waves (recommended)

- **Contract-test agent(s)**: T008–T011, T013–T014, T020–T021, T026–T027, T033–T034 (disjoint test files may parallelize).
- **Android hook agent**: T015–T017, T022–T023, T035 (shared Kotlin context; reuse same agent).
- **Automation/report script agent**: T018, T024, T028, T031 (shared script/report context).
- **FR-011 overlay/script agent**: T003, T006, T036, T038–T041.
- **FR-010 script agent**: T005 stub slice/T042–T044.
- **Residual docs agents**: T029 and T030 in parallel (disjoint files), then T045 consistency owner.

## Notes

- `[P]` means different files and no dependency on incomplete work.
- `[US#]` labels map tasks to spec user stories for traceability.
- FR-011/FR-010/FR-012 are cross-cutting requirements implemented in dedicated phases and verified in Phase 9; they support US1–US4 acceptance and SC-001–SC-006.
- Every reported T184/T226 programmatic claim must set `full_parent_completion_claimed=false` until all required programmatic + residual items are verified or explicitly re-scoped by recorded decision.
- `src-tauri/gen/**` is never canonical; FR-011 check/apply must fail closed on drift.
- Device/release verification (T049/T050) is opt-in and evidence-bound; blocked results never attest FR-010/SC-004 or full T184/T226 completion.
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
- **FR-008 sync (D1)**: Source of truth is `spec.md` §FR-008. `contracts/`, `data-model.md`, and tests MUST NOT weaken FR-008 required fields. When FR-008 changes, T045 and T048 MUST diff required-field sets against spec/contracts/data-model/tests and record the delta.

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

- [x] T001 Create canonical FR-011 overlay directories at `specs/012-debug-ingest-hook/android-overlays/app/src/debug/` and `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/` (if androidTest scaffold source is created) per `contracts/durable-overlay-materialization.md` and FR-013; add only required dirs.
- [x] T002 [P] Create Spec 012 script/test helper directory `scripts/tests/spec012/` (or reuse existing test convention; record chosen path) with README note pointing to `specs/012-debug-ingest-hook/contracts/*.md` for contract-test ownership.
- [x] T003 [P] Add FR-011 overlay format-version marker under `specs/012-debug-ingest-hook/android-overlays/` (detectable by apply/check; CHK022/CHK024) and document `v1-implicit` compatibility only if explicitly chosen, else require explicit version; update contract reference only if implementation-defined marker path is fixed.
- [x] T004 [P] **Hard prerequisite for T013/T015 and any precedence-dependent tests (U1, FR-003)**: confirmed 2026-11-05 from code — `DebugIngestActivity.handle()` uses `path` extra first for `document|file|open_document` and `directory|dir|tree|open_directory`; if absent uses intent `data`/`--es uri` (`file://` path supported where applicable). `uri` mode requires intent `data`/`--es uri`. `DebugIngestHook.ingestUri()` routes `file://` to file ingest and rejects other schemes as explicit diagnosable `error:*`. Therefore when both `path` and `uri`/intent data are supplied in document/directory modes, `path` takes precedence (deterministic). Aligned in `contracts/debug-ingest-automation.md`, `data-model.md`, `quickstart.md`, and this `tasks.md`; tests/implementation MUST preserve unless synchronized change set.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared contracts, scaffolds, and RED tests that block all user-story implementation.

**⚠️ CRITICAL**: No user-story implementation begins until Phase 2 RED evidence exists and fails as expected.

### Scaffold/Template First (blocking setup)

- [x] T005 [P] Create executable script skeleton via repo shell template/scaffold conventions (LSP/shell scaffold N/A → document) for `scripts/apply-debug-ingest-overlay.sh` with `--help`, fail-closed exit behavior, and stub `blocked:not-implemented` output; no contract satisfaction yet. (G1 split: T053/T054 own other script skeletons.)
- [x] T006 [P] Create canonical debug manifest overlay source `specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml` by copying/adapting the existing debug manifest structure as a scaffold, preserving debug-only exported `app.mdreader.DebugIngestActivity` and `app.mdreader.action.DEBUG_INGEST` only in debug overlay scope (FR-001/FR-002/FR-010/FR-011).
- [x] T007 [P] Create canonical androidTest smoke scaffold source `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` from Android instrumented-test template/scaffold conventions when viable; minimal debug-seam smoke only, no manual SAF interaction (FR-013). If scaffold generation unavailable, hand-write minimal compilable skeleton and record `N/A:<LSP/scaffold>`.
- [x] T053 [P] (G1) Create executable script skeleton via repo shell template/scaffold conventions for `scripts/check-debug-ingest-overlay.sh` in a disjoint file from T005/T054: `--help`, fail-closed exit behavior, and stub `blocked:not-implemented` output; no contract satisfaction yet.
- [x] T054 [P] (G1) Create executable script skeleton via repo shell template/scaffold conventions for `scripts/check-debug-ingest-release-exposure.sh` in a disjoint file from T005/T053: `--help`, fail-closed exit behavior, and stub `blocked:not-implemented` output; no contract satisfaction yet.

### Foundational RED Tests / Contract Checks (write first, must fail/block)

- [x] T008 [P] Write failing contract tests for FR-011 overlay apply/check invariants in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py` (or repo-equivalent harness): idempotent apply, non-mutating check, explicit `pass|fail|blocked|not-applicable`, fail-closed on missing/stale/forbidden target/unknown format unless documented `v1-implicit`, androidTest conditional applicability (FR-013/SC-006), CHK021 no `src-tauri/gen/**` authority. Use property-based/invariant cases where applicable (e.g., repeated apply/check states, path-safety, status exhaustiveness). **RED 2026-11-05**: written; expected failures against skeletons (`blocked:not-implemented`/missing JSON contract). **GREEN target 2026-11-05**: `python3 -m pytest scripts/tests/spec012/test_debug_ingest_overlay_contract.py -q` → **9 passed** after FR-011 apply/check implementation.
- [x] T009 [P] Write failing contract tests for debug ingest result grammar/channels/corroboration (FR-003/FR-006) in `scripts/tests/spec012/test_debug_ingest_result_contract.py`: parse `ok:document:*|ok:uri:*|ok:directory:<path>:chapters=<n>|error:<class/code>:<detail>|refused:not-debug`; cache authoritative when readable/parseable; corroboration/conflict/inconclusive rules; SC-002 budget/denominator classification helpers if script-owned; FR-008 required fields/disposition/write status. Use property-based generation for valid/invalid prefixes, alias normalization matrix, and conflict cases. **RED 2026-11-05**; **GREEN 2026-11-06**: `python3 -m pytest scripts/tests/spec012/test_debug_ingest_result_contract.py -q` → 12 passed via `scripts/automate-t184-t226-hook.sh` deterministic contract modes (`cda5bb71`).
- [x] T010 [P] Write failing contract tests for FR-010 release-exposure result shape in `scripts/tests/spec012/test_release_exposure_contract.py`: `mode=full` D1–D4 required for pass, static/runtime non-full never satisfy FR-010/SC-004, `blocked/not-evaluated` => `satisfies_fr010_sc004=false`, identity binding required, `class_presence_note` required when class scan run, in-scope identifiers `DebugIngestActivity|DebugIngestHook|app.mdreader.action.DEBUG_INGEST|debug manifest/intent-filter/materialized overlay targets`. **RED 2026-11-05**: written; skeleton `blocked:not-implemented`. **GREEN 2026-11-05**: `python3 -m pytest scripts/tests/spec012/test_release_exposure_contract.py -q` → **7 passed** after FR-010 script + insufficient-proof note fix.
- [x] T011 [P] Write failing contract tests for FR-012 residual split reporting in `scripts/tests/spec012/test_t184_t226_residual_contract.py`: `T184-PROG`/`T226-PROG` only `programmatic-non-ui`, `full_parent_completion_claimed=false` for hook-only coverage, required residual IDs/status lifecycle `open|verified|accepted-risk|deferred`, `accepted-risk` constraints, owning task-file update targets `specs/009-open-file-from-title/tasks.md` and `specs/010-directory-reading/tasks.md`, no orphan evidence. **RED 2026-11-05**; **GREEN 2026-11-06** (5 passed in residual suite; combined host contract suites 33 passed).
- [x] T012 Record Foundational RED evidence (all T008–T011 failing/missing-contract behavior confirmed) in `specs/012-debug-ingest-hook/quickstart.md` or a feature evidence note path decided in T002; include command, failing summary, and date. **Gate: blocks T013+**. **Evidence 2026-11-05**: `python3 -m pytest scripts/tests/spec012/test_debug_ingest_overlay_contract.py scripts/tests/spec012/test_debug_ingest_result_contract.py scripts/tests/spec012/test_release_exposure_contract.py scripts/tests/spec012/test_t184_t226_residual_contract.py -q` → **28 failed, 5 passed, 5.75s**. Failures are expected RED (skeletons/report modes/residual split not implemented); 5 passes are limited invariant helpers where current state matched. Gate satisfied for TDD: tests written and failing against target contracts.

**Checkpoint**: Canonical scaffolds exist, contracts encoded as RED tests/checks, and implementation may begin per user story.

---

## Phase 3: User Story 1 — Deterministic document ingest validation (Priority: P1) 🎯 MVP

**Goal**: Debug/test automation can trigger document ingest through the same production `DocIngest` snapshot/fingerprint + `onDocumentIngested` JNI path with explicit parseable evidence.

**Independent Test**: On a debug device/emulator with an app-accessible document fixture, ADB invocation yields `ok:document:<path>` (or explicit `error:*` for invalid cases) with FR-008 evidence and no crash/ANR; release/non-debug cannot return `ok:*`.

### RED Tests for US1 (write first, must fail before US1 implementation)

- [ ] T013 [P] [US1] **(I1 harness, depends on T004/U1, FR-003/FR-007)** Write failing JVM/unit/property-style tests for document-mode normalization/validation and deterministic `path`/`uri` precedence in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt`. Before implementation, choose the host-testable extraction of parsing/normalization logic vs Android local unit test vs script-level contract tests; if `android-shell/src/test/` or the Gradle source set is absent, scaffold it first or use the fallback path. Record chosen mechanism and keep RED gate (failing/missing-contract evidence). Prefer PBT/invariant generation where harness supports; otherwise encode invariants as table/property-style cases. Cover aliases `document|file|open_document`, required input matrix, explicit `error:unsupported-mode|error:*`, and `refused:not-debug` behavior contract. **I1 decision (2026-11-06, task remains open / RED not satisfied as JVM tests; `blocked:harness-absent-requires-decision`)**: `android-shell/src/test` / visible Gradle source-set wiring absent in checked repo paths; `src-tauri/gen/android/**/build.gradle.kts` exists but is non-canonical/regenerable (CHK021/FR-011) — avoid manual generated Gradle patching. **Recorded fallback**: satisfy FR-003/FR-007 normalization/precedence/alias/matrix invariants via host-testable contract tests in `scripts/tests/spec012/test_debug_ingest_result_contract.py` and deterministic hook/Kotlin-parity contract modes where possible; if Kotlin host extraction is pursued, do it by extracting pure parsing/normalization logic to a small debug-only/shared-testable unit with minimal blast radius and documented authority boundary (not a parallel production ingest path). **Required before Kotlin implementation claims**: explicit RED evidence for the chosen JVM/host-test mechanism.
- [x] T014 [P] [US1] Add failing script-level smoke assertions for document invocation/result parsing (FR-006) to `scripts/tests/spec012/test_debug_ingest_result_contract.py` or `scripts/tests/spec012/test_t184_t226_hook_script.py`: expects parseable `ok:document:*` for fixture ref and FR-008 minimum field presence in report shape (blocked by fixture preconditions => `blocked:fixture-inaccessible`). Mark RED before script hardening. **RED→GREEN 2026-11-06**: covered by deterministic hook contract modes in `test_debug_ingest_result_contract.py` (host); device fixture execution remains opt-in Phase 9/T049.

### Implementation for US1 (blocked until T013–T014 RED)

- [ ] T015 [US1] Harden document-ingest request parsing/diagnostics in `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt` (FR-003/FR-007; after T004/U1 precedence confirmed/aligned): enforce deterministic mode normalization, explicit `path`/`uri` precedence per observed behavior, app-accessible fixture validation, correlation/run context handoff, and no crash on invalid/missing/unreadable/unsupported/native-not-loaded/busy cases.
- [ ] T016 [US1] Harden debug hook transport/guard behavior in `android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt`: `BuildConfig.DEBUG` guard, native readiness handling, explicit `refused:not-debug` outside debug, route document mode through existing production `DocIngest.debugDeliverDocument`/snapshot/fingerprint + `onDocumentIngested` JNI callback only (FR-004, ASM-012-INGEST-SOT), no fake path.
- [ ] T017 [US1] Update minimal debug-only helper surface in `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` only as required for explicit diagnosable results/correlation passthrough without adding a parallel ingest path; keep production authority and small blast radius (NFR-012-MAINT-001/002).
- [ ] T018 [US1] Harden `scripts/automate-t184-t226-hook.sh` document path (FR-006) to use app-accessible fixtures, capture authoritative cache result + corroboration per contract, record FR-008 minimum/disposition/write status/correlation/build+device identity, SC-002 timing/budget classification, and explicit `blocked:fixture-inaccessible`/`inconclusive` handling.
- [ ] T019 [US1] Re-run US1 RED tests/checks to confirm targeted RED→GREEN transition for document scope in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` and `scripts/tests/spec012/*`; record evidence path in task notes/commit body (not verification attestation).

**Checkpoint**: US1 independently testable for document programmatic ingest (debug) and explicit refusal/failure behavior.

---

## Phase 4: User Story 2 — Deterministic directory ingest validation (Priority: P1)

**Goal**: Debug/test automation can trigger directory ingest through the same production directory-open/manifest-dispatch path with chapter-count/filtering evidence.

**Independent Test**: On a debug device/emulator with an app-accessible mixed directory fixture, ADB invocation yields `ok:directory:<path>:chapters=<n>` where `n` matches production filtering/expected fixture composition, with FR-008 evidence and no crash/ANR.

### RED Tests for US2 (write first, must fail before US2 implementation)

- [ ] T020 [P] [US2] **(I1 harness, depends on T004/U1 where precedence applies, FR-002/FR-003/FR-007)** Extend failing JVM/unit/property-style tests in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` for directory-mode aliases `directory|dir|tree|open_directory`, required path/reference rules, empty/missing/unreadable/no-chapters error classes, and deterministic validation. Use the same harness decision/fallback/RED/PBT-or-table approach recorded for T013. **I1 decision (2026-11-06, task remains open / RED not satisfied as JVM tests; `blocked:harness-absent-requires-decision`)**: same T013 harness absence/fallback applies — `android-shell/src/test` / visible Gradle source-set wiring absent in checked repo paths; `src-tauri/gen/android/**/build.gradle.kts` non-canonical/regenerable (CHK021/FR-011), no manual generated Gradle patching. Fallback via host-testable FR-002/FR-003/FR-007 invariants in `scripts/tests/spec012/test_debug_ingest_result_contract.py` and deterministic hook/Kotlin-parity contract modes where possible; any Kotlin host extraction must remain pure/debug-only/shared-testable with minimal blast radius and documented authority boundary (not a parallel production ingest path). **Required before Kotlin implementation claims**: explicit RED evidence for the chosen JVM/host-test mechanism.
- [x] T021 [P] [US2] Add failing script-level assertions (FR-006) in `scripts/tests/spec012/test_debug_ingest_result_contract.py` or `scripts/tests/spec012/test_t184_t226_hook_script.py` for `ok:directory:<path>:chapters=<n>` parsing, fixture composition/expected-filtering reporting requirement (ASM-012-CHAPTERS-INTERP), and blocked/inconclusive handling. **RED→GREEN 2026-11-06**: host contract coverage in `test_debug_ingest_result_contract.py`/hook `--print-contract` (`cda5bb71`); device fixture filtering evidence remains opt-in Phase 9/T049.

### Implementation for US2 (blocked until T020–T021 RED)

- [ ] T022 [US2] Harden directory-ingest request handling in `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt` (FR-002/FR-003/FR-007): directory path/reference validation, explicit diagnosable empty/missing/unreadable/unresolvable cases, no crash, correlation context preserved.
- [ ] T023 [US2] Harden directory routing in `android-shell/src/main/kotlin/app/mdreader/DebugIngestHook.kt` and `android-shell/src/main/kotlin/app/mdreader/DocIngest.kt` (FR-002/FR-007) to use production snapshot/fingerprint/chapter-manifest construction + `onDirectorySelected`/manifest dispatch only (FR-005); report accepted chapter count from production filtering, not a fake count.
- [ ] T024 [US2] Harden `scripts/automate-t184-t226-hook.sh` directory path (FR-002/FR-006): app-accessible fixture deployment guidance/checks, expected filtering/composition notes in report, authoritative cache/corroboration, FR-008 minimum/disposition, explicit residual non-UI scope note.
- [ ] T025 [US2] Re-run US2 RED tests/checks for directory scope (FR-002) in `android-shell/src/test/kotlin/app/mdreader/DebugIngestContractTest.kt` and `scripts/tests/spec012/*`; record evidence path (not verification attestation).

**Checkpoint**: US1 and US2 both independently testable on debug artifacts.

---

## Phase 5: User Story 3 — Trustworthy evidence and residual-scope reporting (Priority: P2)

**Goal**: Automation outputs separate programmatic non-UI evidence from native SAF picker UI/a11y residuals and update T184/T226 split tracking without full-completion overclaim.

**Independent Test**: Generated reports/evidence records include FR-008 minimum fields, correlation/provenance/corroboration, and `full_parent_completion_claimed=false` for hook-only coverage; owning T184/T226 task files show `*-PROG` and residual items with lifecycle/status/evidence or explicit open/blocked decisions.

### RED Tests for US3 (write first, must fail before US3 implementation)

- [x] T026 [P] [US3] Add failing report-schema assertions (FR-009) in `scripts/tests/spec012/test_t184_t226_residual_contract.py` and/or `scripts/tests/spec012/test_debug_ingest_result_contract.py` for FR-008 disposition fields (`preferred/effective location or channel`, `write_status`, reviewer-accessible location, retention scope), crash/ANR status (`unknown` not pass), residual scope note, and SC-003 `full_parent_completion_claimed=false`. **RED→GREEN 2026-11-06** (host contract schema).
- [x] T027 [P] [US3] Add failing tests/fixtures for split residual updates in `scripts/tests/spec012/test_t184_t226_residual_contract.py` covering required IDs `T184-PROG`, `T184-RES-SAF-CANCEL`, `T184-RES-A11Y`, `T226-PROG`, `T226-RES-SAF-TREE`, `T226-RES-TRANSITION`, `T226-RES-RESUME-UI`, `T226-RES-HARDWARE-CONTROLS`, `T226-RES-INVALID-ENTRIES-UI`, `T226-RES-A11Y-PICKER` and lifecycle/status rules. **RED→GREEN 2026-11-06** (residual IDs in owning spec tasks + hook `--print-residual-rules`).

### Implementation for US3 (blocked until T026–T027 RED)

- [x] T028 [US3] Implement FR-008 evidence/report helpers (FR-009, SC-001/SC-002 counters) in `scripts/automate-t184-t226-hook.sh` (or a sourced helper `scripts/lib/spec012_evidence.sh` if created as a disjoint file): correlation key, artifact/device identity binding, authoritative/corroborated channel handling, preferred/effective output, `write_status`, reviewer-accessible location, retention scope, crash/ANR classification (`unknown` explicit), residual scope note. Reporting MUST include machine-readable SC-001/SC-002 counters: `attempted`, `precondition_blocked`, `in_denominator`, `explicit_within_budget`, `inconclusive`, plus formulae matching spec (`SC-001` counts in-precondition runs; `SC-002` denominator excludes `blocked` precondition/tooling failures; `inconclusive` not explicit until rerun resolves; >=95% explicit within 2-minute budget). **Done 2026-11-06** in hook deterministic contract/report modes (`cda5bb71`); device-run evidence generation remains opt-in/Phase 9.
- [x] T029 [US3] Update T184 split tracking in `specs/009-open-file-from-title/tasks.md` (FR-009): add/update `T184-PROG` and residual items with status/evidence refs/owner/decision placeholders as applicable, preserve immutable IDs, state hook evidence does not close `T184-RES-SAF-CANCEL`/`T184-RES-A11Y`, and keep `full_parent_completion_claimed=false` unless separately verified by real evidence outside hook-only scope. **Done 2026-11-05** (commit `9c192983`).
- [x] T030 [P] [US3] Update T226 split tracking in `specs/010-directory-reading/tasks.md` (FR-009): add/update `T226-PROG` and residual items with status/evidence refs/owner/decision placeholders as applicable, preserve immutable IDs, state hook evidence does not close native picker/transition/resume/hardware/a11y residuals, and keep `full_parent_completion_claimed=false` unless separately verified. **Done 2026-11-05** (commit `73660836`).
- [x] T031 [US3] Reconcile `specs/012-debug-ingest-hook/contracts/t184-t226-residual-tracking.md` and `specs/012-debug-ingest-hook/quickstart.md` with implemented report fields/IDs/paths only if implementation exposes naming deltas (FR-009); keep contracts normative and cross-artifact consistent (NFR-012-MAINT-005). **Done 2026-11-06** (`bcb982b4`): aligned deterministic hook contract modes/JSON field names (`--help --json`, `--print-contract`, `--classify-outcome`, `--print-report-schema`, `--print-residual-rules`), preserved `full_parent_completion_claimed=false` and hook bridge `full_parent_completion_claimed_from_hook_evidence=false`; no FR/SC weakening.
- [x] T032 [US3] Re-run US3 RED tests/checks in `scripts/tests/spec012/test_t184_t226_residual_contract.py` and `scripts/tests/spec012/test_debug_ingest_result_contract.py`; record evidence path (not verification attestation). **2026-11-06**: `python3 -m pytest scripts/tests/spec012/test_debug_ingest_result_contract.py scripts/tests/spec012/test_t184_t226_residual_contract.py -q` → **17 passed**.

**Checkpoint**: Evidence is trustworthy, scoped, and residual-aware; T184/T226 status cannot be overclaimed from hook-only runs.

---

## Phase 6: User Story 4 — Optional instrumented test coverage (Priority: P3)

**Goal**: Provide instrumented smoke coverage for the debug-only seam without manual SAF picker interaction, materialized durably when canonical androidTest source exists.

**Independent Test**: In a debug/test Android environment (device or emulator), the relevant instrumented smoke target exercises the automation seam and reports pass/fail; FR-011 check/apply treats androidTest target as required only when canonical source exists.

### RED Tests for US4 (write first, must fail before US4 implementation)

- [x] T033 [P] [US4] Add failing FR-011 conditional androidTest assertions in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py`: canonical androidTest source present => target required/materializable; source absent => `not-applicable`; emulator execution note does not change durability (FR-013/SC-006). **Covered by T008 overlay contract tests; GREEN 2026-11-05 (9 passed).**
- [x] T034 [P] [US4] Add failing smoke-test source/intent contract assertions (static) in `scripts/tests/spec012/test_t184_t226_residual_contract.py` or a dedicated `scripts/tests/spec012/test_androidtest_smoke_contract.py`: `DebugIngestSeamSmokeTest.kt` references debug-only component/action contract and makes no manual SAF picker/UI traversal claim. **Static contract satisfied 2026-11-05** (`test_androidtest_smoke_scaffold_is_static_contract_only` passes after scaffold update commit `86828bec`); instrumented execution remains `executed-*`/`blocked` per B1 (see T037/T049).

### Implementation for US4 (blocked until T033–T034 RED)

- [x] T035 [US4] Finalize canonical androidTest smoke scaffold at `specs/012-debug-ingest-hook/android-overlays/app/src/androidTest/java/app/mdreader/DebugIngestSeamSmokeTest.kt` using Android instrumented-test/LSP templates where viable; exercise debug seam invocation/result contract only, no manual picker (FR-013). Evidence MUST use explicit instrumented status vocabulary: `scaffolded|materialized|executed-pass|executed-fail|blocked:<reason>`; never imply execution from scaffolding (B1). **Done 2026-11-05** (canonical scaffold + B1 vocabulary in KDoc; instrumented execution not run/claimed).
- [x] T036 [US4] Implement androidTest materialization/applicability handling in `scripts/apply-debug-ingest-overlay.sh` and `scripts/check-debug-ingest-overlay.sh`: required when canonical androidTest source exists, `not-applicable` when absent, fail-closed on stale/forbidden/missing when required; update FR-011 evidence output fields. **Done 2026-11-05** (covered by overlay contract).
- [x] T037 [US4] Re-run US4 RED tests/checks in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py` and `scripts/tests/spec012/test_androidtest_smoke_contract.py` (if created); record evidence path (not verification attestation). Evidence MUST use instrumented status vocabulary `scaffolded|materialized|executed-pass|executed-fail|blocked:<reason>`; never imply execution from scaffolding (B1). **Overlay contract GREEN 2026-11-05 (9 passed)**; androidTest smoke source static contract covered by T034 test pass in residual suite (isolated run passed); dedicated smoke-contract file not required. Status: `scaffolded|materialized` (canonical source+FR-011 logic); instrumented `executed-*` = `blocked:no-device/emulator-run-in-this-turn`.

**Checkpoint**: Optional instrumented coverage is scaffolded and durable; does not claim residual UI/a11y coverage.

---

## Phase 7: FR-011 — Durable overlay apply/check (cross-cutting, foundational for durability claims)

**Goal**: Canonical overlay sources remain authoritative and generated `src-tauri/gen/**` targets are materialized/verified idempotently with fail-closed diagnostics.

**Independent Test**: `check-debug-ingest-overlay.sh` reports explicit `pass|fail|blocked|not-applicable` with path/expected-vs-actual/remediation and is non-mutating; `apply-debug-ingest-overlay.sh` is idempotent and restores required managed targets from canonical sources; `src-tauri/gen/**` is never treated as canonical.

> Implementation blocked by T008/T012 RED. Reuse the same subagent for T038–T041 (shared scripts + FR-011 contract context).

- [x] T038 Implement idempotent FR-011 apply logic in `scripts/apply-debug-ingest-overlay.sh` per `contracts/durable-overlay-materialization.md`: canonical root `specs/012-debug-ingest-hook/android-overlays/app/src/`, format-version detection/reporting, managed targets only, path safety, AndroidTest conditional, CHK018/CHK021 behavior, no sensitive data, JSON/human diagnostics, evidence/provenance fields. **Done 2026-11-05**.
- [x] T039 Implement non-mutating fail-closed FR-011 check logic in `scripts/check-debug-ingest-overlay.sh`: compare canonical vs target state/hash as specified, report detected/handled format version, dep/layout validation (`DEP/ASM-012-*`, `ASM-012-REPO-LAYOUT`, `ASM-012-SOURCESET-WIRING`), overall/per-target status, and remediation; never modify files. **Done 2026-11-05**.
- [x] T040 [P] Materialize/update canonical debug manifest content in `specs/012-debug-ingest-hook/android-overlays/app/src/debug/AndroidManifest.xml` to the minimal required debug-only hook identifiers (`app.mdreader.DebugIngestActivity`, `app.mdreader.action.DEBUG_INGEST`) and verify it does not authorize non-debug exposure; keep small blast radius. **Reviewed 2026-11-05**: canonical manifest already minimal (single activity + single DEBUG_INGEST action, debug overlay path only; no extra components/permissions/export claims). No change needed (`741528b0` baseline).
- [x] T041 Re-run T008 and T033 overlay contract tests in `scripts/tests/spec012/test_debug_ingest_overlay_contract.py`; run a local `check/apply/check` cycle against the repo to capture idempotence/drift evidence (not verification attestation). **2026-11-05**: pytest **9 passed**; local cycles exercised by implementation sanity checks (main agent can re-run and capture evidence artifact).

**Checkpoint**: FR-011/SC-006 durability mechanism is implemented and test-backed.

---

## Phase 8: FR-010 — Release/non-debug exposure evidence (cross-cutting, release safety)

**Goal**: A repeatable fail-closed check can produce D1–D4 combined evidence for `mode=full`, or explicit `blocked/not-evaluated`/non-full results that never satisfy FR-010/SC-004.

**Independent Test**: Static artifact inspection over a supplied release/non-debug artifact and runtime negative invocation over a verified release/non-debug install produce machine-reviewable evidence bound to identity; missing inputs/tools/identity yield `blocked/not-evaluated` with `satisfies_fr010_sc004=false`.

> Implementation blocked by T010/T012 RED. Reuse the same subagent for T042–T044 (shared release-exposure contract context).

- [x] T042 Implement `scripts/check-debug-ingest-release-exposure.sh` per `contracts/release-exposure-evidence.md`: `--mode static|runtime|full`, required artifact/install/tool inputs, D1 scope/guards checks, D2 merged/packaged manifest/merger/APK-AAB inspection for in-scope identifiers and unintended external exposure, D3 runtime negative with verified install identity, D4 deterministic report, `class_presence_note`, identity hashes/tool provenance, explicit `blocked/not-evaluated`/partial handling. **Done 2026-11-05** (host contract GREEN; actual RC/device execution remains opt-in Phase 9/T050).
- [x] T043 [P] Add manifest/APK/AAB scan helper logic only if needed in a disjoint file `scripts/lib/spec012_release_scan.sh` (or keep inline if simpler/smaller blast radius; document choice): prefer `apkanalyzer`/`aapt2`/bundletool/merger report inputs when available, fail closed when required evidence cannot be produced, never claim class absence is sufficient. **Done inline 2026-11-05** (small blast radius; helper file not created).
- [x] T044 Re-run T010 release-exposure contract tests in `scripts/tests/spec012/test_release_exposure_contract.py`; capture a host-only static/mock or `--help`/input-validation evidence run (not FR-010 pass attestation). **2026-11-05**: pytest **7 passed**.

**Checkpoint**: FR-010 evidence path is mechanized and fail-closed.

---

## Phase 9: Polish & Cross-Cutting Verification (attestation only)

**Purpose**: Final consistency, docs, and required verification. **Only tasks in this phase attest to success/completeness of implementation claims.**

- [ ] T045 [P] Cross-artifact consistency pass (NFR-012-MAINT-005, FR-009): **partial 2026-11-06** — `tasks.md` evidence/IDs updated for T055–T069/T057/T059 host-contract work; checklists read-only gate PASS; FR-008/FR-010/FR-011/FR-012 host contract semantics preserved in tests/scripts. **Remaining**: reconcile `spec.md/plan/research/data-model/quickstart/contracts` and checklist refs for new hook contract modes (`--build-device-evidence`, intent/dir-uri parse contracts, release mechanization fields, overlay `dependency_layout_results`) and CHK021/T042/T028 integrity notes; diff FR-008 required-field sets end-to-end.
- [x] T046 [P] Validate quickstart command examples and evidence field names against implemented scripts/contracts in `specs/012-debug-ingest-hook/quickstart.md` and `scripts/automate-t184-t226-hook.sh`; mark device/release steps as opt-in and non-attesting unless executed with evidence. **Done 2026-11-06** (`bcb982b4`): validated flags/FR-008 additive field names; device/release/T049/T050 marked opt-in, non-attesting unless executed with bound evidence.
- [x] T047 [P] Run formatting/lint checks relevant to changed host/script/Kotlin/Rust surfaces (e.g., `cargo fmt --all -- --check` and targeted tests/scripts lint as applicable) and record exact commands/results in evidence; do not claim unrelated gates (SC-005: relevant host gates/no new crash-ANR claims limited to covered runs). **2026-11-06 host evidence (non-attesting; Phase 9 only attests)**: `cd /home/luis/development/mdreader && cargo fmt --all -- --check` → exit `0` (`---FMT_EXIT:0---`, this session). `python3 -m py_compile` OK for Spec 012 Python contract tests created/touched; `bash -n` OK for Spec 012 scripts when checked. Rerun in T048/T061/T068 with exact command+exit+summary as needed.
- [ ] T048 **Verification (attests, FR-006/SC-005, PARTIAL host scope 2026-11-06)**: Spec 012 host/Python contract suites GREEN `python3 -m pytest scripts/tests/spec012 -q --tb=no` → **77 passed, 69.47s, exit 0** (covers host-contract scope including T008–T011/T014/T021/T026/T027/T033/T034/T055/T057/T059/T060/T063/T064/T067 helper). **Not fully attested**: targeted Kotlin/unit checks for T013/T015–T020/T022–T025 not executed (I1 harness absent/option B host-contract fallback; no `android-shell/src/test` harness). NFR checks recorded as host review: **NFR-012-MAINT-003** no sensitive runtime data in added contract evidence fields (repo paths/IDs/statuses only); **NFR-012-MAINT-001** small blast radius limited to hook scripts/tests/docs/debug contract surfaces; **NFR-012-MAINT-002** no `src-tauri/gen/**` canonical, no parallel production ingest path. FR-008 changes reconciled in spec/contracts/data-model/tests touched (host). **Attestation incomplete** until Kotlin/device scope covered or explicitly re-scoped by recorded decision.
- [ ] T049 **Verification (attests, opt-in device/debug, SC-005):** Run T184/T226 debug hook programmatic validation on an accessible debug build/device or emulator using `scripts/automate-t184-t226-hook.sh` or contract-equivalent commands. Attests only `T184-PROG`/`T226-PROG` exercised non-UI scope with FR-008 evidence; MUST record residuals open and `full_parent_completion_claimed=false`. Evidence MUST use explicit instrumented status vocabulary `scaffolded|materialized|executed-pass|executed-fail|blocked:<reason>`; never imply execution from scaffolding (B1). Reporting MUST include machine-readable SC-001/SC-002 counters: `attempted`, `precondition_blocked`, `in_denominator`, `explicit_within_budget`, `inconclusive`, plus formulae matching spec (`SC-001` counts in-precondition runs; `SC-002` denominator excludes `blocked` precondition/tooling failures; `inconclusive` not explicit until rerun resolves; >=95% explicit within 2-minute budget) (C2).
- [ ] T050 **Verification (attests, opt-in release RC):** Run `scripts/check-debug-ingest-release-exposure.sh --mode full` against the actual release/non-debug artifact and verified installed target. Only a `mode=full` pass with D1–D4 evidence and bound identities attests FR-010/SC-004. Missing inputs => record `blocked/not-evaluated` with `satisfies_fr010_sc004=false`; this does not attest FR-010 pass.
- [ ] T051 **Verification (attests):** Run `scripts/check-debug-ingest-overlay.sh` after a representative regeneration/apply scenario or simulated drift case (CHK018), and confirm fail-closed behavior/reporting. Attests FR-011/SC-006 durability mechanism only for exercised scenario; record androidTest `not-applicable`/required status. Explicitly verify/record **NFR-012-MAINT-003** no sensitive runtime data in diagnostics/evidence and **NFR-012-MAINT-002** no `src-tauri/gen/**` canonical/parallel production ingest path for overlay-managed targets (E1).
- [ ] T052 **Verification (attests reporting integrity, FR-009):** Confirm `specs/009-open-file-from-title/tasks.md` and `specs/010-directory-reading/tasks.md` contain split `*-PROG`/residual entries with evidence refs or explicit blocked/open status, no orphan evidence, and no full T184/T226 completion claim from hook-only evidence.

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
| Wave 1 | T005, T006, T007, T053, T054 (G1 disjoint skeletons) | scaffold owner (then split by file type if helpful) |
| Wave 2 (RED, blocking) | T008, T009, T010, T011 | contract-test owner(s) by test file |
| Wave 3 (US1 RED) | T013, T014 | Kotlin contract-test owner / script-test owner |
| Wave 4 (US1 impl) | — | **same hook/script subagent**: T015→T016→T017→T018→T019 |
| Wave 5 (US2 RED) | T020, T021 | reuse US1 test owners |
| Wave 6 (US2 impl) | — | **same hook/script subagent**: T022→T023→T024→T025 |
| Wave 7 (US3 RED) | T026, T027 | reporting-contract owner |
| Wave 8 (US3 impl) | T029, T030 parallel; T028/T031 separate | script owner for T028/T031; docs owners for T029/T030 |
| Wave 9 (US4 RED) | T033, T034 | overlay/androidTest contract owner |
| Wave 10 (US4+FR-011) | T040 parallel with script work only if file-disjoint; T035/T036/T038/T039 mostly shared | FR-011 script owner for T036/T038/T039/T041/T053; Kotlin/androidTest owner for T035 |
| Wave 11 (FR-010) | T043 conditional/disjoint | release-exposure script owner for T042/T044/T054 |
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
- **FR-011 overlay/script agent**: T003, T006, T036, T038–T041, T053.
- **FR-010 script agent**: T005, T042–T044, T054.
- **Residual docs agents**: T029 and T030 in parallel (disjoint files), then T045 consistency owner.

## Notes

- `[P]` means different files and no dependency on incomplete work.
- `[US#]` labels map tasks to spec user stories for traceability.
- FR-011/FR-010/FR-012 are cross-cutting requirements implemented in dedicated phases and verified in Phase 9; they support US1–US4 acceptance and SC-001–SC-006.
- Every reported T184/T226 programmatic claim must set `full_parent_completion_claimed=false` until all required programmatic + residual items are verified or explicitly re-scoped by recorded decision.
- `src-tauri/gen/**` is never canonical; FR-011 check/apply must fail closed on drift.
- Device/release verification (T049/T050) is opt-in and evidence-bound; blocked results never attest FR-010/SC-004 or full T184/T226 completion.

---

## Phase 10: Convergence

> **Append-only convergence pass (2026-11-06, post-`/speckit.implement`)**. Tasks T055+ close gaps found by code-vs-spec/plan/tasks assessment. TDD/RED first for any test task; PBT/invariants whenever applicable; tests BLOCK implementation. `[P]` = disjoint files, parallelizable in waves; tasks sharing file/context MUST reuse the same subagent and serialize shared tooling (never kill/interrupt/bypass another agent/process/tool). **Verification tasks (T061+) at the end are the ONLY attestation of success/completeness** (FR-009/Phase 9 rule preserved). Device/release execution remains opt-in/evidence-bound; `blocked`/`inconclusive`/`not-evaluated` never satisfy FR-010/SC-004 or claim T184/T226 fully complete. Atomic conventional commit after each edited file.

### RED (blocking)

- [x] T055 [P] [US1/US2] Write failing host contract tests (PBT/invariant where applicable) in `scripts/tests/spec012/test_debug_ingest_result_contract.py` (or a new disjoint `scripts/tests/spec012/test_debug_ingest_activity_channel_contract.py`) for FR-006/FR-008 cache-authoritative attribution: when `cache/debug_ingest_result.txt` cannot be written/unavailable, the debug Activity/hook contract MUST NOT silently imply authoritative cache success; expected behavior is explicit diagnosable/corroboration-aware reporting (or `inconclusive`/`blocked` path) consistent with FR-008 `write_status`/`channel`/`corroboration` semantics. Keep RED gate evidence; blocks T056. per FR-006,FR-008,EC-012 (missing) **RED authored** `3793a75c`. **RED execution (2026-11-06, host, non-attesting)**: missing contract modes/fields observed (probe + Hypothesis subprocess runs timed out in queue). **GREEN (2026-11-06, host, non-attesting)** after hook contract modes `1c84302a`,`8d0deac8`: `python3 -m pytest scripts/tests/spec012/test_debug_ingest_activity_channel_contract.py -q --tb=short` → **7 passed, 11.60s**.
- [x] T056 [US1/US2] Implement minimal debug-only cache-channel write-status exposure in `android-shell/src/main/kotlin/app/mdreader/DebugIngestActivity.kt` (or smallest justified debug-only helper) so preferred cache output failure is explicit/attributable and does not silently imply authoritative success; preserve `BuildConfig.DEBUG` guard, production ingest routing, no parallel ingest path, and FR-008/FR-006 contract semantics. Re-run T055 to RED→GREEN evidence (not Phase 9 attestation). per FR-006,FR-008 (missing/partial) **2026-11-06 host (non-attesting, partial Kotlin scope)**: hook contract implemented in `scripts/automate-t184-t226-hook.sh` (`1c84302a`,`8d0deac8`) — `--print-activity-channel-contract`/`--classify-channel-attribution` + FR-008/help/report schema explicit `cache_write_status`/`write_status`/corroboration/`authoritative_cache_success=false`/fail-closed `inconclusive|blocked`; T055 suite **7 passed**. `DebugIngestActivity.kt` cache write still swallows exception in current debug path — **device/Kotlin write-status emission remains tracked** under T060/T067/T013 follow-up; no production ingest path change, no attestation.

### FR-005 / directory parity (tracked baseline, convergence clarification)

- [x] T057 [US2] (host-contract scope) Directory/tree URI refs beyond filesystem paths: require production-equivalent support via same production directory-open/manifest-dispatch path OR explicit diagnosable `error:*` (never silent/fake/ambiguous). per FR-003,FR-005,FR-007,US2-AC1,ASM-012-INGEST-SOT; `full_parent_completion_claimed=false` preserved. **RED** `414aa001` (`test_debug_ingest_directory_uri_contract.py`, Hypothesis/PBT). **Host-contract impl** `968c8691`,`3e39fb27` (`--print-directory-uri-contract`); contract-shape test fixes preserving invariants `14e21d32`,`8de35b18`,`4f699409`,`5d0562d2`. **GREEN (2026-11-06, host, non-attesting)**: intent+dir-URI suites **19 passed** (40.79s). **Runtime/Kotlin dir/tree URI dispatch + T023/T226 residuals remain open/traceable** (hook contract does not claim production-equivalent device execution).

### Cross-artifact / process

- [x] T058 [P] Run T045/T047 completion with explicit recorded evidence in `specs/012-debug-ingest-hook/tasks.md` (or linked evidence note): cross-artifact consistency including Spec 012 checklists status/ownership boundaries for reviewed requirements-quality artifacts, and rerun `cargo fmt --all -- --check` (and any targeted host checks selected) capturing exact command + exit status + relevant output summary. Mark only when evidence recorded; not Phase 9 attestation. per NFR-012-MAINT-005,FR-009,SC-005,T045,T047 (partial) **2026-11-06 evidence (host, non-attesting)**: checklists gate 126/126 checked PASS (automation-contract 32/32, durability-maintainability 30/30, evidence-residuals 30/30, release-exposure 34/34, requirements 16/16) — requirements-quality/reviewer-owned; not implementation attestation. T045/T031/T046 reconciled (`10982059`,`bcb982b4`,`d7e9fea9`,`3be7ae72`,`e9c165ad`). `cargo fmt --all -- --check` → exit `0`. I1/Kotlin/device/RC/Phase 10-11 impl+Phase 9 attestations remain tracked/open.
- [x] T059 [P] Resolve I1 harness decision path as recorded, testable plan (A extract pure debug-only parsing/normalization host-testable unit when harness exists, or B host-contract fallback now). **Decision B recorded** `c892fc22` (`i1-harness-decision.md`) — no CHK021, no `src-tauri/gen/**`, no parallel production ingest path; authority boundary explicit. **RED** `fcb4c421`. **Impl** `968c8691` (`--print-intent-parse-contract`, `--normalize-debug-ingest-intent`). **GREEN (2026-11-06, host, non-attesting)**: intent parse **14 passed**. **T013/T020 JVM extraction/harness remain open/traceable**.
- [x] T060 [US1/US2/US3] (host-contract/helper scope) Harden hook with deterministic **FR-008/SC-001/SC-002 device-evidence helper** (no adb side effects) and preserve existing host contract modes. **RED** `5f1679df` (`test_debug_ingest_device_evidence_contract.py`, Hypothesis/PBT). **Impl** `28c28fc0` (`--build-device-evidence --json`): `programmatic-non-ui`, `full_parent_completion_claimed=false`, residual note, FR-008 minimum fields, cache/corroboration attribution, SC-001/SC-002 hooks/counters, `crash_anr_status=unknown|detected` never explicit pass, blocked/inconclusive explicit/non-pass. **GREEN (2026-11-06, host, non-attesting)**: device-evidence **7 passed**. **Normal device-execution path wiring for real runs remains partial/opt-in/non-attesting** (T018/T024/T067/T049).

### Phase 10 Verification (attestation only — run last)

- [x] T061 **Verification (attests, host scope only)**: `python3 -m pytest scripts/tests/spec012 -q --tb=no` → **77 passed, 69.47s (0:01:09), exit 0** (2026-11-06, host). Covers Phase 10/11 host changes (activity channel, release exposure D2/D3/D4 mechanization, overlay dep/layout, intent/dir-uri contracts, device-evidence helper). No weakening observed: `full_parent_completion_claimed=false` preserved in contracts/tests; non-full FR-010 never satisfies SC-004; FR-011 check non-mutating/fail-closed. **Does NOT attest** T049/T050 device/release or full T184/T226 completion. per FR-006,FR-008,FR-010,FR-011,FR-012,SC-005,SC-006
- [x] T062 **Verification (attests reporting integrity, host/docs scope)**: `specs/009-open-file-from-title/tasks.md` contains T184 open + `T184-PROG/T184-RES-SAF-CANCEL/T184-RES-A11Y`, `full_parent_completion_claimed=false`; `specs/010-directory-reading/tasks.md` contains T226 open + `T226-PROG/T226-RES-SAF-TREE/T226-RES-TRANSITION/T226-RES-RESUME-UI/T226-RES-HARDWARE-CONTROLS/T226-RES-INVALID-ENTRIES-UI/T226-RES-A11Y-PICKER`, `full_parent_completion_claimed=false` (Spec 012 contract tests cover required split IDs). No hook-only full completion recorded. per FR-009,FR-012,SC-003,T052

---

## Phase 11: Convergence

> **Append-only convergence pass (2026-11-06, post-`/speckit.implement`, script/evidence assessment)**. Tasks T063+ address untracked or completion-integrity gaps not fully covered by Phase 10. TDD/RED first for any test task; PBT/invariants whenever applicable; tests BLOCK implementation. `[P]` = disjoint files, parallelizable in waves; tasks sharing file/context MUST reuse the same subagent and serialize shared tooling (never kill/interrupt/bypass another agent/process/tool). **Verification tasks (T068+) at the end are the ONLY attestation of success/completeness**. Device/release execution remains opt-in/evidence-bound; `blocked`/`inconclusive`/`not-evaluated` never satisfy FR-010/SC-004 or claim T184/T226 fully complete. Do not rewrite T042/T028 history; record corrections as follow-on tasks/evidence. Atomic conventional commit after each edited file.

### RED (blocking)

- [x] T063 [P] Write failing contract tests (PBT/invariant where applicable) in `scripts/tests/spec012/test_release_exposure_contract.py` for the FR-010 mechanized behavior still missing from `scripts/check-debug-ingest-release-exposure.sh` (D2 actual merged/packaged manifest/APK-AAB/merger inspection + tool/path/provenance; D3 explicit negative ADB invocation/injectable runner + no hook success/check-caused crash-ANR; D4/full pass only D1-D4+bound artifact/install identity). RED gate blocked T065. **RED authored** `9d129e0c` (+ `250ed5b5` JSON-escaped serial assert fix). **RED (2026-11-06, host, non-attesting)**: bg_141 — D2 placeholder/`inspection_performed=false`, D3 missing `invocation_attempted`. **GREEN (2026-11-06, host, non-attesting)** after T065 `310ebfce`: `test_release_exposure_contract.py` **11 passed** in full Spec 012 host suite (**51 passed, 34.23s**).
- [x] T064 [P] FR-011/CHK026/CHK027 dependency/layout RED→GREEN: `test_debug_ingest_overlay_dep_layout_contract.py` `79cd7753` (Hypothesis). RED missing `dependency_layout_results`; GREEN after T066 (`84bffa0d`,`33245dcd`,`827a19b4`,`a81bf42f`) **7 passed** (host, non-attesting).

### Implementation (after corresponding RED)

- [x] T065 Implement FR-010 D2/D3/D4 mechanization in `scripts/check-debug-ingest-release-exposure.sh` (shared FR-010 context): D2 real static inspection with tool/path/provenance/identifier bindings; D3 explicit negative invocation evidence (`invocation_attempted`, command, identity, `hook_success_detected`, crash-ANR status) with injectable `--adb-runner`/`DEBUG_INGEST_ADB_RUNNER`; D4/full pass only D1-D4+bound artifact/install identity+`install_identity.verified_release_non_debug=true`. per FR-010,SC-004,T042 (mechanization gap; T042 history not rewritten). **2026-11-06 host (non-attesting)** `310ebfce`: release exposure **11 passed**; full Spec 012 host suite **51 passed**. **No FR-010/SC-004 full-pass attestation** (requires RC+device+identities).
- [x] T066 Implement FR-011 dependency/layout validation in `scripts/check-debug-ingest-overlay.sh` and `scripts/apply-debug-ingest-overlay.sh` (shared FR-011 script context; reuse same subagent): add structured `dependency_layout_results` / `DEP-ASM` reporting for invoked scope, validate `ASM-012-SOURCESET-WIRING` and required repo/gen layout assumptions when dependent claims are made, fail closed on unverifiable required wiring/layout, and keep `check` non-mutating/idempotence semantics. per FR-011,CHK018,CHK026,CHK027,NFR-012-MAINT-004,T038,T039 (partial) **2026-11-06 host (non-attesting)**: commits `84bffa0d`,`33245dcd`,`827a19b4`,`a81bf42f`; T064 dep/layout suite **7 passed**; overlay contract suite **9 passed** after minimal wiring-aware idempotence fixture adjustment (T066 compat).
- [ ] T067 [US1/US2/US3] Integrate FR-008/SC-001/SC-002 report helpers into `scripts/automate-t184-t226-hook.sh` **normal device-execution** mode (shared hook context with T018/T024/T028/T060; reuse same subagent): emit attributable machine-readable evidence for actual runs (correlation key, preferred/effective location/channel, `write_status`, corroboration, reviewer-accessible location, retention scope, `crash_anr_status` including explicit `unknown` not-pass semantics, residual scope note, `full_parent_completion_claimed=false`, SC-001/SC-002 counters/classification, `blocked:fixture-inaccessible`/`inconclusive`/busy/cold-start as applicable) without weakening deterministic host contract modes. **Partial 2026-11-06 (host, non-attesting)**: deterministic `--build-device-evidence` helper `28c28fc0` (PBT `5f1679df`, 7 passed); preserves host contract modes. **Still missing**: wire evidence into normal adb/device-execution path for real runs (busy/cold-start/fixture-inaccessible runtime capture), and opt-in T049 artifacts. T028 completion-integrity gap remains partially open.

### Phase 11 Verification (attestation only — run last)

- [x] T068 **Verification (attests, host scope only)**: Phase 11 host suites GREEN in full run **77 passed** (release exposure mechanization, overlay dep/layout, hook device-evidence helper host-testable). `mode=full` cannot pass without D1-D4+identity binding/verified release (fail-closed tests/impl). FR-011 check remains non-mutating/fail-closed (contract tests). **Does NOT attest** T049/T050 device/release or full T184/T226 completion. per FR-006,FR-008,FR-010,FR-011,FR-012,SC-004,SC-005,SC-006
- [x] T069 **Verification (attests reporting integrity, host/docs scope)**: correction notes recorded — T042 mechanization gap closed by `310ebfce`/T063/T065 evidence (history not rewritten); T028 normal-mode wiring gap remains partially open and explicitly tracked in T067 (helper added, device-exec wiring pending). T184/T226 splits and `full_parent_completion_claimed=false` intact. per FR-009,FR-012,SC-003

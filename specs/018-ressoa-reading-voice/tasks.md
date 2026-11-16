# Tasks: 018-sona-inspired-voice

- **Feature**: 018-sona-inspired-voice
- **Spec**: `specs/018-sona-inspired-voice/spec.md`
- **Plan**: `specs/018-sona-inspired-voice/plan.md`
- **Assessment**: `.specify/assessments/sona-inspired-voice/decision.md` (GO)
- **Status**: Implemented & Converged (host gates 1-8 PASS, gate 9 Rust compile PASS)
- **Created**: 2026-11-10

## Conventions

- **[P]**: can run in parallel (different files, no dependency)
- **[US#]**: primary user story served (`US1` selection/persist, `US2` cycle/restart, `US3` synthesis contract, `US4` provenance/identity)
- Tests are included where required to preserve/extend existing contract and gate coverage for this repo (behavioral/regression focus, not wiring tautologies).
- Exact file paths are given; adjust only if discovery reveals additional required callsites (record in PR/commit notes).

## Phase 1: Setup & Catalog Groundwork

**Purpose**: lock bundled voice choice/provenance and asset layout before runtime refactor.

- [x] T001 [US4] Select and acquire curated alternative voice asset (target `pt_BR-cadu-medium` 22050 Hz CC0, fallback `pt_BR-jeff-medium`) and record auditory/technical validation notes and chosen default rationale in `specs/018-sona-inspired-voice/research.md` or feature notes (contingency: tuned-faber-only catalog documented).
- [x] T002 [P] [US4] Add bundled voice directory `crates/adapter-synthesis/assets/voices/pt_BR-cadu-medium/` (or chosen fallback) with `<id>.onnx`, `<id>.onnx.json`, `tokens.txt`, `MODEL_CARD`, and `PROVENANCE.md` including upstream source, HF revision/snapshot, CC0 license, file inventory, checksums, and inference defaults (no LoL/Riot/performer branding).
- [x] T003 [US4] Restructure assets to per-voice dirs plus shared phonemizer data: move/create `crates/adapter-synthesis/assets/voices/_shared/espeak-ng-data/` (or document/implement per-voice contingency) and update `crates/adapter-synthesis/assets/voices/pt_BR-faber-medium/PROVENANCE.md` only if paths/notes require correction (keep existing hashes/license intact).
- [x] T004 [US3][US4] Update `crates/adapter-synthesis/build.rs` to emit a deterministic sorted manifest covering the full bundled catalog layout (all voice dirs + shared/per-voice espeak data).

**Checkpoint**: bundled catalog assets and provenance are complete and build-manifested.

## Phase 2: Foundational — Adapter Synthesis Catalog & Runtime (Blocking)

**Purpose**: core closed catalog, voice-aware synthesis, extraction, and CLI APIs required by all downstream stories.

- [x] T005 [US1][US3] Create closed catalog/types module `crates/adapter-synthesis/src/voice.rs` (`VoiceId`, `VoiceSpec`/metadata, fixed-order `CATALOG`, `DEFAULT_VOICE`, parse/next/prev/list helpers) with exhaustive matches and no `Other/Unknown`; export from `crates/adapter-synthesis/src/lib.rs`.
- [x] T006 [US1][US2][US3] Refactor `crates/adapter-synthesis/src/synth.rs` to resolve per-voice paths from a generalized voices root, load per-voice VITS configs/defaults, and support voice-aware synthesis (`synthesize_with_voice`/active voice API) with compatible default `synthesize(segment)` delegation; validate engine lifecycle on `sherpa-onnx = "=1.13.8"` (cached per-voice or recreate-on-switch contingency) while preserving normalization, prosody, 22050 Hz fail-closed checks, and echoed metadata.
- [x] T007 [US3] Update `crates/adapter-synthesis/src/android_assets.rs` to extract/package the full catalog and shared espeak data idempotently with migration-safe marker/version handling; fail closed on missing/corrupt packaged assets.
- [x] T008 [US1][US3] Extend `crates/adapter-synthesis/src/cli.rs` with `voices` listing and `synthesize --voice <id>` (additive, deterministic JSON/stdout behavior), preserving existing fields/exit semantics.
- [x] T009 [US3] Keep `crates/adapter-synthesis/src/{lib.rs,reg.rs,capabilities.rs}` aligned: `OfflineDefault` only, declared capabilities `(SsmlLevel::None, AlignmentGranularity::None)` unchanged, and public API exports for catalog/selection as needed.

**Checkpoint**: adapter-synthesis can list/select/synthesize bundled voices offline with preserved audio contract.

## Phase 3: User Story 1 — Persisted Selection & Catalog-Aware Surfaces (Priority: P1)

**Goal**: selection persists/restores safely and is visible/selectable through session and bridge surfaces.

**Independent Test**: set voice, persist, reload store/startup seed, and verify default fallback for unknown/missing values plus catalog-aware diagnose/inventory.

- [x] T010 [US1] Add `LAST_VOICE_KEY` and `put_last_voice`/`get_last_voice` to `crates/session/src/store.rs` with safe fallback to `pt_BR-faber-medium` for missing/unknown/malformed values (no panic).
- [x] T011 [P] [US1] Add/extend session tests in `crates/session/tests/` (or inline module) for round-trip, default fallback, and unknown-id handling of persisted voice.
- [x] T012 [US1][US3] Update `src-tauri/src/lib.rs` startup/asset initialization to configure adapter voices root/catalog extraction and seed active voice from session store with default fallback.
- [x] T013 [US1][US4] Refactor `src-tauri/src/lib.rs` diagnose/voice inventory to be catalog-aware (active/default + per-voice asset presence/status for all bundled voices) replacing hardcoded `pt_BR-faber-medium` fields while keeping additive/compatible output where feasible.
- [x] T014 [US1] Add Tauri commands `get_voice`/`set_voice` (and helpers) in `src-tauri/src/lib.rs`/`src-tauri/src/commands.rs` (as appropriate), persist selection, publish updated state, and register in `invoke_handler`.

**Checkpoint**: selected voice is persisted, restored, diagnosable, and settable without playback corruption.

## Phase 4: User Story 2 — Deterministic Cycle & Safe Restart-on-Change (Priority: P1)

**Goal**: `CycleVoiceNext/Prev` traverses catalog deterministically and applies changes safely at synthesis boundaries.

**Independent Test**: cycle across catalog order/wrap and during active production; verify generation invalidation + restart from authoritative resume position and no mixed-voice PCM.

- [x] T015 [US2] Implement catalog-aware `CycleVoiceNext/Prev` handling in `src-tauri/src/lib.rs` (parse/live control path) to advance one step in fixed order, persist/publish selection, and apply safe production restart at authoritative position when active (invalidate generation, re-synthesize through same path).
- [x] T016 [US2][US3] Update `src-tauri/src/lib.rs` synthesis/production path to resolve active `VoiceId` for each production run and pass it to `adapter-synthesis`; ensure stopped/completed or ineligible cycle requests follow existing shell refusal semantics.
- [x] T017 [P] [US2] Update `crates/player/src/cli.rs` and `crates/player/tests/contract/test_shell_ipc_us4.rs` only as needed for additive/contract-correct cycle semantics and payload expectations (preserve existing transport/state rules).
- [x] T018 [P] [US2] Update `contracts/shell-ipc.md` and feature contract `specs/018-sona-inspired-voice/contracts/shell-ipc-voice.md` to document deterministic cycle/select/get behavior, restart-on-change semantics, and any additive fields.

**Checkpoint**: voice cycling is deterministic, contract-aligned, and safe under playback.

## Phase 5: User Story 3 — Audio Contract, Tests, and Script Alignment (Priority: P1)

**Goal**: every bundled voice satisfies synthesis invariants and host test/gate expectations are updated coherently.

**Independent Test**: targeted crate tests + workspace nextest/list diff + clippy/fmt on changed scope.

- [x] T019 [US3] Update/extend `crates/adapter-synthesis` tests (`crates/adapter-synthesis/tests/**`, inline tests) for catalog listing, per-voice synthesis contract (22050 Hz mono f32, echoed metadata), default delegation, unknown/missing asset fail-closed behavior, and CLI `voices`/`--voice` behavior.
- [x] T020 [P] [US3] Update `src-tauri` tests for catalog-aware diagnose/inventory, selection/cycle behavior, and restart semantics as applicable (including `src-tauri/tests/test_noscope.rs` voice-directory assertion to closed bundled catalog, and Android extraction assumptions if covered).
- [x] T021 [US3] Sweep and update remaining runtime/test/script hardcoded `pt_BR-faber-medium`-as-only-voice assumptions (excluding historical assessment/spec records) across `src-tauri`, `crates/adapter-synthesis`, `crates/player`, and `scripts/` as required for correctness.
- [x] T022 [US3] Refresh `scripts/expected-tests.txt` if `cargo nextest list --workspace` target set changes; keep diff clean (gate 2 two-way check).
- [x] T023 [US4] Record provenance/auditory evidence and final default-choice rationale in `specs/018-sona-inspired-voice/research.md` (or dedicated evidence note) and ensure shipped docs/labels remain non-imitative.

**Checkpoint**: behavioral coverage and gate inputs match implemented catalog behavior.

## Phase 6: Verification & Polish

**Purpose**: host gate compliance, documentation, and evidence.

- [x] T024 [US3] Run narrowest affected checks iteratively (`cargo fmt --all`, targeted `cargo nextest`/`cargo test -p adapter-synthesis -p session`, targeted `cargo clippy` as needed) and fix issues.
- [x] T025 [US3] Run host gates 1–8 / `./scripts/verify.sh` (fmt, nextest+expected diff, clippy -D warnings, doc, deny, audit, closed-set/shell checks) and capture results.
- [x] T026 [US4] Update relevant contracts/docs (`contracts/shell-ipc.md`, feature `contracts/*`, CLI/diagnose notes as applicable) for additive behavior and provenance; verify no `INTERNET`/LoL-branding regressions.
- [x] T027 Report final status: host gate outcomes, auditory/default rationale, contingencies exercised (if any), and gate 9/device as executed or unexecuted with reason.

## Dependencies & Execution Order

- **Phase 1**: T001 → T002/T003 → T004 (T002 can parallelize with layout planning but depends on selected asset).
- **Phase 2**: blocks Phases 3–5. T005 → T006; T007 depends on finalized layout (T003/T004); T008/T009 depend on T005/T006 APIs.
- **Phase 3 (US1)**: depends on Phase 2. T010 → T011/T012 → T013/T014.
- **Phase 4 (US2)**: depends on Phases 2–3 selection primitives. T015/T016 build on T014; T017/T018 can proceed in parallel after API/behavior settles.
- **Phase 5 (US3/US4)**: depends on implemented behavior (Phases 2–4). T019–T021 in parallel by crate where possible; T022 after test inventory stabilizes; T023 after T001 validation.
- **Phase 6**: depends on all implementation/tasks complete.

## Parallel Opportunities

- T002 and T003 (asset/provenance authoring) once voice chosen (T001).
- T011, T017, T018, T020 can run in parallel when their dependencies/APIs are stable.
- T019–T021 can be split by crate ownership (`adapter-synthesis`, `session`, `src-tauri`, `player/scripts`).

## Implementation Strategy

- **MVP increment**: complete Phases 1–3 with default-compatible behavior and one validated curated alternative; verify US1/US3 core contract before cycle polish.
- **Safe increment**: deliver US2 cycle/restart only after selection/persistence and production invalidation path are correct.
- **Contingency-safe completion**: if curated alternative fails validation/runtime, keep multi-voice-ready catalog/API + faber default and document rationale (still satisfies evolution plumbing and non-imitative constraints).
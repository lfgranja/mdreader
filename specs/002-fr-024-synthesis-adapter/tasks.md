# FR-024 — Synthesis Adapter Tasks

Branch: `002-fr-024-synthesis-adapter` | Spec: `spec.md`

Phases per plan.md (design-phase crate `crates/adapter-synthesis`): Setup | Tests | Core | Integration | Polish/Verify

## Rules applied (user input)
0. Template/scaffold first — `cargo new --lib` done; `Cargo.toml` edited via scaffold.
1. TDD — tests BLOCK core; see tasks T2–T4.
2. Implementation by DIFFERENT agent — T6–T8 reserved for `general` subagent (not test agent).
3. [P] tasks delegated to max viable agents/subagents/background.
4. Parallel tasks [P] executed in subagent waves; shared context reuses subagent.
5. Verification ONLY at end (V1–V3); NEVER self-attested by implementing agent.
6. Atomic conventional commit after each edit.

## Task list

### Setup [P]
- [X] T1 — Scaffold crate (cargo new; Cargo.toml dependencies; workspace member) [P] — done, committed

### Tests — BLOCKING; written first; property-based where applicable
- [X] T2 — prop_chunk.rs: chunk split ≤5000 bytes, paragraph/heading boundary (proptest) [P] — test agent (ses_ee3fba0cbffebWQe9agntGe2tn)
- [X] T3 — prop_declare.rs: declared_capabilities() invariant (None/None), no mutation (proptest) [P] — same test agent
- [X] T4 — prop_format.rs: format check mono f32 @ 22050 Hz; invalid → named EngineError (proptest) [P] — same test agent
- [X] T5 — unit tests: failure mapping (7 members, closed set); SC-051 mechanism; adapter registration [P] — same test agent

> Block: T6+ NOT started until T2–T5 complete.

### Core — DIFFERENT agent (not T2–T5 agent)
- [X] T6 — lib.rs: declared_capabilities(), AdapterCapabilities, SsmlLevel, AlignmentGranularity [P]
- [X] T7 — chunk.rs: split logic (paragraph/heading; byte-measured ≤5000) [P]
- [X] T8 — format_check.rs: mono f32 @ 22050 Hz verification against DeclaredFormat [P]
- [X] T9 — failure.rs: condition → EngineError 7-member mapping (spec.md:85; C4-A/C4-B) [P]
- [X] T10 — capabilities.rs: (None, None) declaration [P]

### Integration [P]
- [X] T11 — adapter registration in engine-core / offline_default tier
- [X] T12 — player interaction: declaration consumption (SsmlLevel/AlignmentGranularity branches)

### Verification — ONLY these attest success (rule 5)
- [ ] V1 — cargo test workspace (nextest); property tests pass
- [ ] V2 — cargo fmt / clippy -D warnings / cargo deny (constitution gates)
- [ ] V3 — checklists (adapter-interface.md, mechanism-gate.md, requirements.md, segment-resume.md, audio-safety.md) all [x]; byte-budget ≤5000 verified; SC-051 passes; no phantom trait methods

## Parallel waves
Wave 1 (setup): T1 — done
Wave 2 (tests): T2, T3, T4 — shared test agent (background)
Wave 3 (core): T6–T10 — different subagent from wave 2 (per rule 2)
Wave 4 (integration + verify): T11–T12, V1–V3 — verify subagent at end

## Phase 2: Convergence

- [X] T013 Add adapter registration mechanism (offline_default tier membership) per T11 / FR-024 / plan:Project Structure (missing)
- [X] T014 Implement player declaration-consumption branch (SsmlLevel/AlignmentGranularity) per T12 / contracts/adapter-capabilities.md §4 (missing)
- [X] T015 Complete paragraph/heading-aware chunk split + full byte-budget property integration per T2 / prop_chunk.rs (partial)
- [X] T016 Complete cargo nextest workspace run and confirm property tests pass per V1 (partial)
- [X] T017 Finalize core task verification and mark T6–T10 complete per tasks.md (partial)

## Phase 3: Convergence

> Appended per convergence command (rule 6: atomic conventional commit after edit; rule 1: T2–T5 tests BLOCK core; rule 2: core done by agent different from test writer; rule 3/4: [P] tasks delegated / parallel waves; rule 5: verification ONLY at end V1–V3 — these tasks attest, never the agent).

Ordered CRITICAL/HIGH first (F1–F4), then MEDIUM verification (F5–F7). Tests fixed/re-written first (T018–T020) blocking T021–T024 per rule 1.

- [X] T018 [P] Rewrite prop_format.rs to use `engine_core::closed::EngineError` 7-member closed set with correct condition→variant table (rate/width→RateMismatch, channel→ChannelMismatch, init/voice→InitFailed, process/OOM/kill→Unreachable, device refusal→DeviceRefused, device lost→DeviceLost, start refused→StartRefused) per spec.md:85 / C4-A / contracts/adapter-handoff.md §3.1 (contradicts) — test-agent write first, different from core-impl agent
- [X] T019 [P] Align prop_chunk + prop_chunk_refined with chunk.rs split logic; enforce ≤5000 bytes/paragraph-heading invariant via proptest (partial) — shared test subagent, parallel with T018 wave
- [X] T020 [P] Complete T2–T5 property/unit tests (prop_chunk done, prop_declare invariant, prop_format corrected, T5 failure mapping 7-member + SC-051 mechanism + adapter-registration property); do NOT start T021–T024 until T020 passes (missing / rule 1 block)
- [X] T021 [P] Finalize core verification (T6–T10): confirm `declared_capabilities()` unconditional; `format_check.rs` delegates to `engine_core::format::check_handoff`; `failure.rs` re-exports correct 7-member set; `reg.rs` records `OfflineDefault`; no phantom `format()`/`fail()`/`trait_` methods; `chunk.rs` split correct (partial — different agent from T018–T020 test writer per rule 2)
- [X] T022 [P] Execute `cargo nextest run --workspace`; confirm prop_chunk / prop_declare / prop_format all pass after F1/F2 fixes (partial / V1 — verification only, not self-attested)
- [X] T023 [P] Run `cargo fmt --check`; `cargo clippy --workspace -- -D warnings`; `cargo deny check advisories licenses bans sources`; verify adapter `Cargo.toml` declares `[lints] workspace = true`; no suppressions added (partial / V2 — verification only)
- [X] T024 [P] Confirm V3 checklists (adapter-interface.md, mechanism-gate.md, requirements.md, segment-resume.md, audio-safety.md) all verified [x]; byte-budget ≤5000 verified; SC-051 preserved continuously (`offline_default` registered, standins not selectable per `001/spec.md:89` / `closed.rs:305-312`); bad Markdown = document-layer only (no `diag` `parse` kind at v1); declare `AlignmentGranularity::None`; no `AdapterFailed`; document no new authority (partial / V3 — verification only; final attestation)

## Phase 4: Convergence

> Appended per convergence command. Constitution violations first, marked CRITICAL (F1–F3); then HIGH (F4–F8); then MEDIUM (F9–F13). Rule 0: scaffold/LSP before hand-writing. Rule 1: T033 is a NEW test written first and BLOCKS T034 (rule 2: T034 done by an agent different from the T033 writer). Rule 3/4: `[P]` tasks delegated to the maximum viable number of agents, run in waves. Rule 5: only T037–T039 attest success, never an implementing agent. Rule 6: atomic conventional commit after each edit.
>
> Evidence base (all observed on this host): `cargo nextest run -p adapter-synthesis` → `missing documentation for a function` ×2 (`src/reg.rs:4`, `src/consume.rs:4`) + unused imports (`src/reg.rs:2`); `cargo fmt --all -- --check` fails in `consume.rs`, `lib.rs`, `reg.rs`, `tests/prop_chunk.rs`; `tests/prop_format.rs`, `prop_chunk.rs`, `prop_chunk_refined.rs`, `unit_failure.rs`, `unit_consume.rs` all import a frozen surface (`API_CONTRACT.md` §1–§8) that does not exist in the crate.

### Wave A — CRITICAL constitution violations (F1–F3)

- [X] T025 **CRITICAL** Restore compilability of `crates/adapter-synthesis` under the workspace lint set: add `///` docs to every public item in `src/reg.rs` and `src/consume.rs`, delete the unused `AdapterCapabilities, SsmlLevel, AlignmentGranularity` import at `src/reg.rs:2`, and `cargo fmt` the crate — minimal edit only, no behaviour change, so the frozen T018–T020 tests are observed RED on the missing contract rather than on a build error per Constitution §Rust Code Rigor (contradicts / F1)
- [X] T026 **CRITICAL** Rewrite `src/consume.rs` to the frozen `API_CONTRACT.md` §5 surface — `SsmlDelivery`/`ResumeAnchor`/`PlayerPlan` + `ssml_delivery()`/`resume_anchor()`/`plan_for()` — deleting the `_ => "unsupported"` wildcard and the `&'static str` return so every declaration pair maps explicitly and a declared `None` never upgrades to `FullSsml`/`Sentence` per Constitution gate 7 (`constitution.md:614`) / Principle VI (contradicts / F2 + F8)
- [X] T027 **CRITICAL** Rewrite `src/synth.rs` to `synthesize(&DocumentSegment) -> Result<Vec<f32>, EngineFailure>` that FAILS CLOSED — the unbound Piper `faber` model is reported as `FailureCondition::VoiceInitFailure` → `EngineError::InitFailed`, with `session_position` echoed from the segment `resume_point` and an offer of `MAX_OFFER_MS`; the current `Ok(Vec::new())` silent success must not survive per FR-024b / SC-035 (contradicts / F3)

### Wave B — HIGH frozen-contract surfaces (F4–F8)

- [X] T028 [P] Implement `src/chunk.rs` to `API_CONTRACT.md` §2 + §2.1: `MAX_CHUNK_BYTES = 5_000`, `SourceRange` (`new`/`whole_lines`/`line_count`/`is_empty`/`contains_line`/`Display`), `BoundaryKind` (`Paragraph|Heading`, no `Sentence`), `DocumentSegment` (`content`/`source_range`/`boundary_kind`/`segment_index`/`resume_point` + derived `byte_length`), `split_document`, `split_document_within`; enforce all 12 normative rules — byte-budget hard bound, oversized-block char-safe hard split, trimming, losslessness, determinism, zero segments for empty input, 0-based consecutive index, 1-based non-overlapping ranges, `resume_point == Some(source_range)`, hash accepted but never stored per FR-024a / SC-027 / CHK008+CHK016 (missing / F4)
- [X] T029 [P] Implement `src/failure.rs` to `API_CONTRACT.md` §3: `MAX_OFFER_MS = 3_000`, `MAX_CONDITION_DETAIL_CHARS = 200`, the 10-member `FailureCondition` with `ALL`/`variant()`/`detail()`, `EngineFailure` (6 fields), `FailureReportError` (`DetailTooLong`/`OfferTooLong`) and a fail-closed `EngineFailure::report()`; re-export the 7-member `EngineError` from `engine_core::closed` and add no variant, per FR-035 / `spec.md:86` condition→variant table / FR-023a / FR-041 (missing / F5)
- [X] T030 [P] Complete `src/format_check.rs` with `DECLARED_RATE_HZ`/`DECLARED_CHANNELS`/`DECLARED_WIDTH_BITS` re-exported from `DeclaredFormat` and `check_delivery(rate_hz, channels, width_bits, pcm_sample_count, duration_ms)` checking shape FIRST then length against `(duration_ms/1000)*22050` within a half sample, deviation → `RateMismatch`, per FR-024b / SC-035 / R1+R2 / `API_CONTRACT.md` §6 (missing / F6)
- [X] T031 [P] Rewrite `src/reg.rs` to `API_CONTRACT.md` §4: `REGISTERED_TIERS: [EngineTier; 1] = [EngineTier::OfflineDefault]`, `registered_tiers()`, and `is_selectable(tier)` with five DISTINCT arms — `TestDouble` is false because the stand-ins are fixtures while a non-`test_double` adapter is registered; delete the `tier_membership() -> &'static str` literal, since re-declaring a tier name as `&str` is a second list that drifts per SC-051 / R11 / `001/spec.md:89` / FR-044 (missing / F7)

### Wave C — MEDIUM contract and intent alignment (F9–F12)

- [X] T032 [P] Correct the dependency block: raise `pulldown-cmark` from `"0.10"` to the 0.13.x required by `plan.md:23` and Constitution `:408`, then for each of `sherpa-onnx`, `ort`, `pulldown-cmark` record whether it is consumed now or deferred to `003-contract-edit` (`API_CONTRACT.md` §7/§10) — remove any declaration nothing imports so `ort-sys`/`sherpa-onnx-sys` stop compiling for absent code; keep `[lints] workspace = true`, per Constitution §Supply Chain (contradicts / F9)
- [X] T033 [P] **TEST-FIRST — BLOCKS T034** Add a segmentation test file covering the Markdown constructs `pulldown-cmark` actually classifies and the frozen generators omit: setext headings (`===`/`---` underline), `#` lines inside fenced code blocks, list items and block quotes — written by a test agent, binding only to the frozen `API_CONTRACT.md` §2 surface, no local re-declaration of any contract type, proptest where applicable; MUST be written and observed failing before T034 starts per FR-024a / rule 1 (missing / F10)
- [X] T034 [P] Make the split parser-side per `spec.md:94-95`, OR record the deviation explicitly in `API_CONTRACT.md` §2.1 with the misclassification cases named — the current line scan recognises ATX `#` only, so a setext heading, a `#` line inside a fence, a list and a block quote are all mislabelled `Paragraph`; implementation agent MUST differ from the T033 writer per rule 2 (partial / F10, blocked by T033)
- [X] T035 [P] Complete `src/lib.rs` with the required crate-root re-exports — `chunk::{BoundaryKind, DocumentSegment, SourceRange, MAX_CHUNK_BYTES}`, `consume::{PlayerPlan, ResumeAnchor, SsmlDelivery}`, `failure::{EngineError, EngineFailure, FailureCondition, FailureReportError, MAX_CONDITION_DETAIL_CHARS, MAX_OFFER_MS}` — and mark `capabilities::capabilities()` `#[must_use]`, so one declaration is expressed once with no second authority per `API_CONTRACT.md` §1/§8 / FR-044 (missing / F11)
- [ ] T036 [P] Re-scope the V1/V2 gates to the adapter crate and RECORD, without silently skipping, the pre-existing workspace blockers observed on this host: `alsa-sys v0.4.0` build failure (no `libasound2-dev`) blocks `cargo nextest run --workspace` via `crates/player`; `[workspace.lints.clippy]` reports `lint group nursery/cargo has the same priority (0)`; `engine-core` and `doc-structure` each fail clippy with pre-existing errors; `cargo fmt --all -- --check` also fails in `crates/player` and `crates/session`. None of these are owned by 002 per V1/V2 (partial / F12)

### Wave D — verification ONLY (rule 5; T037–T039 attest, no implementing agent may self-certify)

- [ ] T037 [P] **V1'** Run `cargo nextest run -p adapter-synthesis -p engine-core`; confirm `prop_chunk`, `prop_chunk_refined`, `prop_declare`, `prop_format`, `unit_failure`, `unit_consume` all GREEN, including the byte-budget property that Constitution `:353-354` requires a test to enforce; report the T036 blocker verbatim rather than passing around it (partial / V1 scoped)
- [ ] T038 [P] **V2'** Run `cargo fmt --check -p adapter-synthesis`; `cargo clippy -p adapter-synthesis --all-targets -- -D warnings`; `cargo deny check advisories licenses bans sources`; confirm zero `_ =>` in `crates/adapter-synthesis/src/`, zero `#[allow]`/`#[expect]` suppressions, no `unwrap`/`expect`/`panic!`/`todo!`/`dbg!`/`println!` even in tests, `[lints] workspace = true` present, and `crates/engine-core` unmodified per `API_CONTRACT.md` §0 rule 4 (partial / V2 scoped)
- [ ] T039 [P] **V3' — final attestation** Confirm and re-attest, rather than carry as evidence, the 148 checklist items currently marked `[x]` across `adapter-interface.md`, `mechanism-gate.md`, `requirements.md`, `segment-resume.md`, `audio-safety.md` — every mark must be re-witnessed against a green T037/T038 run, since none of them is exercised by a crate that does not compile. Then verify: byte-budget ≤5000 enforced by a passing test; `SC-051` preserved as a selector invariant (`offline_default` sole selectable, `test_double` stand-ins not selectable per `001/spec.md:89` / `closed.rs:305-312`); no `AdapterFailed` and no second `EngineError` list; bad Markdown stays document-layer with no `diag` schema expansion; `AlignmentGranularity::None` declared; `SC-044` still `DEFERRED` per `constitution.md:643`; no second authority over playback (contradicts / F13; rule 5 — the sole attestation)

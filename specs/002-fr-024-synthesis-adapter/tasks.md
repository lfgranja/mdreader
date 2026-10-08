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
- [ ] T2 — prop_chunk.rs: chunk split ≤5000 bytes, paragraph/heading boundary (proptest) [P] — test agent (ses_ee3fba0cbffebWQe9agntGe2tn)
- [ ] T3 — prop_declare.rs: declared_capabilities() invariant (None/None), no mutation (proptest) [P] — same test agent
- [ ] T4 — prop_format.rs: format check mono f32 @ 22050 Hz; invalid → named EngineError (proptest) [P] — same test agent
- [ ] T5 — unit tests: failure mapping (7 members, closed set); SC-051 mechanism; adapter registration [P] — same test agent

> Block: T6+ NOT started until T2–T5 complete.

### Core — DIFFERENT agent (not T2–T5 agent)
- [ ] T6 — lib.rs: declared_capabilities(), AdapterCapabilities, SsmlLevel, AlignmentGranularity [P]
- [ ] T7 — chunk.rs: split logic (paragraph/heading; byte-measured ≤5000) [P]
- [ ] T8 — format_check.rs: mono f32 @ 22050 Hz verification against DeclaredFormat [P]
- [ ] T9 — failure.rs: condition → EngineError 7-member mapping (spec.md:85; C4-A/C4-B) [P]
- [ ] T10 — capabilities.rs: (None, None) declaration [P]

### Integration [P]
- [ ] T11 — adapter registration in engine-core / offline_default tier
- [ ] T12 — player interaction: declaration consumption (SsmlLevel/AlignmentGranularity branches)

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

- [ ] T013 Add adapter registration mechanism (offline_default tier membership) per T11 / FR-024 / plan:Project Structure (missing)
- [ ] T014 Implement player declaration-consumption branch (SsmlLevel/AlignmentGranularity) per T12 / contracts/adapter-capabilities.md §4 (missing)
- [ ] T015 Complete paragraph/heading-aware chunk split + full byte-budget property integration per T2 / prop_chunk.rs (partial)
- [ ] T016 Complete cargo nextest workspace run and confirm property tests pass per V1 (partial)
- [ ] T017 Finalize core task verification and mark T6–T10 complete per tasks.md (partial)

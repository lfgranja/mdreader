# Tasks: Adotar o DESIGN-SONA (Option A subset)

- [X] T001 Create feature branch: `git switch dev && git switch -c 005-adopt-design-sona` (branch tracks this feature per plan.md); abort on dirty tree (`git status --porcelain` must be empty). Record branch point SHA in T003 STATE note.

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/.

**Tests**: YES — TDD is binding per Constitution Principle III and user rule 1. Every user-story wave writes its tests FIRST and every test must be shown FAILING (RED) before that story's implementation tasks may start. Property-based testing (`proptest`) is mandated wherever a randomization oracle applies.

**Organization**: Tasks grouped by user story (US1–US4). Each story phase = Tests (BLOCKING) → Implementation → Checkpoint.

**Format**: `- [ ] [ID] [P?] [Story] Description with file path`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks). Delegated to subagent waves; tasks sharing context reuse the same subagent.
- **[Story]**: US1–US4. Setup/Foundational/Polish/Verification phases carry NO story label.
- Every task includes an exact file path. No vague tasks.
- **BLOCKING**: Tasks marked BLOCKING gate all later tasks in their scope until complete.
- **NEVER**: No agent attests success. ONLY the final Phase verification tasks attest completeness.
- **Deterministic hook**: After finishing editing ANY file, make an atomic conventional commit with a detailed summary (`scripts/commit-hook.sh <file> "<summary>"`).
- **Scaffold-first**: Generate mechanical skeleton via LSP code actions BEFORE writing by hand (enum variants + derive blocks, test-module signatures, proptest strategy stubs, serde blocks).
- **Shared tooling**: serialized — one agent at a time; never kill/interrupt another agent's tool.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Feature branch, green baseline, prerequisite anchors — nothing else may start until this phase commits clean.

- [ ] T001 Create feature branch: `git switch dev && git switch -c 005-adopt-design-sona` (branch tracks this feature per plan.md); abort on dirty tree (`git status --porcelain` must be empty). Record branch point SHA in T003 STATE note.
- [X] T002 Baseline gate evidence: run `scripts/verify.sh` (gates 1–8) on the untouched tree; it MUST exit 0. Record the run in the T003 STATE note. If baseline is red, STOP the feature.
- [X] T003 Prerequisite anchor audit + memory update: verify all anchors exist at stated anchors (fail-closed on miss): `crates/player/src/backend.rs` `enqueue_earcon` (line :601, FIFO `push_short_mix` :327), `crates/player/src/earcon.rs` `EarconKind` (7 variants), `crates/diag/src/writer.rs` `EventKind` (exactly eleven kinds :70-93), `src-tauri/src/status_view.html` (tokens :21-31, reduced-motion guard :275, indeterminate infinite :535, @keyframes indeterminate :918), `src-tauri/src/status_view.rs`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` constants block `:728-743` (incl. `DESIGN_SONA_PROGRESS_FILL = 0xFFF2F0E6`), `src-tauri/tauri.conf.json` `"frontendDist": "../dist"`, `scripts/verify.sh`, `scripts/expected-tests.txt`, `DESIGN-SONA/`.

**Checkpoint**: branch created, baseline green, all anchors verified — wave execution may begin.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The surface-audit twin infrastructure every story's tests share, and the S2 packaging wiring (Assumptions #9 + contract S2: «the wiring is this feature's first implementation task»).

**⚠️ CRITICAL**: No user-story implementation may begin until this phase is complete.

- [X] T004 Create the audit scaffolding `src-tauri/src/surface_audit.rs` (new module, `pub mod surface_audit;` in `src-tauri/src/lib.rs` — scaffold generated first via rust-analyzer then filled): (a) closed auditee resolver — the exact 6-entry list of `contracts/surface-audit.md` A1 (1 `src-tauri/src/status_view.html`; 2 `src-tauri/src/status_view.rs` relevant constants; 3 `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` slice `:728-743`; 4 `DESIGN-SONA/{tokens.md, components.md, DESIGN-SONA.md, preview.html}` (CHANGELOG.md and VERSION are created in T028 — added to auditee list only after T028); 5 `src-tauri/tauri.conf.json`; 6 `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`), paths resolved from `CARGO_MANIFEST_DIR/..`; a missing auditee returns a named error — fail-closed, never skip; (b) a FIXED parser — byte-scanner producing token-color occurrences per auditee with file+line for every hit.
- [X] T005 [RED] [P] C-H8 served-identity twin: fill `src-tauri/src/surface_audit/served_identity.rs` (T004's empty scaffold) with test `surface_audit_served_identity_binds_status_view_as_bundle` — parse `src-tauri/tauri.conf.json` `frontendDist`, read the served bundle entry (`<frontendDist>/index.html`), assert **byte-identity** with `src-tauri/src/status_view.html` (contract S2: «MUST BE byte-identical… at acceptance»); a missing bundle or missing auditee = NAMED failure (fail-closed; A2: check id + file + line estimate). MUST FAIL NOW with the named reason «served bundle is the dist/index.html bench, byte-differs from src-tauri/src/status_view.html» — this RED BLOCKS T006.
- [X] T006 Implement S2 packaging wiring: make the packaged app actually serve `src-tauri/src/status_view.html` as the bundle entry — build-time copy `status_view.html` → `dist/index.html` via a NEW `src-tauri/build.rs` step (copy file at build time using `std::fs::copy` in the build script, with `cargo:rerun-if-changed=src/status_view.html`); `src-tauri/tauri.conf.json` `frontendDist` stays `../dist` (byte-identity asserted by T005). «The bench `dist/index.html` content may remain as a fallback only if it does not leave the audited surface unserved» — it leaves it unserved as the default route, so the bench page is REPLACED; with it dies the numeric-rate bench leak «Taxa: 1.0×» (data-model §SurfaceInventory dead-removal list). Update `src-tauri/src/status_view.rs` hooks accordingly if needed. Commit via hook per file.
- [X] T007 [RED] [P] C-H1 closed-hex census twin: `surface_audit_chroma_census` in `src-tauri/src/surface_audit/chroma.rs` (T004's empty scaffold): walk the closed auditee list (T004 resolver); every hex token in every auditee must be a member of the audited set (10 members; harp value resolved via ledger constant), audited over BOTH progress render branches of `status_view.html` (determinate fill branch AND the `indeterminate` pill/JS branch — no third audited state). Violation classes fail with check id + file + line: (i) any hex outside the set; (ii) jade `#5AD6A8`/erratum `#5FD0A0` paint; (iii) celeste `#B9A6F5` paint (vacancy — §SonaToken: «zero paint in audited files»). MUST FAIL TODAY — RED blocking T007.
- [X] T008 [RED] [P] C-H2 role-exclusivity twin: `surface_audit_role_exclusivity` in `src-tauri/src/surface_audit/exclusivity.rs`: `#8FD8E8` occurs ONLY inside `:focus-visible` rules (TalkBack draws its own ring — E1; today ZERO `:focus-visible` exists); `#FFD400` paints ONLY {reason banner warning state, in-app 3 s caution dot}; `#FF4D63` paints ONLY permanent-failure; caution×failure never co-render in the same declared state (mutual contrast 2.25:1 — C-H2 adjacency clause); any off-role paint = the D2 violation class. MUST FAIL TODAY — RED blocking T008.
- [X] T009 [RED] [P] C-H6 movement census twin: `surface_audit_movement_census` in `src-tauri/src/surface_audit/movement.rs`: parse every `@keyframes`/`animation`/`transition` declaration in `status_view.html` (fixed parser) — ≤1 continuous loop, exactly the sliding pill; a `@media (prefers-reduced-motion: reduce)` guard EXISTS and freezes the pill at a VISIBLE on-track position (`animation-play-state: paused` + static offset — an invisible/opacity-zero freeze FAILS, contract S5); the yellow pulse carries `animation-iteration-count` bounded by the 3 s TTL (E2: strictly inside the TTL); NO pulse on the failure color; banner `slideIn` one-shot ≤200 ms (no looping slide); any other loop fails. MUST FAIL TODAY (`indeterminate 1.5s infinite` at status_view.html) — RED blocking T009.
- [X] T010 [RED] [P] C-H7 contrast solver twin: `surface_audit_contrast_solver` in `src-tauri/src/surface_audit/contrast.rs`: implement the WCAG 2.1 AA contrast calculator as a PURE function (relative luminance + ratio, parameterized by effective background — void `#0B0F1A` / stage `#141B2D` — so future light modes reuse it; `research.md` R12) with a unit table (sanity margins: fill 16.75/15.01 over void/stage, fill×track 4.88, `--sona-boundary` 3.43/3.07 narrow non-text pass; the `--sona-aria` declared-drift case 11.20→10.59 documented), then assert every painted pair parsed from `status_view.html` meets text ≥4.5:1 / non-text ≥3:1. Declared design-dir numbers are documentation, NOT authority — the computation is (`research.md` R12). A failing pair = violation. MUST FAIL TODAY — RED blocking T010.
- [X] T011 [RED] [P] C-H5 aria-inventory twin: `surface_audit_aria_inventory` in `src-tauri/src/surface_audit/aria.rs`: over `status_view.html` — every enumerated interactive control carries a non-empty `aria-label` + state attribute; `#progress` carries `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; a visually-hidden `role="status"` live region exists (the ladder value channel; the `#speedDots` strip stays `aria-hidden`); `#reasonBanner` is typed `role="status"` and the legacy `role="alert"`+`aria-live="polite"` pair (`:252`) is ABSENT; NO painted numeric rate anywhere (painted-numeral scan on `Taxa:`/`×` patterns fails — the value-blind invariant S4). MUST FAIL TODAY (no aria-valuetext in the file; alert+polite pair present) — RED blocking T011.

**Checkpoint**: Foundation ready — audit scaffolding + ledger consumer exist; served-identity twin RED. User-story RED waves below may now start.

---

## Phase 3: User Story 1 — Identidade visual/sonora da notificação (Priority: P1) 🎯 MVP

**Goal**: the MediaStyle notification keeps ONLY the platform's single accent channel set to `#F2F0E6` (closed member `--sona-voice`) — título + seção atual + progresso only, no body text/rate/voice/mark; warning = static reason line (TTL 3 s) + Ostinato earcon; the pulsing dot lives in-app, never in the notification (D2 erratum recorded in the dir by T011).

**Independent Test**: `surface_audit_accent_only_constants` green — then the device inspection of quickstart §3.1 (10/10 sheets; device leg lives in Phase 8).

### Tests for User Story 1 (write FIRST; must FAIL before T013) ⚠️

- [X] T012 [P] [US1] GREEN verification (not RED): `surface_audit_accent_only_constants` in `src-tauri/src/surface_audit/accent.rs` — confirm the MediaStyle block in `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (`:728-743`) already paints EXACTLY `0xFFF2F0E6` (`DESIGN_SONA_PROGRESS_FILL` at line 730, already updated — not `0xFFFFFFFF` as originally estimated). Verify structural invariants: no mark/logo constant, no rate literal, no body-text constant (FR-002/FR-028 — `setContentText` non-null guard stays). The test should already PASS; if it fails, identify the remaining gap. (Originally written as RED, but state is already green — reconciliation per analysis U-09.)

### Implementation for User Story 1

- [X] T013 [US1] Verify `MediaMirror.kt` constants block `:728-743` already updated: `DESIGN_SONA_PROGRESS_FILL` already `0xFFF2F0E6` at line 730 (not `0xFFFFFFFF`). Confirm KDoc retagged to canonical token names (`--sona-voice`, `--signal-caution`, `--signal-failure`). NO other Kotlin change — declarative mirror stays (SC-035), no manifest/permission change (D4 exclusion intact). GREEN T012. Commit via hook.<|end|><|start|>edit to=assistant<|channel|>analysis<|message|>[specs/005-adopt-design-sona/tasks.md#9869]
### Implementation for User Story 1

- [X] T013 [US1] Verify `MediaMirror.kt` constants block `:728-743` already updated: `DESIGN_SONA_PROGRESS_FILL` already `0xFFF2F0E6` at line 730 (not `0xFFFFFFFF`). Confirm KDoc retagged to canonical token names (`--sona-voice`, `--signal-caution`, `--signal-failure`). NO other Kotlin change — declarative mirror stays (SC-035), no manifest/permission change (D4 exclusion intact). GREEN T012. Commit via hook.

**Checkpoint**: US1 host leg green — the MediaStyle block paints exactly `#F2F0E6`; no off-set paint. Device inspection (SC-003) belongs to Phase 8 (device-build owner).

---


## Phase 4: User Story 2 — Confirmar controles por earcon (Priority: P1) 🎯 MVP

**Goal**: eyes-free confirmation for all 7 confirmation types (play→Adagio · pause→Andante · rate-step→Unison · skip-fwd→Allegro-frente · skip-back→Allegro-trás · warning→Ostinato · failure→Sforzando) under the SHIPPED 004 grammar (pitches never re-specified); burst = truncate/substitute «latest wins»; system earcons interrupt; emission failure → action proceeds + AT MOST one re-emission + typed diag event; ≤100 ms first-consumed-sample.

**Independent Test**: `prop_earcon_burst_latest_wins` + `test_earcon_latency_oracle` + diag twins green — then quickstart §3.3 blind-test protocol (device leg, Phase 8).

### Tests for User Story 2 (write FIRST; must FAIL before T017/T018) ⚠️

- [X] T014 [P] [US2] RED burst/overlap property tests (property-based, `proptest`): add the `#[cfg(test)] mod earcon_burst` to `crates/player/src/backend.rs` with `prop_earcon_burst_latest_wins` — for random interleaved action sequences (bursts of 2–3 heterogeneous confirmations inside 100–150 ms; ~10 000 cases) assert the invariants: (1) the single `short_mix` channel is never widened (no second channel, no pre-mix chain — contract E2.5); (2) an in-flight USER earcon is TRUNCATED — its tail dropped BEFORE the append — so the newest action's first sample is the next consumed one («latest wins», E2.2); (3) SYSTEM earcons (Ostinato/Sforzando, from `EarconKind::{Warning,Failure}`) interrupt any in-flight user confirmation and fire immediately (E2.3); (4) the additive mix remains earcon↔narração ONLY (per-slot `mul_add`), never earcon↔earcon.
- [X] T015 [P] [US2] RED latency oracle `[US2]` test `test_earcon_latency_oracle` in `crates/player/src/backend.rs` tests — reuse 004's T152 first-consumed-sample technique (deterministic host gate; NEVER trial-only): the first consumed short-mix sample for EVERY trigger (all 7 confirmations) arrives ≤100 ms from its trigger (system Ostinato/Sforzando measured from the state-change shot), in isolation AND under the burst patterns of T014 (FR-010; D3: «the bound survives rajadas»). MUST FAIL under bursts TODAY (FIFO tail delays >100 ms) — RED blocking T018. Remediation B3 (2026-10-26): baseline `sem oráculo` removed; host oracle = authority; device trial secondary only.
- [X] T016 [P] [US2] RED diag kind twins: in `crates/diag/src/writer.rs` extend `contract_schema` tests: (a) the closed `EventKind` set becomes EXACTLY ELEVEN — adding `EarconTruncation` (truncation counter) and `EarconEmissionFailure` (typed emission-failure event) per `data-model.md` §DiagnosticEvent — assert serde snake_case round-trip for all 11 and that NO `Other`/`Unknown` arm exists; (b) negative serialization: an unknown kind must fail deserialization (never map); in `crates/player` add `test_earcon_truncation_diag`: every truncation of an in-flight user earcon registers exactly ONE `EarconTruncation` entry; the emission-failure path (synthesis error / short channel unavailable) registers the typed event carrying the required `error` field AFTER the single re-emission (never silenc…).

### Implementation for User Story 2

- [X] T017 [US2] Diag §VII coordinated extension: add `EarconTruncation` + `EarconEmissionFailure` variants to `EventKind` in `crates/diag/src/writer.rs` — scaffold-first (rust-analyzer «add variant», keeping the derive + enum style verbatim), then hand-fill doc comments citing `data-model.md` §DiagnosticEvent + `contracts/earcon-mapping.md` E3/E4. Closed-set discipline (§VII): the SAME task carries the coordinated edits — `crates/diag/src/scan.rs` scanner arms (explicit, no `_` arm, no `Other`), `contract_schema` text updated from «exactly nine» to «exactly eleven», the diag crate semver bump in `crates/diag/Cargo.toml`, and the docs line in `docs/diagnostic-record.md`. GREEN T016. Commit via hook per file.
- [X] T018 [US2] Implement burst policy + emission failure in `crates/player/src/backend.rs`: (1) classify incoming earcons User vs System by `EarconKind` (Play/Pause/SkipFwd/SkipBack/RateStep = user; Warning/Failure = system — the mapping of `contracts/earcon-mapping.md` E1; `EarconKind` stays unchanged, pitches never re-specified); (2) `enqueue_earcon` truncates: drop the tail of an in-flight USER earcon BEFORE appending the new one (latest-wins); a SYSTEM earcon takes the channel head immediately (interrupt); (3) synthesis error / short-channel unavailable → the transport action proceeds, narration untouched, AT MOST ONE re-emission at the next render space, then ONE typed `EarconEmissionFailure` diag event (`error` field set) — replacing the current silen… Commit via hook.

**Checkpoint**: US2 host leg green — burst policy, ≤100 ms oracle, eleven-kind diag record with typed counters. Device blind-test legs belong to Phase 8.

---

## Phase 5: User Story 3 — Superfície in-app com tokens do DESIGN-SONA (Priority: P2)

**Goal**: the StatusView token baseline — marfim `#F2F0E6` título + progress fill (flat, both states, no gradient) over trilho `--sona-boundary #55688F`, periwinkle `--sona-hush` seção, indeterminate = solid sliding pill (004's shimmer REMOVED — C5), reduced-motion freeze (E2), closed palette with role exclusivity (D2/E1), aria semantics inventory (D5).

### Implementation for User Story 3

- [X] T024 [US3] Verify token restyle already applied in `src-tauri/src/status_view.html` (EXCLUSIVE file owner — same subagent as T025/T027; no parallel edits to this file): (1) confirm closed 10-token palette already declared at lines 21-31 (`--sona-void #0B0F1A`, `--sona-stage #141B2D`, `--sona-voice #F2F0E6`, `--sona-etwahl #8FD8E8`, `--sona-hush #A9B8CE`, `--sona-harp` (= ledger), `--sona-celeste #B9A6F5` (declared only — ZERO paint), `--sona-boundary #55688F`, `--signal-caution #FFD400`, `--signal-failure #FF4D63`); confirm `--warning`/`--failure` renamed to `--signal-caution`/`--signal-failure`; (2) confirm any remaining `rgba()` tints removed — warning/failure paint roles follow sanctioned set. Tests already declared in Phase 2 (T007/T007-equivalent). GREEN. Commit via hook.
- [X] T025 [US3] Verify aria semantics already applied in `src-tauri/src/status_view.html` (SC-001 item (iv), SC-007 foundation; data-model §SurfaceInventory Live-region row; D5): confirm `role="progressbar"` + `aria-valuenow`/`aria-valuetext` (pt-BR valuetext) on `#progress`; confirm visually-hidden live region `role="status"` announcing ladder value («velocidade 1,25») and reason/failure states; confirm `#speedDots` stays `aria-hidden`; confirm `#reasonBanner` typed `role="status"` (legacy `role="alert"`+`aria-live="polite"` pair removed); every enumerated interactive control (`#btnSkipBack`, `#btnPlayPause`, `#btnStop`, `#btnSkipForward`, `#btnRateDown`, `#btnRateUp`, offer-dialog buttons) carries non-empty `aria-label` + state attribute. Tests already declared in Phase 2 (T011/T011-equivalent). GREEN. Commit via hook.

**Checkpoint**: US3 host leg green — closed palette + exclusivity + movement contract + contrast solver + aria channel on the restyled source of record.

---

## Phase 6: User Story 4 — Lente Sinestésica reduzida (Priority: P3)

**Goal**: the reduced-Lente stamps on the SAME StatusView — Concha Harmônica "R" header (C4 variant: no jade, no celeste), `--sona-stage` metadata card (C2/batch 96), structural dead removals (B4), CLOSED element inventory (exactly the §SurfaceInventory 9 entries), no path to the excluded screens (S1), header-asset census-clean.

**Independent Test**: `surface_audit_element_inventory` green (incl. excluded-screen unreachability + asset census) — then quickstart §3.6 (Phase 8).

### Tests for User Story 4 (write FIRST; must FAIL before T027) ⚠️

- [X] T026 [P] [US4] RED element-inventory twin (C-H4 + SC-006 host half) test `surface_audit_element_inventory` in `src-tauri/src/surface_audit/inventory.rs` (T004's empty scaffold — its file is disjoint from the Wave-3 twins, so it may run in Wave 3 or 4; it must be RED before T027): over `status_view.html` (fixed parser, order-preserving): the element inventory is EXACTLY the closed §SurfaceInventory enumeration — Header mark, Metadata card, Progress, Transport controls, Rate ladder, Reason line, Resume offer, Focus ring (`:focus-visible` class), Live region — with the 004-contract kept elements bound to them; (ii) dead removals are STRUCTURAL (element deleted, never `display:none` corpses — S4) and the shipped remainders are GONE: `#versionFooter…` — any other element (visible, hidden, or dead) = inventory violation (SC-001 item (ii) fails). MUST FAIL TODAY — RED blocking T027.

### Implementation for User Story 4

- [X] T027 [US4] Reduced-Lente stamps in `src-tauri/src/status_view.html` + `src-tauri/src/status_view.rs` (SAME subagent/design-owner as T024/T025 — one file-space context): (1) header: bind the Concha Harmônica "R" mark (`DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` copied into the audited surface's asset location per the §SurfaceInventory Header-mark binding; the triad `logo-canonical.svg` stays host-faces-only — C4); (2) metadata card: título/seção on a `--sona-stage` card; the upstream authority fallback (`metadata_title.unwrap_or(filename)` semantics from `crates/player/src/surface.rs:426`) is preserved so a LOADED document never shows an empty title slot; a genuinely empty slot renders the stable glyph «—» in `--sona-hush` with `aria-label="Sem …`; GREEN T026. Commit via hook per file.

**Checkpoint**: US4 host leg green — stamps present, inventory closed, exclusions unreachable, asset census-clean. All four stories' host legs together are exercised by the audit suite (C-H1…C-H8) and the player/diag twins (C-H9).

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: the Q1/Q2 design-dir maintenance, 001/004 point amendments, host-brand faces, ADR registration, and the verify.sh gate wiring. Lands AFTER the token/earcon work lands (Q1: «executed after the token/earcon work lands»).

- [X] T028 [P] DESIGN-SONA alignment + normalization (R1/Q1 — one coordinated task; auditee A1 #4 + #6 files) → **SC-001/SC-006** (census of DESIGN-SONA auditees): (1) the four rate rows align to the 004 contract grammar — Rate change = Unison (RateStep) tick 880 Hz / 60 ms detached from the Allegro skip row — in `DESIGN-SONA.md` §6 (`:60,:85`), `tokens.md` §6, `components.md` §5, `accessibility.md` §2; (2) the failure row records the shipped E4 ≈330 Hz square + minor-2nd ≈450 ms (D3 kept as design origin, marked superseded for pitch); (3) ONE canonical banned green `#5AD6A8` — the `#5FD0A0` mention becomes a recorded erratum; (4) ALL remaining off-set hexes in the auditee dir are reconciled so the census passes: the non-audited token rows (`tokens.md` §2 Branco `#FFFFFF` row, §3 `--signal-flow`/`--signal-info… Commit via hook per file.
- [X] T029 [P] Point amendments to 001/004 (hybrid approach — Assumptions #8) → **FR-005/SC-001** (cohérence `#5AD6A8`/`#5FD0A0` between 004 and 005): in `specs/004-ressoa-dogfood-build/contracts/surface.md` correct the prohibited green to the canonical `#5AD6A8` (the `#5FD0A0` erratum recorded, name-to-value coherence — batch Q1 item (3)) and the «progress white» clause to the `--sona-voice` fill token (C1; the maintainer's veto that would re-admit `#FFFFFF` is recorded as NOT applied); in `specs/004-ressoa-dogfood-build/spec.md` amend SC-004 + the T051-04 protocol text to the B1 split — salto-frente/salto-trás as SEPARATE confirmation types (7-type protocol, ≥15 trials/type, no two equal consecutively, no in-session feedback, 5–10% traps, familiarization WITH feedback, confusion-matrix CSV artifact + direction-of-skip sub-scor… Commit via hook per file.
- [X] T030 [P] Ressonador Canônico host faces (FR-004; SC-006 repo half): the seal lives ONLY in host-facing artifacts — (i) the human CLI banner on **stderr** of the engine-diagnostics face (`crates/diag/src/cli.rs` human-formatter path; human banners never enter data stdout; text banner only — no binary asset dependency); (ii) `README.md` references the artifact `DESIGN-SONA/assets/logo/seal-engine-ressonador.svg`; (iii) a version-metadata note beside `DESIGN-SONA/VERSION` (created in T028). Never on the notification/lock-screen/in-app surfaces (enforced by T026/T029). Commit via hook per file.
- [X] T031 [P] ADR registration (Assumptions #15): **update/verify** `docs/adr/adr-003-earcon-at-focus-loss.md` (file already exists — do NOT recreate) — the formal record that the Ostinato/Sforzando emission AT the focus-loss state-change (same call as the pause) follows ADR-001's deliberate platform deviation (constitution Tauri Platform Contract, Audio Focus / Driving-Safety); cite `research/clarify-b2-earcon-failure-degradation.md` + the ISO 15006 short-term-signal class; record the F4 contract (`reason_ttl_ms = null` — failure latching until state end; cleanup ONLY by new document/session/app-restart). Follow the `docs/*.md` heading pattern. Commit via hook.
- [X] T032 Wire the surface-audit gate (R13/T157 precedent): in `scripts/verify.sh` add a fail-closed `surface-audit` gate step AFTER the gate-2 test step: `cargo nextest run --workspace -E 'test(surface_audit)'` plus the C-H9 player/diag twins (`-E 'test(earcon)'`) — every audit function is named with the `surface_audit_` prefix so the filter is deterministic; the H8 DEFERRED branch is spelled out (a missing BUILT bundle ⇒ reports «DEFERRED (owner=device-build)» exactly like gate-8-full posture and still fails on ANY other named breach; never silently passes); a missing auditee/missing test target fails NAMED (A2: check id + file + line); inverting a check for negative-control runs is prohibited outside a temp copy. Then two-way sync `scripts/expected-tests.txt` with `cargo nextest list --workspace-type all`. Commit via hook.

**Checkpoint**: design dir normalized and tagged; 001/004 contracts coherent with 005 decisions; verify.sh carries the surface-audit step deterministically.

---

## Phase 8: Verification (attestation-only — the ONLY tasks that attest success/completeness)

**Purpose**: run the deterministic gates and the device-leg trials. NOTHING else in this feature claims completeness. Mirrors 004's T051/T052 posture: human trial sheets and honest DEFERRED records — never silent passage.

- [ ] T033 Gate-9 build artifact: `cargo tauri android build --target aarch64` MUST produce a device artifact; `adb install -r <apk>` when the reference phone (Redmi 2201117TY, API 33) is attached; record the artifact path + gate-9 exit. Also re-run `scripts/check-manifest.sh` + `scripts/check-shell-purity.sh` (Tauri Platform Contract: no manifest/permission change; SC-035 Kotlin purity).
- [X] T034 Full deterministic gate run: `scripts/verify.sh` (gates 1–8 + the new `surface-audit` step) MUST exit 0, with the exit code and step list recorded as the binding attestation. Spot-check filters per quickstart §1: `cargo nextest run -E 'test(chroma)' --workspace`, `-E 'test(inventory)' --workspace`, `-E 'test(earcon)' --workspace`.
- [ ] T035 Device trials per quickstart §3 (10/10 sheets; owner=device-build; DEFERRED-with-a-REASON if the device is unavailable — plan.md: «device trials are the only non-host acceptance legs»): §3.1 SC-003 notification purity (marfim accent only; no body text/rate/voice/mark; warning = reason line + Ostinato + 3 s TTL); §3.2 SC-001 visual purity (determinate AND indeterminate; no third color; single column; no excluded screen reachable); §3.3 SC-002 blind-test protocol exactly as written (7 types ≥15 trials, traps 5–10%, confusion-matrix CSV artifact, risky pairs Allegro-trás×Andante / Unison×Allegro-frente, ≥24 h replication, pilot buzz-check + haptic-disposition line mandatory — E3/E6); §3.4 SC-004 mute trial; §3.5 SC-005 device latency ≤100 ms; §3.6 SC-…
- [ ] T037 [P] FR-006 host task: Implement and verify duck-exempt short-mix channel plays under mudo/DND state with host verification oracle; action of transport never blocked by mute; redundancy cor+forma+texto+posição preserved. File: `crates/player/src/backend.rs`. → FR-006 (E-01 remediation).


- [ ] T036 Final privacy + scope glance (quickstart §4) + attestation record: `Cargo.lock` diff = UNTOUCHED (zero new dependencies — constitution IV); `diag scan` shows no new egress beyond 004's known set; the haptics-inert disposition held during §3.3 (E3); launcher icon unchanged (D4); update `.specify/memory/STATE.md` with the final resume point + gate evidence. The completion report cites T034's verify.sh exit + T033's artifact as the binding attestations; every DEFERRED leg is named explicitly with its owner (device-build).

**Checkpoint**: Only T033–T036 may attest success/completeness. No agent statement elsewhere attests.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies — starts immediately; BLOCKS everything (T002 green-baseline is a hard stop gate).
- **Phase 2 (Foundational)**: depends on Phase 1; BLOCKS all user-story implementation (T004→T005→T006 strictly sequential; T007–T011 RED may land in Wave 3).
- **User-story phases (3–6)**: all depend on Phase 2. US1 and US2 are both P1 and run in PARALLEL after Phase 2 (disjoint file families). US3's RED wave (T007–T011) may START while US1/US2 implementations land — and T007–T011 must exist (RED) before T024 regardless. US4's T026 needs T008/T011 to exist (its inventory leans on the focus-ring/live-region twins) and must be shown FAILING before T027 regardless; its file is DISJOINT from the T007–T011 twins (T004's `surface_audit/inventory.rs` scaffold — no byte sharing), so the RED may land in Wave 3 or as Wave 4's first step — that owner then runs T024 → T025 → T027.
- **Phase 7 (Polish)**: T028/T029 run after US2/US3/US4 implementations land (Q1 ordering); T030/T031 anytime after Phase 2; T032 strictly after ALL twins exist.
- **Phase 8 (Verification)**: depends on ALL previous phases; the ONLY attestations.

### Within-Story Order (TDD red-to-green)

- Every story: RED test tasks FIRST (all shown failing with NAMED reasons) → **GATE** → implementation tasks → GREEN checkpoint.
- Models before services: the diag kind extension (T017) precedes its consumer (T018).
- Same-file edits (`status_view.html`: T024 → T025 → T027) run SEQUENTIALLY under one file-owning subagent — never parallel edits to one file; the audit twins are per-check child modules (one file per check, T004(e)) so separate agents never collide.

### Parallel Execution (subagent waves)

```
Wave 1 (after T001–T003) — Phase 2, one agent (same module + tauri.conf context):
  T004 scaffolding → T005 RED → T006 wiring      (strictly sequential)

Wave 2 (after T006) — P1 stories, two disjoint agents:
  Agent B (accent.rs census RED + MediaMirror.kt constants): T012 RED → T013   [US1]
  Agent C (crates/player + crates/diag): T014 RED, T015 RED, T016 RED
        → T017 (diag) then T018 (player, consumes the kinds)  [US2]

Wave 3 (after T006; overlaps Wave 2 by file-family) — one agent, five tasks
  (disjoint test files in one shared module — single context, restyle not required yet):
  T007/T008/T009/T010/T011 RED (src-tauri::surface_audit)      [US3]

Wave 4 (after Waves 2+3 twins are RED; exclusive file owner):
  Agent E (status_view.html + status_view.rs + surface_audit/inventory.rs):
    T026 RED (inventory) → T024 restyle → T025 aria → T027 stamps
    (sequential; the US4 RED T026 is shown failing BEFORE T027 lands)   [US3+US4]

Wave 5 (after Wave 4) — polish, four disjoint agents in parallel:
  T028 (DESIGN-SONA + tag) | T029 (specs/004) | T030 (README+diag CLI) | T031 (docs/adr)
  → then T032 (verify.sh + expected-tests.txt) — one agent, after all twins exist

Wave 6 (final attestation chain — strictly sequential):
  T033 → T034 → T035 → T036
```

### Parallel Example: US3 test wave (one agent, same context)

```bash
Task: "RED surface_audit_chroma_census in src-tauri/src/surface_audit/chroma.rs"
Task: "RED surface_audit_role_exclusivity in src-tauri/src/surface_audit/exclusivity.rs"
Task: "RED surface_audit_movement_census in src-tauri/src/surface_audit/movement.rs"
Task: "RED surface_audit_contrast_solver in src-tauri/src/surface_audit/contrast.rs"
Task: "RED surface_audit_aria_inventory in src-tauri/src/surface_audit/aria.rs"
```

### Parallel Example: P1 stories (Wave 2)

```bash
Agent B: T012 → T013                                # accent census twin + MediaMirror.kt constants (US1)
Agent C: T014, T015, T016, T017, T018               # burst/oracle/diag kinds (US2)
```

## Implementation Strategy

### MVP First (US1 + US2 — the two P1 stories)

1. Complete Phase 1: Setup → Phase 2: Foundational
2. Wave 2 in parallel: US1 (accent-only census → constants) + US2 (burst/oracle/diag)
3. **STOP and VALIDATE**: `surface_audit_accent_only_constants`, `prop_earcon_burst_latest_wins`, `test_earcon_latency_oracle`, diag twins — all green; device legs DEFERRED with reason
4. Ship the P1 increment; P2/P3 layer onto the same audited file

### Incremental Delivery

1. Setup + Foundational → foundation (scaffold, RED twins, S2 wiring)
2. US1 + US2 (P1) → MVP host legs green
3. US3 (P2) → token baseline + aria + movement → the suite grows C-H1/H2/H5/H6/H7
4. US4 (P3) → Lente reduzida stamps + closed inventory → C-H4 + asset census
5. Polish → design-dir normalization/tag + 004 amendments + host faces + gate wiring
6. Verification → gates 1–8 + surface-audit + gate 9 + device sheets — ONLY these attest

## Test Coverage Map (quickstart §1 ↔ contract A2 naming)

| Check | Twin tests | Workspace target |
|---|---|---|
| C-H1 chroma census | `surface_audit_chroma_census` (both progress states) | `src-tauri` |
| C-H2 role exclusivity | `surface_audit_role_exclusivity` | `src-tauri` |
| C-H3 accent-only | `surface_audit_accent_only_constants` (parses the Kotlin block) | `src-tauri` |
| C-H4 element inventory | `surface_audit_element_inventory` (+ asset census + exclusions) | `src-tauri` |
| C-H5 aria inventory | `surface_audit_aria_inventory` | `src-tauri` |
| C-H6 movement census | `surface_audit_movement_census` | `src-tauri` |
| C-H7 contrast solver | `surface_audit_contrast_solver` | `src-tauri` |
| C-H8 served-identity | `surface_audit_served_identity_binds_status_view_as_bundle` (DEFERRED branch when bundle missing) | `src-tauri` |
| C-H9 short-mix behavior | `prop_earcon_burst_latest_wins` + `test_earcon_latency_oracle` + `test_earcon_truncation_diag` (+ diag scan twins) | `crates/player` · `crates/diag` |

## Notes

- **Source of precedence**: the check specification is `contracts/surface-audit.md` A1–A4; the audited sets are `data-model.md`; binding rules are `contracts/status-surface.md` S1–S7 + `contracts/earcon-mapping.md` E1–E7. When a task description and a contract disagree, the contract WINS — and the disagreement is a task-level bug to record in the commit summary, never a local code choice.
- **A4 whitelist discipline**: the auditee list and the allowed hex set are DATA owned by the contracts; loosening them = coordinated contract edit (same discipline as closed sets), never a script tweak; a failure is never silenced by deleting an auditee or an assertion — resolve it in the audited file.
- **fail-closed everywhere**: a missing auditee/bundle/ledger is a FAILED check (twins, T003 anchors, T008/T011 auditee list), never a skip.
- **zero new dependencies** (constitution IV): every twin is pure Rust over auditee bytes; `proptest` is already in the workspace; `Cargo.lock` untouched is itself a Polish/Verification-checkable claim.
- **gates stay the nine constitution gates** (R13): `surface-audit` is a verify.sh STEP, not a tenth gate — no amendment required.
- **closed-set discipline** (§VII): `EventKind` closed at exactly eleven; `EarconKind` untouched (004 grammar owns pitches); the audited hex set stays 10 members (harp via ledger); every addition = coordinated edit + version bump + docs.
- Commit every file edit via `scripts/commit-hook.sh` (rule 4); subagent waves pay each file closure immediately — the hook is the arbiter of the clean delta.
- Stop at any checkpoint to validate the story independently; every story's host legs are runnable without a device.
- [NOTE] A10 (remediation, 2026-10-26): F3 empty-slot glyph + aria-label="Sem título"; F4 static red banner (reason_ttl_ms=null, one-shot <=200ms, no timer) — link to SC-006/SC-005 device trials. See spec.md F3/F4 (2026-10-24, F1-F4).

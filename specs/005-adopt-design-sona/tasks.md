# Tasks: Adotar o DESIGN-SONA (Option A subset)

**Input**: Design documents from `/specs/005-adopt-design-sona/`
**Prerequisites (all loaded)**: plan.md · spec.md (36 clarifications) · research.md (R1–R13) · data-model.md (§SonaToken, §SurfaceInventory, §NotificationSurface, §EarconConfirmation, §RateLadder, §ProgressIndication, §MovementSpec, §HapticDisposition, §DiagnosticEvent, §DesignDirState, §ContrastPair) · contracts/ (`status-surface.md` S1–S7, `earcon-mapping.md` E1–E7, `surface-audit.md` A1–A4) · quickstart.md · checklists/requirements.md (16/16) · `.specify/memory/constitution.md` (v1.12.0)

**Tests**: INCLUDED and NON-OPTIONAL — TDD is binding (constitution Principle III «Test-First (NON-NEGOTIABLE)» + user's rule 1). Every user-story wave writes its tests FIRST and every test must be shown FAILING (RED) before that story's implementation tasks may start. Property-based testing (`proptest` 1.12.0, already a workspace dev-dependency) is mandated wherever a randomization oracle applies (burst policy), and exhaustive-claim form ("property over every occurrence") is used for the byte-level audits.

**Organization**: Tasks are grouped by user story (US1 P1 notification accent, US2 P1 earcon confirmations, US3 P2 StatusView token baseline, US4 P3 reduced Lente), plus Setup, Foundational, Polish and a final Verification phase.

---

## Execution Protocol (user's four rules — binding on every task)

0. **Scaffold-first codegen**: before hand-writing ANY code, generate its mechanical skeleton first — enum variants + derive blocks, test-module signatures, proptest strategy stubs, the crate-style `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]` and `#[serde(rename_all = "snake_case")]` blocks — via the LSP code actions (rust-analyzer) or by verbatim-reusing the matching shipped pattern in the same crate, THEN fill in the logic by hand. Hand-written logic never precedes generated structure.
1. **TDD RED gate (BLOCKING)**: tasks marked `RED` write tests ONLY. A story's implementation tasks are BLOCKED until (a) every RED test of that story exists and (b) it is demonstrated FAILING on the current tree with the NAMED failure (e.g. "off-set hex `#FFFFFF` in `status_view.html:26`", "EventKind has nine kinds, expected eleven") — the test must fail for the RIGHT reason, never for a compile error that masks the oracle. Constitution III: red-to-green in the task order below.
2. **[P] = parallel wave, subagent-driven**: `[P]` tasks touch disjoint files and are dispatched in waves to subagents. Tasks in the same wave sharing context (same crate/file family) run on the SAME subagent; disjoint families get separate subagents. The wave tables under "Parallel Execution (subagent waves)" are the dispatching authority. One file = one owner: parallel agents never edit the same file.
3. **Verification attests, never the agent**: ONLY the tasks in "Phase 8: Verification (attestation-only)" may attest success/completeness, each by running its named deterministic command/trial sheet and recording the artifact. Statements of completeness anywhere else in this feature are prohibited.
4. **Deterministic commit hook**: after EVERY file edit finishes, run `scripts/commit-hook.sh <file-path> "<detailed summary of the edits made>"` — it performs the atomic `git add` + conventional commit of that single file (user rule 4; constitution V determinism). Summaries are detailed (what changed + task ID), e.g. `feat(005): T008 retokenize MediaMirror.kt progress-fill constant to --sona-voice (C1)`. No manual `git commit` outside the hook.

---

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (disjoint files, no dependency on an incomplete task)
- **[Story]**: US1–US4 · Setup/Foundational/Polish/Verification phases carry NO story label
- Exact file paths in every description

## Path Conventions

Amendment to the existing implemented workspace (branches off `dev`, which carries 001–004):
- `src-tauri/src/status_view.html` — the single in-app surface of record (758 lines shipped)
- `src-tauri/src/status_view.rs`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`
- `crates/player/src/{backend.rs,earcon.rs}` · `crates/diag/src/writer.rs`
- `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (constants block `:728-743`)
- `DESIGN-SONA/`, `scripts/{verify.sh,expected-tests.txt}`, `specs/004-ressoa-dogfood-build/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: feature branch, green baseline, prerequisite anchors — nothing else may start until this phase commits clean.

- [ ] T001 Create feature branch: `git switch dev && git switch -c 005-adopt-design-sona` (the branch tracks this feature per plan.md); abort on a dirty tree (`git status --porcelain` must be empty). Record the branch point SHA in the T003 STATE note
- [ ] T002 Baseline gate evidence: run `scripts/verify.sh` (gates 1–8) on the untouched tree; it MUST exit 0. Record the run in the T003 STATE note. If the baseline is red, STOP the feature — a non-green baseline invalidates every RED→GREEN claim of this task list
- [ ] T003 Prerequisite anchor audit + memory update: verify all of the following exist at the stated anchors (a miss FAILS the task — fail-closed): `crates/player/src/backend.rs` `enqueue_earcon` (≈:511, FIFO `push_short_mix` ≈:288-290), `crates/player/src/earcon.rs` `EarconKind` (7 variants ≈:11-25), `crates/diag/src/writer.rs` `EventKind` (exactly nine kinds ≈:69), `src-tauri/src/status_view.html` (slates `:21-22`, white fill `:26`, signal vars `:29-30`, shimmer `:400,:748`), `src-tauri/src/status_view.rs`, `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` constants block `:724-746` (incl. `DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF`), `src-tauri/tauri.conf.json` `"frontendDist": "../dist"`, `scripts/verify.sh`, `scripts/expected-tests.txt`, `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`, `DESIGN-SONA/tokens.md` §1 ledger, `specs/004-ressoa-dogfood-build/contracts/{earcon.md,surface.md}`. Append the resume point + baseline evidence to `.specify/memory/STATE.md`; commit it via the hook

**Checkpoint**: branch created, baseline green, all anchors verified — wave execution may begin.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the surface-audit twin infrastructure every story's tests share, and the S2 packaging wiring (Assumptions #9 + contract S2: «the wiring is this feature's first implementation task»).

**⚠️ CRITICAL**: No user-story implementation may begin until this phase is complete.

- [ ] T004 Create the audit scaffolding `src-tauri/src/surface_audit.rs` (new module, `pub mod surface_audit;` in `src-tauri/src/lib.rs` — scaffold generated first via rust-analyzer then filled): (a) closed auditee resolver — the exact 6-entry list of `contracts/surface-audit.md` A1 (1 `src-tauri/src/status_view.html`; 2 `src-tauri/src/status_view.rs` relevant constants; 3 `android-shell/…/MediaMirror.kt` slice `:728-743`; 4 `DESIGN-SONA/{tokens.md, components.md, DESIGN-SONA.md, preview.html, CHANGELOG.md, VERSION}`; 5 `src-tauri/tauri.conf.json`; 6 `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg`), paths resolved from `CARGO_MANIFEST_DIR/..`; a missing auditee returns a named error — fail-closed, never skip; (b) a FIXED parser — byte-scanner producing ordered `(offset, kind)` spans for hex literals (`#rrggbb`, `rgba(...)`, `0xAARRGGBB`), CSS selector/at-rule blocks and HTML attribute runs; **no unguarded regex over documents** (contract A3); (c) ledger consumer — parses `DESIGN-SONA/tokens.md` §1 table rows into `token → hex` and resolves the `--sona-harp` constant (`#E8C46A` at alignment time); the census never hardcodes harp — a painted harp that differs from the ledger constant FAILS (drift fails the check); (d) audited-set types: the 10 hexes of `data-model.md` §SonaToken (`--sona-void #0B0F1A`, `--sona-stage #141B2D`, `--sona-voice #F2F0E6`, `--sona-etwahl #8FD8E8`, `--sona-hush #A9B8CE`, `--sona-celeste #B9A6F5` reserved, `--sona-boundary #55688F` + signal paints `--signal-caution #FFD400`, `--signal-failure #FF4D63`) with `--sona-harp` ledger-resolved; (e) create the EIGHT twin child modules as empty scaffolds with `pub mod` declarations wired in the parent: `src-tauri/src/surface_audit/{served_identity,accent,chroma,exclusivity,movement,contrast,aria,inventory}.rs` — ONE file per check, so each RED task fills ONLY its own child file and parallel agents never collide on bytes (rule 0: generate the empty module skeleton first, fill later). Zero new dependencies. Commit via hook
- [ ] T005 RED C-H8 served-identity twin: fill `src-tauri/src/surface_audit/served_identity.rs` (T004's empty scaffold) with test `surface_audit_served_identity_binds_status_view_as_bundle` — parse `src-tauri/tauri.conf.json` `frontendDist`, read the served bundle entry (`<frontendDist>/index.html`), assert **byte-identity** with `src-tauri/src/status_view.html` (contract S2: «MUST BE byte-identical… at acceptance»); a missing bundle or missing auditee = NAMED failure (fail-closed; A2: check id + file + line estimate). MUST FAIL NOW with the named reason «served bundle is the dist/index.html bench, byte-differs from src-tauri/src/status_view.html» — this RED BLOCKS T006
- [ ] T006 Implement S2 packaging wiring: make the packaged app actually serve `src-tauri/src/status_view.html` as the bundle entry — build-time copy `status_view.html` → `dist/index.html` (a `src-tauri/build.rs` step, or the repo's existing packaging script — whichever keep the twin green on every build); `src-tauri/tauri.conf.json` `frontendDist` stays `../dist` (or is repointed consistently — either way T005 must pass byte-identity). «The bench `dist/index.html` content may remain as a fallback only if it does not leave the audited surface unserved» — it leaves it unserved as the default route, so the bench page is REPLACED; with it dies the numeric-rate bench leak «Taxa: 1.0×» (data-model §SurfaceInventory dead-removal list). Update the `src-tauri/src/status_view.rs` doc-header to declare the wiring. GREEN T005 (C-H8, missing-bundle case exercises only the DEFERRED branch). Commit via hook per file

**Checkpoint**: Foundation ready — audit scaffolding + ledger consumer exist; served-identity twin green. User-story RED waves below may now start.

---

## Phase 3: User Story 1 — Identidade visual/sonora da notificação (Priority: P1) 🎯 MVP

**Goal**: the MediaStyle notification keeps ONLY the platform's single accent channel set to `#F2F0E6` (closed member `--sona-voice`) — título + seção atual + progresso only, no body text/rate/voice/mark; warning = static reason line (TTL 3 s) + Ostinato earcon; the pulsing dot lives in-app, never in the notification (D2 erratum recorded in the dir by T023).

**Independent Test**: `surface_audit_accent_only_constants` green — then the device inspection of quickstart §3.1 (10/10 sheets; device leg lives in Phase 8).

### Tests for User Story 1 (write FIRST; must FAIL before T008) ⚠️

- [ ] T007 [P] [US1] RED accent-only twin (C-H3) `[US1]` test `surface_audit_accent_only_constants` in `src-tauri/src/surface_audit/accent.rs` (T004's empty scaffold): parse the MediaStyle constants block of `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` (slice `:728-743` only — auditee A1 #3; the fixed `0xAARRGGBB` literal parser, not regex) and assert: (i) the app-painted value set is EXACTLY `{"0xFFF2F0E6"}` — the accent `--sona-voice` — so TODAY it FAILS because `DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF` paints off-set white (C-H1/H3 violation, C1: «the literal-white reading is vetoed»); (ii) structural invariants of the block: no mark/logo constant, no rate literal, no body-text constant (FR-002/FR-028 — `setContentText` non-null guard stays 004's `:1418` pattern, cited by comment, not duplicated); (iii) the stored progress-fill token equals the `--sona-voice` resolver/ledger constant (a future drift to any other value fails the check). Scaffold-first: test-module signature generated before logic. Run it, record the failing output in the commit summary

### Implementation for User Story 1

- [ ] T008 [US1] Update the `MediaMirror.kt` constants block `:728-743`: replace `DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF` with `0xFFF2F0E6` and the comment «fill = `--sona-voice` — registro 2026-10-24, C1; contract "progress white" superseded»; retag the block's KDoc to the canonical token names (`--sona-voice`, `--signal-caution`, `--signal-failure`). NO other Kotlin change — declarative mirror stays (SC-035), no manifest/permission change (D4 exclusion intact). GREEN T007. Commit via hook

**Checkpoint**: US1 host leg green — the MediaStyle block paints exactly `#F2F0E6`; no off-set paint. Device inspection (SC-003) belongs to Phase 8 (device-build owner).

---

## Phase 4: User Story 2 — Confirmar controles por earcon (Priority: P1) 🎯 MVP

**Goal**: eyes-free confirmation for all 7 confirmation types (play→Adagio · pause→Andante · rate-step→Unison · skip-fwd→Allegro-frente · skip-back→Allegro-trás · warning→Ostinato · failure→Sforzando) under the SHIPPED 004 grammar (pitches never re-specified); burst = truncate/substitute «latest wins»; system earcons interrupt; emission failure → action proceeds + AT MOST one re-emission + typed diag event; ≤100 ms first-consumed-sample.

**Independent Test**: `prop_earcon_burst_latest_wins` + `test_earcon_latency_oracle` + diag twins green — then quickstart §3.3 blind-test protocol (device leg, Phase 8).

### Tests for User Story 2 (write FIRST; must FAIL before T012/T013) ⚠️

- [ ] T009 [P] [US2] RED burst/overlap property tests (property-based, `proptest`): add the `#[cfg(test)] mod earcon_burst` to `crates/player/src/backend.rs` with `prop_earcon_burst_latest_wins` — for random interleaved action sequences (bursts of 2–3 heterogeneous confirmations inside 100–150 ms; ~10 000 cases) assert the invariants: (1) the single `short_mix` channel is never widened (no second channel, no pre-mix chain — contract E2.5); (2) an in-flight USER earcon is TRUNCATED — its tail dropped BEFORE the append — so the newest action's first sample is the next consumed one («latest wins», E2.2); (3) SYSTEM earcons (Ostinato/Sforzando, from `EarconKind::{Warning,Failure}`) interrupt any in-flight user confirmation and fire immediately (E2.3); (4) the additive mix stays earcon↔narração ONLY (`mul_add` per slot; earcon↔earcon mixing inside one slot fails); (5) pause drop-and-refire stays scoped to the user-initiated set; (6) the FIFO residual serialization of `short_mix` (shipped form `backend.rs:266,285-290`) FAILS the property today (D3). Scaffold-first: proptest strategies generated as stubs before the oracle. MUST FAIL before T013 (RED)
- [ ] T010 [P] [US2] RED latency oracle `[US2]` test `test_earcon_latency_oracle` in `crates/player/src/backend.rs` tests — reuse 004's T152 first-consumed-sample technique: the first consumed short-mix sample for EVERY trigger (all 7 confirmations) arrives ≤100 ms from its trigger (system Ostinato/Sforzando measured from the state-change shot), in isolation AND under the burst patterns of T009 (FR-010; D3: «the bound survives rajadas»). MUST FAIL under bursts TODAY (FIFO tail delays >100 ms) — RED blocking T013
- [ ] T011 [P] [US2] RED diag kind twins `[US2]`: in `crates/diag/src/writer.rs` extend `contract_schema` tests: (a) the closed `EventKind` set becomes EXACTLY ELEVEN — adding `EarconTruncation` (truncation counter) and `EarconEmissionFailure` (typed emission-failure event) per `data-model.md` §DiagnosticEvent — assert serde snake_case round-trip for all 11 and that NO `Other`/`Unknown` arm exists; (b) negative serialization: an unknown kind must fail deserialization (never map); in `crates/player` add `test_earcon_truncation_diag`: every truncation of an in-flight user earcon registers exactly ONE `EarconTruncation` entry; the emission-failure path (synthesis error / short channel unavailable) registers the typed event carrying the required `error` field AFTER the single re-emission (never silence-without-registration — edge case #72); extend `crates/diag/src/scan.rs` tests: the scanner accepts the two new kinds (`diag scan --format json` paths) and `diag::prop_no_prose` still passes for them (`detail` ≤200 chars, identifier-pattern, no document text). MUST FAIL (kinds absent) — RED blocking T012/T013

### Implementation for User Story 2

- [ ] T012 [US2] Diag §VII coordinated extension: add `EarconTruncation` + `EarconEmissionFailure` variants to `EventKind` in `crates/diag/src/writer.rs` — scaffold-first (rust-analyzer «add variant», keeping the derive + enum style verbatim), then hand-fill doc comments citing `data-model.md` §DiagnosticEvent + `contracts/earcon-mapping.md` E3/E4. Closed-set discipline (§VII): the SAME task carries the coordinated edits — `crates/diag/src/scan.rs` scanner arms (explicit, no `_` arm, no `Other`), `contract_schema` text updated from «exactly nine» to «exactly eleven», the diag crate semver bump in `crates/diag/Cargo.toml`, and the docs line in `docs/diagnostic-record.md`. GREEN T011. Commit via hook per file
- [ ] T013 [US2] Implement burst policy + emission failure in `crates/player/src/backend.rs`: (1) classify incoming earcons User vs System by `EarconKind` (Play/Pause/SkipFwd/SkipBack/RateStep = user; Warning/Failure = system — the mapping of `contracts/earcon-mapping.md` E1; `EarconKind` stays unchanged, pitches never re-specified); (2) `enqueue_earcon` truncates: drop the tail of an in-flight USER earcon BEFORE appending the new one (latest-wins); a SYSTEM earcon takes the channel head immediately (interrupt); (3) synthesis error / short-channel unavailable → the transport action proceeds, narration untouched, AT MOST ONE re-emission at the next render space, then ONE typed `EarconEmissionFailure` diag event (`error` field set) — replacing the current silent no-op; (4) each truncation increments the `EarconTruncation` counter once; (5) single `short_mix`, `mul_add` slots, pause drop-and-refire untouched. GREEN T009/T010/T011. Commit via hook

**Checkpoint**: US2 host leg green — burst policy, ≤100 ms oracle, eleven-kind diag record with typed counters. Device blind-test legs belong to Phase 8.

---

## Phase 5: User Story 3 — Superfície in-app com tokens do DESIGN-SONA (Priority: P2)

**Goal**: the StatusView token baseline — marfim `#F2F0E6` título + progress fill (flat, both states, no gradient) over trilho `--sona-boundary #55688F`, periwinkle `--sona-hush` seção, indeterminate = solid sliding pill (004's shimmer REMOVED — C5), reduced-motion freeze (E2), closed palette with role exclusivity (D2/E1), aria semantics inventory (D5).

**Independent Test**: the five US3 twins green (`surface_audit_chroma_census`, `surface_audit_role_exclusivity`, `surface_audit_movement_census`, `surface_audit_contrast_solver`, `surface_audit_aria_inventory`) — then visual trial quickstart §3.2 (Phase 8).

### Tests for User Story 3 (write FIRST; all must FAIL before T019/T020) ⚠️

- [ ] T014 [P] [US3] RED closed-hex census twin (C-H1) `[US3]` test `surface_audit_chroma_census` in `src-tauri/src/surface_audit/chroma.rs`: walk the closed auditee list (T004 resolver; today the dir auditees have NO `CHANGELOG.md`/`VERSION` — itself a named fail-closed breach): every hex token in every auditee must be a member of the audited set (10 + ledger harp), audited over BOTH progress render branches of `status_view.html` (determinate fill branch AND the `indeterminate` pill/JS branch — no third audited state). Violation classes fail with the check id + file + line: (i) any hex outside the set; (ii) jade `#5AD6A8`/erratum `#5FD0A0` paint; (iii) celeste `#B9A6F5` paint (vacancy — §SonaToken: «zero paint in audited files = INVENTORY VIOLATION»); (iv) slate leftovers `#0f172a`/`#1e293b` and off-set progress white `#FFFFFF`; (v) `rgba()`/derived variants (contract S3: «no rgb()/derived variants»); for the DESIGN-SONA auditees the ONLY forgiveness is lines explicitly marked `ERRATA:` (each naming exactly one recorded erratum — created by T023). MUST FAIL TODAY (slates/white present; dir missing CHANGELOG/VERSION) — RED blocking T019/T023
- [ ] T015 [P] [US3] RED role-exclusivity twin (C-H2) `[US3]` test `surface_audit_role_exclusivity` in `src-tauri/src/surface_audit/exclusivity.rs`: `#8FD8E8` occurs ONLY inside `:focus-visible` rules (TalkBack draws its own ring — E1; today ZERO `:focus-visible` exists); `#FFD400` paints ONLY {reason banner warning state, in-app 3 s caution dot}; `#FF4D63` paints ONLY permanent-failure; caution×failure never co-render in the same declared state (mutual contrast 2.25:1 — C-H2 adjacency clause); any off-role paint = the D2 violation class. MUST FAIL TODAY — RED blocking T019/T020
- [ ] T016 [P] [US3] RED movement census twin (C-H6) `[US3]` test `surface_audit_movement_census` in `src-tauri/src/surface_audit/movement.rs`: parse every `@keyframes`/`animation`/`transition` declaration in `status_view.html` (fixed parser) — ≤1 continuous loop, exactly the sliding pill; a `@media (prefers-reduced-motion: reduce)` guard EXISTS and freezes the pill at a VISIBLE on-track position (`animation-play-state: paused` + static offset — an invisible/opacity-zero freeze FAILS, contract S5); the yellow pulse carries `animation-iteration-count` bounded by the 3 s TTL (E2: strictly inside the TTL); NO pulse on the failure color; banner `slideIn` one-shot ≤200 ms (no looping slide); any other loop fails. MUST FAIL TODAY (`indeterminate 1.5s infinite` at `:402,:748`; shimmer gradients; no media query anywhere — E2 dive) — RED blocking T019
- [ ] T017 [P] [US3] RED contrast solver twin (C-H7) `[US3]` test `surface_audit_contrast_solver` in `src-tauri/src/surface_audit/contrast.rs`: implement the WCAG 2.1 AA contrast calculator as a PURE function (relative luminance + ratio, parameterized by effective background — void `#0B0F1A` / stage `#141B2D` — so future light modes reuse it; `research.md` R12) with a unit table (sanity margins: fill 16.75/15.01 over void/stage, fill×track 4.88, `--sona-boundary` 3.43/3.07 narrow non-text pass; the `--sona-aria` declared-drift case 11.20→10.59 documented), then assert every painted pair parsed from `status_view.html` meets text ≥4.5:1 / non-text ≥3:1. Declared design-dir numbers are documentation, NOT authority — the computation is (`research.md` R12). A failing pair fails the check (resolution = the ledger's own recipe, applied in T019 — never an off-set rescue hex). MUST FAIL TODAY (off-set paints make pairs unbindable) — RED blocking T019
- [ ] T018 [P] [US3] RED aria-inventory twin (C-H5 / SC-001 item (iv)) `[US3]` test `surface_audit_aria_inventory` in `src-tauri/src/surface_audit/aria.rs`: over `status_view.html` — every enumerated interactive control carries a non-empty `aria-label` + state attribute; `#progress` carries `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; a visually-hidden `role="status"` live region exists (the ladder value channel; the `#speedDots` strip stays `aria-hidden`); `#reasonBanner` is typed `role="status"` and the legacy `role="alert"`+`aria-live="polite"` pair (`:252`) is ABSENT; NO painted numeric rate anywhere (painted-numeral scan on `Taxa:`/`×` patterns fails — the value-blind invariant S4). MUST FAIL TODAY (no aria-valuetext in the file; alert+polite pair shipped) — RED blocking T020

### Implementation for User Story 3

- [ ] T019 [US3] Token restyle + movement in `src-tauri/src/status_view.html` (EXCLUSIVE file owner — same subagent as T020/T022; no parallel edits to this file): (1) replace the var block `:21-30` with the closed 10-token palette using canonical names (`--sona-void #0B0F1A`, `--sona-stage #141B2D`, `--sona-voice #F2F0E6`, `--sona-etwahl #8FD8E8`, `--sona-hush #A9B8CE`, `--sona-harp #E8C46A` (= ledger), `--sona-celeste #B9A6F5` (declared only — ZERO paint), `--sona-boundary #55688F`, `--signal-caution #FFD400`, `--signal-failure #FF4D63`); rename `--warning`/`--failure` → canonical `--signal-caution`/`--signal-failure`; DELETE the `rgba()` tints (`:179-180`) — warning/failure paint roles follow T015's sanctioned set; (2) backgrounds `#1e293b`/`#0f172a` and their gradients (`:21-22,:57,:191`) → `--sona-void` flat (the metadata card goes `--sona-stage` in Phase 6); (3) progress: determinate = flat `--sona-voice` fill over the `--sona-boundary` track; indeterminate = solid sliding pill `--sona-voice` over the same track — DELETE `linear-gradient(90deg, transparent, var(--progress-fill), transparent)` (`:400`) and the shimmer keyframes (`:748-752`), replace with the pill animation; value-blind preserved (no numeric rate painted); (4) movement contract (E2): ONLY the pill loops; add `@media (prefers-reduced-motion: reduce)` freezing it paused at a VISIBLE offset; the cautions pulse bounded inside the 3 s TTL; failure paint static (no keyframe); `slideIn` one-shot ≤200 ms preserved; single-column geometry preserved (C6 — no 62%/38% grid); (5) `:focus-visible` ring `--sona-etwahl` on every enumerated interactive control (E1). GREEN T014–T017 (inventory completion is Phase 6's T021). Commit via hook
- [ ] T020 [US3] Aria semantics in `src-tauri/src/status_view.html` (SC-001 item (iv), SC-007 foundation; data-model §SurfaceInventory Live-region row; D5): add `role="progressbar"` + `aria-valuenow`/`aria-valuetext` (pt-BR valuetext) on `#progress`; add the visually-hidden live region `role="status"` announcing the ladder value («velocidade 1,25») and the reason/failure states — `#speedDots` stays `aria-hidden`; RE-TYPE `#reasonBanner` from the incompatible `role="alert"`+`aria-live="polite"` pair (`:252`) to a single `role="status"`; every enumerated interactive control (`#btnSkipBack`, `#btnPlayPause`, `#btnStop`, `#btnSkipForward`, `#btnRateDown`, `#btnRateUp`, the offer-dialog buttons) carries a non-empty `aria-label` + state attribute. GREEN T018. Commit via hook

**Checkpoint**: US3 host leg green — closed palette + exclusivity + movement contract + contrast solver + aria channel on the restyled source of record.

---

## Phase 6: User Story 4 — Lente Sinestésica reduzida (Priority: P3)

**Goal**: the reduced-Lente stamps on the SAME StatusView — Concha Harmônica "R" header (C4 variant: no jade, no celeste), `--sona-stage` metadata card (C2/batch 96), structural dead removals (B4), CLOSED element inventory (exactly the §SurfaceInventory 9 entries), no path to the excluded screens (S1), header-asset census-clean.

**Independent Test**: `surface_audit_element_inventory` green (incl. excluded-screen unreachability + asset census) — then quickstart §3.6 (Phase 8).

### Tests for User Story 4 (write FIRST; must FAIL before T022) ⚠️

- [ ] T021 [P] [US4] RED element-inventory twin (C-H4 + SC-006 host half) `[US4]` test `surface_audit_element_inventory` in `src-tauri/src/surface_audit/inventory.rs` (T004's empty scaffold — its file is disjoint from the Wave-3 twins, so it may run in Wave 3 or 4; it must be RED before T022): over `status_view.html` (fixed parser, order-preserving): the element inventory is EXACTLY the closed §SurfaceInventory enumeration — Header mark, Metadata card, Progress, Transport controls, Rate ladder, Reason line, Resume offer, Focus ring (`:focus-visible` class), Live region — with the 004-contract kept elements bound to them; (ii) dead removals are STRUCTURAL (element deleted, never `display:none` corpses — S4) and the shipped remainders are GONE: `#versionFooter` (`:223,:254`), unrendered state fields, the always-present «Seção: —» row (`:234`), no `Taxa:` remnant; (iii) excluded screens unreachable — no route/link/hidden handler to Cockpit Eyes-Free, Console Braun & Rams, Sumário Editorial Suíço, Matriz de Telemetria (S1: reachable «by code inspection»); (iv) the header asset `DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` (auditee A1 #6) is census-clean: ZERO `#B9A6F5` and ZERO `#5AD6A8` (C4 — asserted on the file, never trusted); the canonical triad `logo-canonical.svg` is NOT referenced by the surface (host faces only); (v) the Ressonador Canônico seal appears NOWHERE on device surfaces (SC-006 host half). MUST FAIL TODAY (footer/section-row/state fields present; no header mark) — RED blocking T022

### Implementation for User Story 4

- [ ] T022 [US4] Reduced-Lente stamps in `src-tauri/src/status_view.html` + `src-tauri/src/status_view.rs` (SAME subagent/design-owner as T019/T020 — one file-space context): (1) header: bind the Concha Harmônica "R" mark (`DESIGN-SONA/assets/logo/logo-primary-concha-r.svg` copied into the audited surface's asset location per the §SurfaceInventory Header-mark binding; the triad `logo-canonical.svg` stays host-faces-only — C4); (2) metadata card: título/seção on a `--sona-stage` card; the upstream authority fallback (`metadata_title.unwrap_or(filename)` semantics from `crates/player/src/surface.rs:426`) is preserved so a LOADED document never shows an empty title slot; a genuinely empty slot renders the stable glyph «—» in `--sona-hush` with `aria-label="Sem título"` (card content — no new element, F3); a null section line is OMITTED structurally (never «Seção: —» — F3); cold-start/empty render (F1): transport disabled (the real pre-document `get_status_view` → `Err("no document loaded")` state), resume offer only when applicable, NO first-run CTA/picker/instruction line; (3) rate ladder: the positional gold dot `--sona-harp` on the closed 7-rung ladder `[0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 4.0]` (value-blind — the T020 live region carries the value); (4) delete the dead remainders structurally (T021's list); (5) keep `status_view.rs` constants/id hooks element-inventory-exact (auditee A1 #2). GREEN T021. Commit via hook

**Checkpoint**: US4 host leg green — stamps present, inventory closed, exclusions unreachable, asset census-clean. All four stories' host legs together are exercised by the audit suite (C-H1…C-H8) and the player/diag twins (C-H9).

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: the Q1/Q2 design-dir maintenance, 001/004 point amendments, host-brand faces, ADR registration, and the verify.sh gate wiring. Lands AFTER the token/earcon work (Q1: «executed after the token/earcon work lands»).

- [ ] T023 [P] DESIGN-SONA alignment + normalization (R1/Q1 — one coordinated task; auditee A1 #4 + #6 files): (1) the four rate rows align to the 004 contract grammar — Rate change = Unison (RateStep) tick 880 Hz / 60 ms detached from the Allegro skip row — in `DESIGN-SONA.md` §6 (`:60,:85`), `tokens.md` §6, `components.md` §5, `accessibility.md` §2; (2) the failure row records the shipped E4 ≈330 Hz square + minor-2nd ≈450 ms (D3 kept as design origin, marked superseded for pitch); (3) ONE canonical banned green `#5AD6A8` — the `#5FD0A0` mention becomes a recorded erratum; (4) ALL remaining off-set hexes in the auditee dir are reconciled so the census passes: the non-audited token rows (`tokens.md` §2 Branco `#FFFFFF` row, §3 `--signal-flow`/`--signal-info #3D8BE8`/`Dim #C9CDD2` rows; the celeste row → **RESERVED vacancy** per C2; the harp row → positional rate dot; the etwahl rows → focus ring per E1) end up ONLY as `ERRATA:`-marked provenance lines, each naming its decision (C1 fill-token, C2 celeste, E1 etwahl, D1 typography/spacing non-binding); (5) the stale etwahl-progress rows fixed at `tokens.md:16/:28/:47/:65`, `components.md:17`, `DESIGN-SONA.md:60/:85`, `preview.html:238`; (6) the yellow-dot «Notificação (ponto pulsante)» erratum (`tokens.md:48`, `components.md:18`); (7) normalization: single declared version `v2.2.0` in every audited header (+ `accessibility.md`), NEW `DESIGN-SONA/CHANGELOG.md` (the batch-Q1 entry, listing errata) + `DESIGN-SONA/VERSION`; `git tag design-sona-v2.2.0` on the alignment commit; (8) repoint the spec's Assumptions pin line to «homologado e fixado na revisão etiquetada `design-sona-v2.2.0`» (single `specs/005-adopt-design-sona/spec.md` edit, recorded in the commit). GREEN of T014's dir half. Commit via hook per file (8 files)
- [ ] T024 [P] Point amendments to 001/004 (hybrid approach — Assumptions #8): in `specs/004-ressoa-dogfood-build/contracts/surface.md` correct the prohibited green to the canonical `#5AD6A8` (the `#5FD0A0` erratum recorded, name-to-value coherence — batch Q1 item (3)) and the «progress white» clause to the `--sona-voice` fill token (C1; the maintainer's veto that would re-admit `#FFFFFF` is recorded as NOT applied); in `specs/004-ressoa-dogfood-build/spec.md` amend SC-004 + the T051-04 protocol text to the B1 split — salto-frente/salto-trás as SEPARATE confirmation types (7-type protocol, ≥15 trials/type, no two equal consecutively, no in-session feedback, 5–10% traps, familiarization WITH feedback, confusion-matrix CSV artifact + direction-of-skip sub-score, ≥24 h replication, rapid-sequence sub-test, haptic-disposition line — `contracts/earcon-mapping.md` E5/E6); each amended paragraph carries a cross-reference to `specs/005-adopt-design-sona/`. No other 001/004 text moves. Commit via hook per file
- [ ] T025 [P] Ressonador Canônico host faces (FR-004; SC-006 repo half): the seal lives ONLY in host-facing artifacts — (i) the human CLI banner on **stderr** of the engine-diagnostics face (`crates/diag/src/cli.rs` human-formatter path; human banners never enter data stdout; text banner only — no binary asset dependency); (ii) `README.md` references the artifact `DESIGN-SONA/assets/logo/seal-engine-ressonador.svg`; (iii) a version-metadata note beside `DESIGN-SONA/VERSION` (created in T023). Never on the notification/lock-screen/in-app surfaces (enforced by T014/T021). Commit via hook per file
- [ ] T026 [P] ADR registration (Assumptions #15): create `docs/adr/adr-002-earcon-at-focus-loss.md` — the formal record that the Ostinato/Sforzando emission AT the focus-loss state-change (same call as the pause) follows ADR-001's deliberate platform deviation (constitution Tauri Platform Contract, Audio Focus / Driving-Safety); cite `research/clarify-b2-earcon-failure-degradation.md` + the ISO 15006 short-term-signal class; record the F4 contract (`reason_ttl_ms = null` — failure latching until state end; cleanup ONLY by new document/session/app-restart). Follow the `docs/*.md` heading pattern. Commit via hook
- [ ] T027 Wire the surface-audit gate (R13/T157 precedent): in `scripts/verify.sh` add a fail-closed `surface-audit` gate step AFTER the gate-2 test step: `cargo nextest run --workspace -E 'test(surface_audit)'` plus the C-H9 player/diag twins (`-E 'test(earcon)'`) — every audit function is named with the `surface_audit_` prefix so the filter is deterministic; the H8 DEFERRED branch is spelled out (a missing BUILT bundle ⇒ reports «DEFERRED (owner=device-build)» exactly like gate-8-full posture and still fails on ANY other named breach; never silently passes); a missing auditee/missing test target fails NAMED (A2: check id + file + line); inverting a check for negative-control runs is prohibited outside a temp copy. Then two-way sync `scripts/expected-tests.txt`: add every new target line (`src-tauri::surface_audit_*` family, `player::prop_earcon_burst_latest_wins`, `player::test_earcon_latency_oracle`, `player::test_earcon_truncation_diag`, `diag` additions) and run `cargo nextest list` vs the pinned file to prove gate-2's two-way diff still passes. Commit via hook per file

**Checkpoint**: design dir normalized and tagged; 001/004 contracts coherent with 005 decisions; verify.sh carries the surface-audit step deterministically.

---

## Phase 8: Verification (attestation-only — the ONLY tasks that attest success/completeness)

**Purpose**: run the deterministic gates and the device-leg trials. NOTHING else in this feature claims completeness. Mirrors 004's T051/T052 posture: human trial sheets and honest DEFERRED records — never silent passage.

- [ ] T028 Gate-9 build artifact: `cargo tauri android build --target aarch64` MUST produce a device artifact; `adb install -r <apk>` when the reference phone (Redmi 2201117TY, API 33) is attached; record the artifact path + gate-9 exit. Also re-run `scripts/check-manifest.sh` + `scripts/check-shell-purity.sh` (Tauri Platform Contract: no manifest/permission change; SC-035 Kotlin purity)
- [ ] T029 Full deterministic gate run: `scripts/verify.sh` (gates 1–8 + the new `surface-audit` step) MUST exit 0, with the exit code and step list recorded as the binding attestation. Spot-check filters per quickstart §1: `cargo nextest run -E 'test(chroma)' --workspace`, `-E 'test(inventory)' --workspace`, `-E 'test(earcon)' --workspace`
- [ ] T030 Device trials per quickstart §3 (10/10 sheets; owner=device-build; DEFERRED-with-a-REASON if the device is unavailable — plan.md: «device trials are the only non-host acceptance legs»): §3.1 SC-003 notification purity (marfim accent only; no body text/rate/voice/mark; warning = reason line + Ostinato + 3 s TTL); §3.2 SC-001 visual purity (determinate AND indeterminate; no third color; single column; no excluded screen reachable); §3.3 SC-002 blind-test protocol exactly as written (7 types ≥15 trials, traps 5–10%, confusion-matrix CSV artifact, risky pairs Allegro-trás×Andante / Unison×Allegro-frente, ≥24 h replication, pilot buzz-check + haptic-disposition line mandatory — E3/E6); §3.4 SC-004 mute trial; §3.5 SC-005 device latency ≤100 ms; §3.6 SC-006 Lente reduzida + Ressor absence; §3.7 SC-007 TalkBack announcements 10/10 + reduced-motion freeze leg (trigger recorded per F2: system toggle and/or `adb shell settings put global animator_duration_scale 0`; render-time forced class PROHIBITED). A systematic confusion = MEASURED evidence authorizing a 004-contract amendment — recorded, never a silent edit
- [ ] T031 Final privacy + scope glance (quickstart §4) + attestation record: `Cargo.lock` diff = UNTOUCHED (zero new dependencies — constitution IV); `diag scan` shows no new egress beyond 004's known set; the haptics-inert disposition held during §3.3 (E3); launcher icon unchanged (D4); update `.specify/memory/STATE.md` with the final resume point + gate evidence. The completion report cites T029's verify.sh exit + T028's artifact as the binding attestations; every DEFERRED leg is named explicitly with its owner (device-build)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: no dependencies — starts immediately; BLOCKS everything (T002 green-baseline is a hard stop gate)
- **Phase 2 (Foundational)**: depends on Phase 1; BLOCKS all user-story implementation
- **User-story phases (3–6)**: all depend on Phase 2. US1 and US2 are both P1 and run in PARALLEL after Phase 2 (disjoint file families). US3's RED wave (T014–T018) may START while US1/US2 implementations land — and T014–T018 must exist (RED) before T019 regardless; US4's T021 needs T015/T018 to exist (its inventory leans on the focus-ring/live-region twins) and must be shown FAILING before T022 regardless; its file is DISJOINT from the T014–T018 twins (T004's `surface_audit/inventory.rs` scaffold — no byte sharing), so the RED may land in Wave 3 or as Wave 4's first step — that owner then runs T019 → T020 → T022
- **Phase 7 (Polish)**: T023/T024 run after US2/US3/US4 implementations land (Q1 ordering); T025/T026 anytime after Phase 2; T027 strictly after ALL twins exist
- **Phase 8 (Verification)**: depends on ALL previous phases; the ONLY attestations

### Within-Story Order (TDD red-to-green)

- Every story: RED test tasks FIRST (all shown failing with NAMED reasons) → **GATE** → implementation tasks → GREEN checkpoint
- Models before services: the diag kind extension (T012) precedes its consumer (T013)
- Same-file edits (`status_view.html`: T019 → T020 → T022) run SEQUENTIALLY under one file-owning subagent — never parallel edits to one file; the audit twins are per-check child modules (one file per check, T004(e)) so separate agents never collide

## Parallel Execution (subagent waves)

```text
Wave 1 (after T001–T003) — Phase 2, one agent (same module + tauri.conf context):
  T004 scaffolding → T005 RED → T006 wiring      (strictly sequential)

Wave 2 (after T006) — P1 stories, two disjoint agents:
  Agent B (accent.rs census RED + MediaMirror.kt constants): T007 RED → T008   [US1]
  Agent C (crates/player + crates/diag): T009 RED, T010 RED, T011 RED
        → T012 (diag) then T013 (player, consumes the kinds)  [US2]

Wave 3 (after T006; overlaps Wave 2 by file-family) — one agent, five tasks
  (disjoint test files in one shared module — single context, restyle not required yet):
  T014/T015/T016/T017/T018 RED (src-tauri::surface_audit)      [US3]

Wave 4 (after Waves 2+3 twins are RED; exclusive file owner):
  Agent E (status_view.html + status_view.rs + surface_audit/inventory.rs):
    T021 RED (inventory) → T019 restyle → T020 aria → T022 stamps
    (sequential; the US4 RED T021 is shown failing BEFORE T022 lands)   [US3+US4]

Wave 5 (after Wave 4) — polish, four disjoint agents in parallel:
  T023 (DESIGN-SONA + tag) | T024 (specs/004) | T025 (README+diag CLI) | T026 (docs/adr)
  → then T027 (verify.sh + expected-tests.txt) — one agent, after all twins exist

Wave 6 (final attestation chain — strictly sequential):
  T028 → T029 → T030 → T031
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
Agent B: T007 → T008                                # accent census twin + MediaMirror.kt constants (US1)
Agent C: T009, T010, T011, T012, T013               # burst/oracle/diag kinds (US2)
```

---

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

### Test Coverage Map (quickstart §1 ↔ contract A2 naming)

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

---

## Notes

- **Source of precedence**: the check specification is `contracts/surface-audit.md` A1–A4; the audited sets are `data-model.md`; binding rules are `contracts/status-surface.md` S1–S7 + `contracts/earcon-mapping.md` E1–E7. When a task description and a contract disagree, the contract WINS — and the disagreement is a task-level bug to record in the commit summary, never a local code choice
- **A4 whitelist discipline**: the auditee list and the allowed hex set are DATA owned by the contracts; loosening them = coordinated contract edit (same discipline as closed sets), never a script tweak; a failure is never silenced by deleting an auditee or an assertion — resolve it in the audited file
- **fail-closed everywhere**: a missing auditee/bundle/ledger is a FAILED check (twins, T003 anchors, T008/T023 auditee list), never a skip
- **zero new dependencies** (constitution IV): every twin is pure Rust over auditee bytes; `proptest` is already in the workspace; `Cargo.lock` untouched is itself a Polish/Verification-checkable claim
- **gates stay the nine constitution gates** (R13): `surface-audit` is a verify.sh STEP, not a tenth gate — no amendment required
- **closed-set discipline** (§VII): `EventKind` closed at exactly eleven; `EarconKind` untouched (004 grammar owns pitches); the audited hex set stays 10 + ledger harp; every addition = coordinated edit + version bump + docs
- Commit every file edit via `scripts/commit-hook.sh` (rule 4); subagent waves pay each file closure immediately — the hook is the arbiter of the clean delta
- Stop at any checkpoint to validate the story independently; every story's host legs are runnable without a device

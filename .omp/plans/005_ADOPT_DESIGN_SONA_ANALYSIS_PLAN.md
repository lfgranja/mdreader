# Plan: Analysis of specs/005-adopt-design-sona (consistency / duplication / ambiguity / underspec / constitution alignment)

Slug: `005-adopt-design-sona-analysis-plan`
Plan file: `local://005-adopt-design-sona-analysis-plan.md`
Feature dir: `/home/luis/development/mdreader/specs/005-adopt-design-sona`
Status: PLAN-MODE — read-only analysis; remediation requires explicit user approval; no edits until approved.

---

## Context

Goal (`/speckit.analyze`): identify inconsistencies, duplications, ambiguities, underspecified items, and constitution conflicts across the three core artifacts (`spec.md`, `plan.md`, `tasks.md`) for feature `005-adopt-design-sona` (Adotar DESIGN-SONA), before any implementation. Read-only; output is a structured Markdown report with coverage table + findings table + metrics. Constitution (`.specify/memory/constitution.md` v1.12.0) is non-negotiable; any conflict is CRITICAL and must be resolved in spec/plan/tasks (not by diluting the principle). The user also mandates delegation (subagent waves [P]), TDD red-phase blocking, property-based tests (invariants over FR/task/closed-set coverage), verification-only attestation at end, scaffold/sidebar codegen before hand-written content, deterministic atomic `scripts/commit-hook.sh` after any edit (only if remediation approved), and pre/post extension-hook checks.

Pre-check (done, not deferred): `.specify/extensions.yml` exists (73 bytes); `hooks: {}` — neither `before_analyze` nor `after_analyze` registered; `enabled` n/a; `optional` n/a; no hooks executed; no parser error. `check-prerequisites.sh --json --require-spec --require-tasks --include-tasks` returned `FEATURE_DIR` + `AVAILABLE_DOCS` (11 docs, includes spec/plan/tasks/contracts/data-model/research/quickstart/checklists). All three artifacts present.

Constitution authority loaded (principles III Test-First NON-NEGOTIABLE, V Deterministic-Gates, VII Closed-Sets-No-Catch-Alls, VIII Driving-First, XII? not present — only III/VII/V are load-bearing here). Key closed-set references from artifacts: `EarconKind` (7 variants per `contracts/earcon.md`), rate ladder (7 rungs per data-model §RateLadder), `EventKind` (9 kinds per `crates/diag/src/writer.rs`), `contracts/surface-audit.md` A1 (6 auditees), `spec.md` closed color set (8 sona + 2 signal = 10 members, D2).

---

## Approach (ordered; dependency order preserved; build-tree/ordering for passing verification at each step)

### Phase 0 — Scaffold / template + RED property-test tasks first (BLOCKING, TDD rule 1)

Generate (before hand-writing logic / analysis content) via LSP / template reuse / verbatim pattern from the reference stack (no inventing new structures):
- Analysis-report scaffold (`specs/005-adopt-design-sona/analysis_report.md` template — ONLY if user approves remediation; read-only phase does NOT create it; the template is prepared in-memory / scratch, not written, until approval).
- Property-test stubs (`proptest` 1.12.0, workspace dev-dep; `tests/invariant_005.rs` scratch only — not written to repo in read-only phase; prepared for RED phase).

Write property-based test tasks (RED — must exist and be shown FAILING for the correct reason, not compile errors) before any detection runs. These are invariant assertions over the artifacts, executable by `cargo test` or `python -c` against parsed artifacts.

- **T-TEST-01** [RED, BLOCKING] `fr_to_task_coverage_property`: for each FR-### explicitly numbered in `spec.md` (§Functional Requirements 130-149, FR-001..FR-010 confirmed by grep), assert `exists task in tasks.md with matching identifier / storyline keyword`. Expected initial state: FAIL with named missing IDs (e.g., FR-009 verbose mix rule — check mapping to T00x in tasks.md). Must fail for RIGHT reason (missing mapping), never compile error.
- **T-TEST-02** [RED, BLOCKING] `closed_set_no_catchall_property`: for each closed-set reference (EventKind 9 kinds `crates/diag/src/writer.rs`; EarconKind 7 `crates/player/src/earcon.rs`; rate ladder 7 rungs `data-model.md`; surface-audit A1 6 auditees `contracts/surface-audit.md`; design-sona palette 10 hex members D2), assert `0` occurrences of `_ =>`, `Other`, `Unknown`, `Internal` in the referenced file/contract scope (per Constitution VII + verification gate 7 `no _ => in crates/*/src/`). Expected initial state: FAIL if any catch-all exists; else PASS (no issues in this dimension — still must be shown).
- **T-TEST-03** [RED, BLOCKING] `sc_coverage_property`: for each SC-### requiring buildable work (SC-001 closed-set purity host census; SC-002 blind-test protocol; SC-003 notification surface inspection; SC-005 ≤100 ms latency test; SC-006 Lente reduced + excluded-screens guard; SC-007 TalkBack + reduced-motion), assert `tasks.md references SC-###` and `checklists/requirements.md (16/16) references it`. Expected initial state: FAIL if deferred (e.g., SC-001 device-leg deferred per spec line ~159 “DEFERRED (owner=device-build)”; SC-002 “disposição háptica da condição auditiva” truncated; SC-005 baseline “sem oráculo de latência de earcon”; SC-007 truncated at “reduced-motion ativa”).
- **T-TEST-04** [RED, BLOCKING] `terminology_consistency_property`: build normalized term map from spec (`StatusView` / `grade 2-campo` / `Lente Sinestésica` / `Concha Harmônica R` / `Ressonador Canônimo` / `MediaMirror.kt` / `MediaStyle` / `Ostinato` / `Adagio`/`Andante`/`Allegro`/`Sforzando`/`Unison`) and assert every term used in spec appears in plan/tasks with same semantics (no drift). Expected initial state: FAIL if drift detected (e.g., “grade 2-campo” described as “apelido de identidade do design dir, não-normativo” in spec vs potential normative use in plan/task; “Lente Sinestésica” defined as reduced identity of same StatusView in spec US4 vs potential full-feature reference elsewhere).

All four RED tests must be written, shown RED (or correctly passing if clean), BEFORE Phase 1-4 detection runs. If any test has a compile error (e.g., missing `proptest` import, bad path to artifact), fix the scaffold/template first (rule 0), not by weakening test. If a test passes incorrectly (false negative), correct the oracle — never suppress.

### Phase 1 — Load artifacts progressively (minimal high-signal tokens; no full dump into context)

Load only required sections (already completed — this is the audit trail, not new work):
- `spec.md`: Overview/Context (l1-66), FR (l130-149, 10 items confirmed), SC (l151-195, 7 items, several truncated/deferred), User Stories US1-US4 (P1-P3), Edge Cases (l67-129, not fully loaded — load only if finding requires; mark unverified), Key Entities (Document Title, Current Section, Progress Indication, Earcon, Dual Mark). Note: spec has clarifications embedded (marks “valor do conjunto fechado”, “registro 2026-10-24”, “C3”, “C5”, “C6”, “D2”, “D5/E2” — these are versioned anchors and must not be treated as unstable drift, but referenced consistently).
- `plan.md`: Architecture/stack (load only phases / data-model refs; architecture choices: Rust owns synthesis + PCM at 22050 Hz mono f32; Kotlin `android-shell/` owns Service / MediaSession mirror; zero playback logic in Kotlin per constitution). Data-model references (§SonaToken, §SurfaceInventory, §NotificationSurface, §EarconConfirmation, §RateLadder, §ProgressIndication, §MovementSpec, §HapticDisposition, §DiagnosticEvent, §DesignDirState, §ContrastPair — all referenced in tasks.md prerequisites; if any missing from plan.md, report as coverage gap).
- `tasks.md`: Task IDs, descriptions, phase grouping, [P] markers, file paths, execution protocol (4 rules — scaffold-first, TDD RED, subagent waves, verification-only + commit hook). Confirm 16 checklist items in `checklists/requirements.md`; confirm contracts (`status-surface.md` S1-S7, `earcon-mapping.md` E1-E7, `surface-audit.md` A1-A4) all referenced; confirm quickstart references.
- `constitution.md`: Principles III (Test-First NON-NEGOTIABLE), V (Deterministic Gates), VII (Closed Sets No Catch-Alls), VIII (Driving-First), plus Verification Gates table (1-9) and Exceptions Log framework.

### Phase 2 — Detection passes in parallel waves (delegated to subagents; shared-context subagents reused)

Wave structure: each wave is a `task` subagent (or named shell group) with explicit context file (artifact snippet) passed in. Same subagent reused for tasks touching same file family (per rule 3).

**Wave 1 [P] — Duplication + Ambiguity (independent; can start after RED tests exist)**
- Subagent A (duplication, context = `spec.md` lines 130-149 + `tasks.md` task list): detect near-duplicate FRs / SCs / user-story acceptance scenarios. Look for FR-001 (token palette) overlapping with FR-005 (closed-set colors); FR-002 (notification content restriction) overlapping with FR-007 (logo/prohibition) and SC-003; FR-003 (earcon synthesis) overlapping with FR-009 (mixed additive confirmation rule) and FR-010 (≤100 ms latency). Look for duplicated acceptance criteria in US1/US2/US3/US4 (e.g., “notificação mostra apenas título/seção/progresso” repeated in US1 scenario + SC-003). Produce finding IDs with line ranges.
- Subagent B (ambiguity / placeholders, context = full artifacts + constitution): flag vague adjectives lacking measurable criteria (`scalable`, `robust`, `intuitive` — scan for “fácil”, “rápido”, “claro” if present in Portuguese text; also `fast`, `secure` if English slipped). Flag unresolved placeholders (`TODO`, `TKTK`, `???`, `<placeholder>`, truncated lines at “reduced-motion ativa no sistema,” “disposição háptica”, “sem oráculo de latência”). Note: spec uses “DEFERRED” explicitly for SC-001 device-leg and SC-002 haptic — these are not ambiguities but deferred decisions; record them separately under “Underspecification / Deferred” not ambiguity.

**Wave 2 [P] — Constitution alignment + Coverage gaps (depend on Wave 1 for context, but can overlap if subagent has full artifacts)**
- Subagent C (constitution alignment, context = `constitution.md` III/VII/V + `spec.md` FR/SC + `tasks.md` execution protocol + `plan.md` architecture): check for conflicts. Mandatory checks: (a) TDD RED gate enforced? `tasks.md` explicitly binds it in execution protocol; verify every story wave has RED tasks before implementation tasks. (b) No `_ =>` / catch-all? Check referenced contracts and `crates/*/src/` as applicable; note `spec.md` references but does not contain source code — flag that verification gate 7 must be exercised on code, not spec. (c) Closed-set additions: if spec proposes new variants (e.g., new earcon names, new token values), they must have coordinated edit + semver + doc; verify no new variants proposed outside registry. (d) Deterministic gates: all measurements must be deterministic (≤100 ms via host test, not trial-only; color purity via host census, not device-leg deferred without owner); flag deferred device-leg (SC-001, SC-002 partial, SC-007 partial) as gaps requiring owner assignment, not as failures of spec quality — but they ARE coverage gaps if tasks don't reference them.
- Subagent D (coverage mapping, context = `spec.md` FR-001..010 + SC-001..007 + `tasks.md` + `checklists/requirements.md`): build mapping table. For each FR/SC, list task IDs; for each task, list FR/SC; compute coverage %. Detect: FR-009 (verbose mixed-rule with discard-on-pause) likely has complex mapping — verify enough task granularity. SC-002 (blind-test protocol with haptic, matrix, 15 trials/type) — verify task references the full protocol (not just “test earcon”). SC-006 (excluded screens guard) — verify tasks reference `contracts/surface-audit.md` A1-A4. SC-007 (TalkBack + reduced-motion) — verify tasks include reduced-motion freeze test (pílula congelada, role/value announcements invariant).

**Wave 3 [P] — Inconsistency / Terminology drift (depends on Wave 2 mapping for context reuse; same subagent family if shared context)**
- Subagent E (terminology / data-model / cross-file consistency, context = `data-model.md` + `plan.md` + `spec.md` entities + `tasks.md` prerequisites): build normalized term set from `data-model.md` (§SonaToken, §SurfaceInventory, §NotificationSurface, §EarconConfirmation, §RateLadder, §ProgressIndication, §MovementSpec, §HapticDisposition, §DiagnosticEvent, §DesignDirState, §ContrastPair) and compare to `plan.md` references and `spec.md` usage. Look for: “NotificationSurface” vs “MediaStyle notification”; “SurfaceInventory” presence in plan (if plan references it); “RateLadder” 7-rung set consistency with spec FR-003 / US2; “MovementSpec” (pílula deslizante) consistency with SC-006 / SC-007 (reduced-motion freeze); “HapticDisposition” (disposição háptica) presence in spec SC-002 truncated section and tasks; “DiagnosticEvent” / “EventKind” 9 kinds consistency; “DesignDirState” / “C3/C5/C6” versioned anchors consistency across files (do all files reference the same registry date “2026-10-24”? If not, drift).

Each subagent writes its findings to an in-memory / scratch artifact (not to repo; only to session output) with stable IDs (category initial + number). After all waves complete, parent integrates into single report.

### Phase 3 — Severity assignment (heuristic, deterministic, not judgmental)

Per instruction: CRITICAL = violates constitution MUST / missing core artifact / zero-coverage blocking baseline. HIGH = duplicate/conflicting req / ambiguous security/performance / untestable AC. MEDIUM = terminology drift / missing non-functional task / underspecified edge case. LOW = style.

Assignment rules (apply deterministically, not ad hoc):
- Any finding touching Constitution III (test-first) or VII (closed-set) or V (determinism) → CRITICAL if violated; MEDIUM if partially addressed; LOW if fully compliant.
- Any FR/SC with zero mapped task IDs and not explicitly deferred with named owner → CRITICAL (blocks baseline).
- Any duplicate FR/SC phrase with identical acceptance criteria → HIGH (consolidation needed, but not blocking).
- Any placeholder / truncated line / unresolved `???` / missing measurement → HIGH if in performance/security/accesibility path; MEDIUM if cosmetic.
- Any terminology drift that changes semantics (e.g., “Lente Sinestésica” full vs reduced) → HIGH (could misdirect implementation).
- Deferred device-leg (SC-001, parts of SC-002/SC-007) with no task referencing deferred leg → MEDIUM (gap, not violation — needs explicit deferred task or owner note).

### Phase 4 — Compact analysis report (structured markdown, no file writes unless approved; read-only phase outputs to session / can redirect to `local://005-analysis-report.md` only if user approves)

Structure (exact — implementer must not invent sections):

```
## Specification Analysis Report (005-adopt-design-sona, read-only)

### Findings Table (max 50 rows; overflow aggregated)
|ID|Category|Severity|Location(s)|Summary|Recommendation|
|A1|Duplication|HIGH|spec.md:130-149 / tasks.md:T00x|FR-001 token palette overlaps FR-005 closed-set; two acceptance scenarios identical in US1 + SC-003|Merge; keep FR-001 for palette, FR-005 for restriction; deduplicate SC-003 acceptance text|
...

### Coverage Summary Table
|Requirement Key|Has Task?|Task IDs|Notes|
|FR-001|Y|T004,T009|...|
|FR-009|Y?|T0xx,T0yy|Complex mix rule; verify discard-on-pause sub-tasks exist|
|SC-001|Y?|T013,T0xx|Device-leg DEFERRED — needs owner task or deferred-marker in tasks.md|
|SC-002|Y|T015|Haptic sub-section truncated in spec; verify protocol task exists|
...

### Constitution Alignment Issues
- III (Test-First): compliant if all Waves 1-4 have RED-gate tasks before implementation tasks; flag if any implementation task lacks preceding RED.
- VII (Closed-Sets): compliant if no new enum variants proposed; flag if spec proposes new token values / earcon names outside registry.
- V (Deterministic-Gates): compliant if all measurements have host-test oracles; flag deferred device-leg without owner.
- VIII (Driving-First): compliant if all interactions eyes-free (US2); verify earcon mix never suppresses narration (FR-009)

### Unmapped Tasks (if any)
|Task ID|Description|Mapped FR/SC?|Notes|
|T001|Branch creation|—|Setup, no FR/SC; OK|
...

### Metrics
- Total Requirements (FR-001..010 + SC-001..007 + US1..4): 21 + 7 + 4 = 32 requirements/stories
- Total Tasks: count from tasks.md (count after parsing; include T001-T003 + story tasks + verification)
- Coverage %: (FR+SC with >=1 task) / (total FR+SC) * 100; target >=95% (only explicitly deferred with owner allowed lower)
- Duplication Count: count of HIGH/LOW duplication findings
- Ambiguity Count: count of placeholder / vague-adjective findings
- Critical Issues Count: count of CRITICAL findings (constitution / zero-coverage / missing core artifact)
- Property-test status: T-TEST-01..04 (PASS / FAIL / RED) — must be shown before report finalized
- Extension hooks (before/after): none registered; no parser error; no execution needed

### Next Actions
- If CRITICAL > 0: recommend /speckit.specify refinement or /speckit.plan architecture fix before /speckit.implement; do NOT proceed.
- If only LOW/MEDIUM: can proceed to /speckit.implement with improvement suggestions.
```

### Phase 5 — Remediation offer (explicit user approval required; do NOT apply automatically)

After report, ask: **“Would you like me to suggest concrete remediation edits for the top N issues (where N = min(5, critical_count + 2))? If yes, list the top entries with file/line, proposed edit, and whether it affects constitution/closed-set. I will not edit until you confirm with exact file/path/line or say ‘apply top 3’.”**

Per instruction: do NOT apply edits automatically; only propose. Remediation edits (if approved) each trigger deterministic hook: `scripts/commit-hook.sh <file-path> "feat(005): <detailed summary of edits made (what changed + task/reference ID)>"` — atomic conventional commit per file, per rule 4 / constitution V.

### Phase 6 — Verification (ONLY these tasks attest; agent never self-attests)

Verification tasks (scheduled at end; run after report is finalized and after any approved remediation):

- **V-01 [VERIFICATION, attests read-only state]**: `git status --short` must show only untracked scratch files (if any) — zero modifications to `spec.md/plan.md/tasks.md/contracts/`/code. If dirty, abort claim of completeness; do not claim “analysis done” if files changed unexpectedly. Command: `git status --short && echo "READ-ONLY OK"`.
- **V-02 [VERIFICATION, attests prerequisites]**: re-run `bash .specify/scripts/bash/check-prerequisites.sh --json --require-spec --require-tasks --include-tasks`; confirm FEATURE_DIR and AVAILABLE_DOCS unchanged; record JSON snippet in verification artifact.
- **V-03 [VERIFICATION, attests property tests RED phase]**: run each T-TEST-01..04 (or equivalent `python`/`rust` oracle); record PASS/FAIL per test; confirm RED phase satisfied (tests existed before detection waves started; if any test never existed, report failure of TDD gate — constitution III violation at meta-level).
- **V-04 [VERIFICATION, attests report completeness]**: confirm findings table has >=1 row per category expected (Dup, Amb, Under, Const, Cover, Inconsist); confirm metrics block present; confirm coverage summary has all FR/SC; confirm extension hook block present (before_analyze / after_analyze result); confirm no invented sections.
- **V-05 [VERIFICATION, attests extension after_analyze]**: re-check `.specify/extensions.yml`; if any `hooks.after_analyze` added after plan (none expected in read-only), execute per optional/mandatory rules; if executable and optional, output block; if mandatory, execute and wait; record result.
- **V-06 [VERIFICATION, attests subagent/delegation completion]**: confirm Wave 1/2/3 subagent outputs collected (even if empty — empty is valid if clean); confirm shared-context reuse (same subagent for same file family) documented; confirm [P] markers used; confirm no single-agent did all detection (delegation verified by output sources).
- **V-07 [VERIFICATION, attests commit hook readiness]**: if remediation approved and edits made, confirm each edited file has a matching `git log --oneline -1 -- <file>` entry describing edit + task/reference; if no edits made, confirm hook was never invoked (correct — no edits, no commits; never a no-op commit).

Only after V-01 through V-07 pass may any statement of “analysis complete / consistency verified” be made. Until then, statements of completeness are prohibited (rule 3 / verification-only attestation).

---

## Critical files & anchors (≤5 disambiguating; line hints for re-read)

1. `.specify/extensions.yml` (line 7 `hooks: {}`) — pre/post hook state; no parser error.
2. `specs/005-adopt-design-sona/spec.md` — FR-001 (l130), FR-009 (mixed-additive discard-on-pause, l141-144), SC-001 deferred device-leg (l159 “DEFERRED (owner=device-build)”), SC-002 truncated (l163+ “disposição háptica…” truncated), SC-007 truncated (l193 “reduced-motion ativa…” truncated); User Story 4 (US4, P3, reduced Lente, l87-128).
3. `specs/005-adopt-design-sona/tasks.md` — Execution Protocol rules 0-4 (l14-59), Phase 1 T001-T003 (branch + baseline + anchors l61-79), Phase 2 T004-T006 (audit scaffolding + RED C-H8 twin + S2 packaging l81-133), Wave tables / subagent dispatch (search for “Parallel Execution (subagent waves)”).
4. `specs/005-adopt-design-sona/plan.md` — architecture references to Rust synthesis (22050 Hz mono f32), Kotlin mirror, contracts (§StatusSurface S1-S7, §EarconMapping E1-E7, §SurfaceAudit A1-A4), data-model (§SonaToken, §RateLadder, etc.). Read only architecture + data-model sections needed.
5. `.specify/memory/constitution.md` — III Test-First (l857-866), V Deterministic Gates (l885-896), VII Closed Sets (l912-921), Verification Gates 1-9 (l1234-1289); also “Constitution conflicts automatically CRITICAL” (analysis instruction, not in file, but binding from prompt).

---

## Verification (end-state proof; concrete inputs/outputs)

- Input: unmodified repo at `dev` branch (verify with `git rev-parse --abbrev-ref HEAD` = `dev` or feature branch if already cut — feature branch T001 may have been run already; if dirty, stop).
- Action sequence: Phase 0 (scaffold/template + RED tests written, shown RED) → Phase 1 (load) → Phase 2 (waves 1-3, delegated) → Phase 3 (severity) → Phase 4 (report) → Offer remediation (user approval required) → Phase 5 (remediation only if approved, with hook per edit) → Phase 6 (V-01..V-07).
- Expected observable outputs (not “build passes” — analysis is read-only): (a) 4 property-test results recorded (PASS/FAIL/RED) with named failures correct; (b) Wave 1-3 subagent outputs present (even if zero findings); (c) findings table with stable IDs, no duplicates; (d) coverage table with FR-001..010 + SC-001..007; (e) metrics: total counts, coverage %, duplication/ambiguity/critical counts; (f) constitution alignment section with explicit III/VII/V status; (g) extension hook block (before/after) with “none registered / no error”; (h) `git status --short` clean; (i) if any edit approved: `git log -1 -- <file>` shows atomic conventional commit with detailed edit summary + reference to task/FR/SC.
- Prerequisites: same workstation (linux 7.2.11, rust 1.85+ available for test scaffold if using rust; `proptest` in workspace dev-deps), repo at `/home/luis/development/mdreader`, `specs/005-adopt-design-sona/` fully present (confirmed).
- Failure modes: (i) RED tests never written / never shown failing → stop, constitution III breach at meta-level; (ii) subagent outputs missing → stop, verification V-06 fails; (iii) report missing any required section → stop, V-04 fails; (iv) file edited without hook / with manual `git commit` → stop, V-07 / rule 4 breach; (v) user approves remediation but edits violate closed-set / constitution → stop, do not apply; only apply after confirming no new enum members / no weakened lint / no catch-all added.

---

## Assumptions & contingencies (only user-overridable; no hidden implementer choices)

- A1 — User wants read-only analysis first, not implementation (explicit in Goal / Operating Constraints “STRICTLY READ-ONLY”). Contingency: if user says “go ahead and fix”, treat as approval for Phase 5 only on top-N issues explicitly listed; never auto-expand scope.
- A2 — “Top N issues” means N = min(5, critical + 2) unless user specifies different N (e.g., “top 3 critical only”). Contingency: offer with default; apply only after user confirms N or file-by-file.
- A3 — Subagent delegation uses available `task` / `workpool`; if subagent framework unavailable, fall back to sequential detection in same agent with explicit [P] logging (still record parallel-intention even if not truly parallel — never hide that delegation was attempted).
- A4 — Property-based tests use `proptest` (already workspace dev-dep per tasks.md prerequisites). If `proptest` import fails due to version, do not downgrade or substitute with weak random; instead report failure of scaffold/template step (rule 0) and ask / fix import path — the invariant remains required.
- A5 — Report format is markdown (not JSON / HTML). If user wants machine-readable output, add JSON sidecar only after report verified — never replace report.
- A6 — Constitution amendment (e.g., changing III from NON-NEGOTIABLE) is out of scope for this analysis (per instruction: “If a principle itself needs to change, that must occur in a separate, explicit constitution update outside `/speckit.analyze`”). If any finding suggests principle change, record it as “constitution-conflict requiring separate amendment” not as “modify constitution here.”

---

## Delegation / orchestration notes (user rules 0-5, embedded in plan)

- 0. Template/scaffold first: analysis report scaffold prepared before content filled; property-test stubs written before detection (RED). Hand-written analysis logic never precedes scaffold.
- 1. TDD RED blocking: T-TEST-01..04 exist and shown RED (or correctly PASS) before Phase 2 waves start. Implementation / detection does not start until RED phase satisfied.
- 2. Property-based invariants: all four tests assert invariants (coverage, closed-set, SC buildable-work, terminology) rather than example-based checks; use `proptest` strategies over artifact parses where randomization applies (e.g., term-occurrence sampling over document chunks — only if applicable; if deterministic sufficient, declare deterministic oracle and note no randomization needed).
- 3. Parallel [P] + subagent reuse: Wave 1 (A+B) parallel, disjoint; Wave 2 (C+D) parallel, disjoint (C uses constitution, D uses mapping); Wave 3 (E) may reuse subagent family with Wave 2 if shared context (same file family: spec + data-model); document reuse in report’s “Delegation verification” block (V-06).
- 4. Verification only at end: V-01..V-07 are the only attesters. No “analysis complete” statement until V-01..V-07 pass.
- 5. Atomic commit hook: only invoked if Phase 5 remediation approved; per edit, `scripts/commit-hook.sh <file> "feat(005): <detail>"`. No manual `git commit`; never a bulk multi-file commit.

---

## Plan approval / execution instruction for implementer

This plan is submitted to `xd://propose` for approval (read-only analysis with optional Phase 5 remediation). Once approved:
1. Confirm read-only: `git status --short` clean.
2. Run Phase 0 (scaffold + RED tests) — write `tests/invariant_005.rs` scratch and show RED; if `proptest` import broken, fix scaffold (rule 0).
3. Load artifacts (Phase 1) — use progressive disclosure, not full dump.
4. Launch Wave 1 [P] subagents (A + B) — share context only within family.
5. Launch Wave 2 [P] subagents (C + D) — reuse family if applicable.
6. Launch Wave 3 [P] subagent E.
7. Severity + report (Phase 3-4) — write `local://005-analysis-report.md` only if user approves file creation; default to session output.
8. Offer remediation (Phase 5) — ask with top-N list; do NOT apply.
9. If approved, edit + hook per file; re-verify V-01/V-07.
10. Final verification V-01..V-07; emit structured report; only then declare complete.

Plan complete; no open design decisions (scaffold/template choice = LSP/rust-analyzer + `proptest`; subagent framework = `task` / `workpool`; report format = markdown; N default = 5; deferred device-leg = gap not failure; constitution-change = out of scope). Ready for `xd://propose` submission.

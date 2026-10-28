# Convergence Plan — 005-adopt-design-sona (speckit.converge)

Slug: convergence-005-plan.md
Task: Assess present state of `specs/005-adopt-design-sona/` artifacts vs. codebase; append remaining work to `tasks.md`. No design decisions — only diagnostic/append.

## Context
Plan-mode active (read-only until `xd://propose` approval). Feature branch `005-adopt-design-sona` tracks the DESIGN-SONA v2.2.0 adoption (Option A subset: notification accent-only + reduced Lente + earcon confirmation + token restyle on `status_view.html`). Constitution (§I–XI, Tauri Contract, Audio Pipeline) governs; `spec.md` = sole intent; `tasks.md` = source of task IDs.

## Approach (parallel waves — disjoint files only)

**Wave 1 (scout A — prerequisites + scaffolding)** — one agent, shared tooling (scripts/verify.sh): verify T003 anchors (backend.rs enqueue_earcon, earcon.rs, diag writer, status_view.html, status_view.rs, MediaMirror.kt, tauri.conf.json), T004 surface_audit.rs scaffold, T005 served-identity twin, T006 wiring. Outputs: pass/fail per anchor + byte-identity of `dist/index.html` ↔ `status_view.html`.

**Wave 2 (scout B — US1 notification + US2 earcon)** — disjoint from Wave 1 (crates/player vs src-tauri): verify T007/T008 MediaMirror.kt accent (`0xFFF2F0E6`), T009–T013 burst policy + diagnostic kinds (`EventKind` exactly 11, no `_` arm, no `Other`), latency oracle ≤100 ms, mute/dignity (FR-006) property test. Outputs: green/red per twin.

**Wave 3 (scout C — US3 status-view restyle)** — ONE file-space owner (`status_view.html`): verify T014 (census over both progress branches, 10 hex members, no jade `#5AD6A8`/erratum `#5FD0A0`, no `rgba()` tints, no `--sona-celeste` paint), T015 (exclusivity: `#8FD8E8` only `:focus-visible`; `#FFD400` only warning banner + dot; `#FF4D63` only failure; no co-render), T016 (movement: ≤1 continuous loop, reduced-motion guard freezes pill visibly, yellow pulse bounded by 3 s TTL, no pulse on failure, banner `slideIn` one-shot ≤200 ms), T017 (contrast calculator pure, ≥4.5:1 text / ≥3:1 non-text over void `#0B0F1A` and stage `#141B2D`), T018 (aria inventory: `#progress` `role="progressbar"` + `aria-valuenow`/`aria-valuetext`; hidden `role="status"` live region; `#reasonBanner` `role="status"`; no `alert`+`polite`; no painted numeric rate). Also T019 restyle applied (token vars in `:root`, no old `rgba()` tints, `#1e293b`/`#0B0C15` backgrounds replaced?). Outputs: census/inventory/movement/contrast/aria findings.

**Wave 4 (scout D — US4 reduced Lente + polish)** — disjoint (design-dir + contracts): verify T021 inventory (exact §SurfaceInventory, dead removals structural not `display:none`, no excluded screens reachable), T022 stamps (`Concha Harmónica "R"` header bind, `--sona-stage` metadata card, value-blind rate indicator positional only, upstream authority `metadata_title.unwrap_or(filename)` preserved, empty slot glyph `—` + `aria-label="Sem título"`, failure banner static red `--signal-failure` `reason_ttl_ms=null` one-shot ≤200 ms not timer-based), T023 design-dir alignment (rate rows match 004 grammar `Unison`; failure D3→E4 ≈330 Hz documented; banned green `#5AD6A8`; remaining off-set hexes reconciled; `CHANGELOG.md` + `VERSION` + tag `design-sona-v2.2.0`), T024 001/004 point amendments (green prohibition, progress `--sona-voice`), T025 Ressonador host faces (`README.md`, CLI stderr banner, `VERSION` note — never notification/lock-screen/in-app). Outputs: inventory/alignment/host-face findings.

**Wave 5 (scout E — verification / gates)** — one agent, shared with Wave 1 (verify.sh): verify T027 gate wiring (surface-audit step after gate-2, `-E 'test(surface_audit)'` + C-H9 twins, fail-closed on missing bundle, H8 deferred branch written exactly), T028 gate-9 build (`cargo tauri android build --target aarch64` produces `aarch64` artifact), T029 full deterministic gate (`scripts/verify.sh` exits 0, step list recorded), T030 device trials (10/10 per `quickstart.md` §3 — SC-003 notification purity, SC-001 visual, SC-002 blind-test protocol ≥15 trials/type 7 types, confusion-matrix CSV, 5–10% traps, ≥24 h replication, SC-004 mute, SC-005 dev, SC-006 Lente; record DEFERRED-with-reason if device unavailable — Redmi 2201117TY API 33). Outputs: gate exit + device trial sheet status.

**Serialization rule (shared-tooling queue)**: Only Wave 1 and Wave 5 share `scripts/verify.sh` (single queue). Waves 2–4 never call verify.sh; each uses its own file set (crates/player, src-tauri/src/status_view.html, design-dir/). No cross-agent interruption of long-running commands.

## Critical files & anchors
- `specs/005-adopt-design-sona/spec.md` FR-001..FR-012 + SC-003/SC-006 + edge cases F1–F4 (intent source)
- `specs/005-adopt-design-sona/plan.md` phases 1–8 + architecture/stack + data-model (execution spec)
- `specs/005-adopt-design-sona/tasks.md` T001–T030 (task identity; max task ID used for new append)
- `specs/005-adopt-design-sona/contracts/` (surface-audit.md A1–A4, status-surface.md S1–S7, earcon-mapping.md E1–E7) — contract wins over task description
- `.specify/memory/constitution.md` §I–XI, Tauri Platform Contract, Audio Pipeline — highest-severity violations first
- Source anchors: `src-tauri/src/status_view.html` (token restyle), `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` `:728-743` (accent constant), `crates/player/src/backend.rs` `enqueue_earcon` (burst truncation + typed diag events), `crates/diag/src/writer.rs` `EventKind` (11 variants), `scripts/verify.sh` (surface-audit step)
- Design-dir anchors: `DESIGN-SONA/tokens.md`, `components.md`, `DESIGN-SONA.md`, `preview.html`, `CHANGELOG.md`, `VERSION`, `assets/logo/logo-primary-concha-r.svg` (T023 auditee #4 + #6)

## Verification (end-to-end proof — only Wave 5 attest)
After all waves: `bash .specify/scripts/bash/check-prerequisites.sh --json --require-spec --require-tasks --include-tasks` → FEATURE_DIR + AVAILABLE_DOCS. Then `scripts/verify.sh` gate; `cargo nextest run --workspace -E 'test(surface_audit)'`; `cargo tauri android build --target aarch64` artifact path recorded; if reference device unavailable, T030 records `DEFERRED (owner=device-build; device Redmi 2201117TY unavailable)` — never silent pass.

Success criteria: all findings have one checklist item in new Phase-N: Convergence section of `tasks.md`; zero findings = byte-for-byte unchanged `tasks.md` (converged); every finding ordered CRITICAL/HIGH/MEDIUM; constitution-violation tasks emitted first with `CRITICAL`; no existing task IDs reused; no edits to `spec.md`/`plan.md`; commit per edited file via hook.

## Assumptions & contingencies
- A1 (verified): `status_view.html` already carries tokens (`:root` vars present) — T019 restyle largely done; gap is audit twins (C-H1..C-H9), not token declaration.
- A2 (verified): `MediaMirror.kt` `DESIGN_SONA_PROGRESS_FILL = 0xFFF2F0E6` — T008 done.
- A3 (unverified — confirm in Wave 3): `dist/index.html` is byte-identical to `status_view.html` (same 27405 bytes observed); need exact `cmp` proof for T006.
- A4 (contingency): If Wave 3 finds `rgba()` tints or old slate backgrounds still present in `status_view.html`, treat as T019 partial — append remediation; do not edit file (convergence does not edit code).
- A5 (contingency): If device unavailable for Wave 5, T030 records deferred with reason; gate passes only if all host checks green and deferred is explicitly documented.
- A6 (load-bearing): No new dependencies (§IV); if `proptest` missing from workspace, T009/T010 skip with named failure — never add crate.

## Delegation execution (this turn)
Launch Waves 1–5 as named `task` subagents (parallel where disjoint, queued where shared tool `verify.sh`). Wave 5 waits for Wave 1's verify step + Waves 2–4 file assessments. Each subagent writes its finding summary to `agent://<id>`; parent integrates and writes `local://convergence-005-plan.md` (this file) + submits `xd://propose`, then — on approval — executes integration + appends to `tasks.md`.

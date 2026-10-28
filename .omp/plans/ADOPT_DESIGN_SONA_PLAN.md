# Plan: adopt-design-sona (specs/005-adopt-design-sona/)

Slug: `adopt-design-sona`. Title: "Adotar o DESIGN-SONA — wave-by-wave subagent execution". Plan file: `local://adopt-design-sona-plan.md`. Submitted via `xd://propose`; approval restores full write.

## Context (literal ask)

Implement feature `005-adopt-design-sona` (spec at `/home/luis/development/mdreader/specs/005-adopt-design-sona/`). End state: MediaStyle notification paints ONLY the `--sona-voice` accent (`#F2F0E6`, closed) with title + current-section + progress; no body/rate/voice/mark; warning = static reason line + Ostinato; earcon controls confirm all 7 response types under SHIPPED 004 grammar; in-app surface uses DESIGN-SONA tokens; verification attested only by Phase 8. Rules bind per `tasks.md` §"Execution Protocol": TDD (RED FAIL before GREEN), [P] parallel to max viable subagents, shared-tool serialization, verification ONLY at end, atomic conventional commit per edit, never interrupt another agent's tool.

Plan mode is active; read-only until approval via `xd://propose`; this file is the complete execution spec — zero design decisions left to implementer.

## Approach (ordered, decision-complete)

Phase 1 — Pre-flight (sequential, this agent, no subagent yet)
1. Read `.specify/extensions.yml`: `hooks: {}` → skip hook block (no executable hooks). Confirmed.
2. Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`: FEATURE_DIR=`specs/005-adopt-design-sona`; AVAILABLE_DOCS=`spec.md plan.md research.md data-model.md contracts/ quickstart.md tasks.md`. Confirmed.
3. Checklists: `005-remediation-checklist.md` (0 total, 0 unchecked), `requirements.md` (16/16 checked). Overall PASS. Confirmed, proceed.
4. Ignore-file check (repo is Rust + Kotlin): verify `.gitignore` contains `target/` `*.class`; create/append `.dockerignore`, `.eslintignore` as needed. Step completes; commit `chore: verify ignore patterns for rust/kotlin`.
5. Load context: `plan.md` (tech stack: Tauri 2.12.1, Rust 1.85+, Kotlin Android shell), `data-model.md` (notification surface, earcon, session models), `contracts/` (`earcon-mapping.md`, `status-surface.md`, `surface-audit.md`), `tasks.md` (phases 1–8, TDD per story, [P] markers, verification-only Phase 8). Confirmed; commit `docs: load 005-adopt-design-sona spec context`.

Phase 2 — Wave 0: Test scaffolding (TDD RED first) — disjoint files, parallel subagents allowed
- Delegate to subagent `scout-us1` (context only, read-only): read `spec.md` §"Identidade visual/sonora" + `contracts/surface-audit.md` + `tasks.md` US1 test section. Deliver: target constants block `MediaMirror.kt` slice `:728-743`, expected `0xFFF2F0E6` only, structural invariants (no mark/logo/rate/body-text constants). No writes.
- Delegate to subagent `scout-us2` (read-only): read `contracts/earcon-mapping.md` + `tasks.md` US2 test section. Deliver: 7 confirmation-type mapping (Adagio/Andante/Unison/Allegro-frente/Allegro-trás/Ostinato/Sforzando), SHIPPED 004 grammar rules (pitches never re-specified, burst = truncate/substitute latest-wins, interruption, ≤100ms first-consumed-sample, at-most-one-re-emission + typed diag).
- Delegate to subagent `scout-us3` (read-only): read `spec.md` §"Superfície in-app" + `data-model.md` token registry + `contracts/status-surface.md`. Deliver: token names (`--sona-voice`, `--signal-caution`, `--signal-failure`, etc.) and in-app surface fields.
- Shared-tool rule: only `scout-us1` touches `cargo check` if needed; others wait in queue (none expected — all read-only).
- All three must return before Wave 1 starts. Block if any stalls.
- Commit after each scout delivers: `docs: scout US1/US2/US3 spec fragments (wave 0)`.

Phase 3 — Wave 1: US1 RED tests + US2 RED tests (parallel, disjoint test files, TDD)
- Subagent `tdd-us1`: write `src-tauri/src/surface_audit/accent.rs` (scaffold from T004 — empty file confirmed by plan.md §"Project Structure"; if missing, create empty) with property-based/invariant test `surface_audit_accent_only_constants`: parse `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` slice `:728-743` (fixed `0xAARRGGBB` literal parser, NOT regex), assert set exactly `{"0xFFF2F0E6"}`; assert no mark/logo/rate/body-text constant. Must FAIL today (currently `DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF`). Directory verified; no other Kotlin change.
- Subagent `tdd-us2`: write `tests/earcon/earcon_burst.rs` (new) with `prop_earcon_burst_latest_wins` + `test_earcon_latency_oracle` + diag twins; and `tests/earcon/earcon_mapping.rs` asserting 7-type closed mapping matches `earcon-mapping.md`. Must FAIL before T012/T013.
- Dependency: both RED test files written before any GREEN edit. Block Wave 2 until both confirm FAIL (run `cargo test surface_audit_accent_only_constants` and `cargo test prop_earcon_burst_latest_wins`; expect non-zero exit).
- Serial shared tool: only one subagent runs `cargo test` at a time; other waits. Record wait explicitly.
- Commit after each RED file lands: `test(us1/us2): RED accent-only / earcon burst (tdd)`.

Phase 4 — Wave 2: US1 implementation (T008) + US2 implementation (T012/T013) — sequential per-file, can parallel if disjoint
- Subagent `impl-us1`: edit `android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` block `:728-743` ONLY: replace `DESIGN_SONA_PROGRESS_FILL = 0xFFFFFFFF` → `0xFFF2F0E6`; replace comment with `fill = --sona-voice — registro 2026-10-24, C1; contract "progress white" superseded`; retag KDoc to `--sona-voice`, `--signal-caution`, `--signal-failure`. NO manifest/permission change (D4 exclusion intact); no other Kotlin. Confirm GREEN (`cargo test surface_audit_accent_only_constants`). Atomic commit: `fix(us1): MediaMirror accent-only --sona-voice 0xFFF2F0E6 (T008)`.
- Subagent `impl-us2`: implement earcon emission per SHIPPED 004 grammar — burst truncate/substitute, interruption, emission-failure → action proceeds + at most one re-emission + typed diag event; ≤100ms first-consumed-sample. Files: `src/earcon/` (exact paths from `plan.md` §"Source Code" — verify `src/earcon/` exists; if not, create per plan). Confirm GREEN (`cargo test prop_earcon_burst_latest_wins`). Atomic commit: `feat(us2): earcon controls 7-type SHIPPED-004 (T012/T013)`.
- Serial shared tool: `cargo test` for impl-us1 runs; impl-us2 queues; then reverse.
- Block Wave 3 until both GREEN.

Phase 5 — Wave 3: US3 RED + US4 RED (parallel, disjoint)
- Subagent `tdd-us3`: write `src-tauri/src/surface/in_app.rs` or `tests/surface/in_app_tokens.rs` asserting surface uses only DESIGN-SONA tokens (`--sona-voice`, `--signal-caution`, `--signal-failure`); no off-brand tokens; structural: no body-text field, no rate field. Must FAIL.
- Subagent `tdd-us4`: write `tests/lente/lente_reduced.rs` (reduced Synesthetic Lens, P3) asserting reduced-mode invarants (only pre-defined token set visible, no extra dimensions). Must FAIL.
- Serial `cargo test`; commits `test(us3/us4): RED in-app tokens / reduced lens`.

Phase 6 — Wave 4: US3 implementation + US4 implementation (sequential per-file, disjoint)
- Subagent `impl-us3`: edit in-app surface files (from `plan.md` source tree); replace any non-canon token with `--sona-voice` / `--signal-caution` / `--signal-failure`; confirm GREEN. Commit `feat(us3): in-app surface DESIGN-SONA tokens (T019/T020)`.
- Subagent `impl-us4`: implement reduced lens (P3, lower priority — only after US1/US2/US3 verified). Confirm GREEN. Commit `feat(us4): lente sinestésica reduzida (T022)`.

Phase 7 — Wave 5: Cross-cutting / polish (Phase 7 of tasks.md)
- Subagent `polish`: address SC-003 (device inspection ≤3 s play-to-audio), SC-004 (≤100ms host), SC-005 (≤3 s), FR-006 mute, FR-007 no-logo structural, T011/T030. File-level edits only; shared `cargo test` serial. Commit per edit: `polish: SC-003/004/005 + FR-006/007 (T011/T030)`.

Phase 8 — Verification (attestation ONLY — never before this phase)
- Subagent `verify-us1`: device inspection of quickstart §3.1 (10/10 sheets; MediaStyle shows only title + section + progress; accent `#F2F0E6`; no body/rate/voice/mark). Output: pass/fail per sheet.
- Subagent `verify-us2`: quickstart §3.3 blind-test protocol — 7 confirmation types confirm by earcon; burst/latest-wins; interruption; ≤100ms. Output: pass/fail per type.
- Subagent `verify-us3`: quickstart §3.4 / §3.5 — surface tokens match contract; no off-brand; structural invariants.
- Subagent `verify-us4`: Phase 8 verification tasks V001–V006 (from `tasks.md` §8 / `plan.md` §"Verification Subsection") — run `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo doc --workspace --no-deps`, `cargo deny check`, `cargo audit`; device-build artifact (`tauri android build --target aarch64`). Only these tasks attest success — implementer MUST NOT claim completion from earlier phases.
- If ANY verification fails: stop, record failure in `specs/005-adopt-design-sona/checklists/005-remediation-checklist.md` or new `specs/005-adopt-design-sona/verification-fail.md`, do NOT proceed to "done". If all pass: commit `verify: Phase 8 attestation — US1-US4 + V001-V006 (all green)`.

## Critical files & anchors (≤5, non-obvious)

- `/home/luis/development/mdreader/specs/005-adopt-design-sona/tasks.md` — phases 1–8, TDD rules, [P] rules, verification-only Phase 8. Line ranges: execution protocol `12-21`; US1 test `74-87`; US2 test `94-110`; US3 `118-135`; US4 `143-153`; Phase 8 `169-178`; dependencies `180-194`; parallel examples `196-242`.
- `/home/luis/development/mdreader/specs/005-adopt-design-sona/contracts/earcon-mapping.md` — 7-type closed mapping, SHIPPED 004 grammar (no pitch re-spec, burst, interruption, diag, ≤100ms). Read full.
- `/home/luis/development/mdreader/specs/005-adopt-design-sona/contracts/surface-audit.md` — `surface_audit_accent_only_constants` contract (C-H1/H3, C1 veto of white, structural invariants). Anchor for T007/T008.
- `/home/luis/development/mdreader/android-shell/src/main/kotlin/app/mdreader/MediaMirror.kt` — slice `:728-743` only; fixed literal parser `0xAARRGGBB`; current fault `0xFFFFFFFF`. Confirmed by plan.md / tasks.md.
- `/home/luis/development/mdreader/specs/005-adopt-design-sona/plan.md` §"Project Structure" — `src-tauri/src/` / `android-shell/src/` / `tests/` / `crates/` layout. Verify `src/earcon/` existence before US2 edit; if absent create per plan.

## Verification (end-state proof, only Phase 8 attests)

Concrete checks (run ONLY in Phase 8):
- `bash -c 'cd /home/luis/development/mdreader && cargo test surface_audit_accent_only_constants prop_earcon_burst_latest_wins test_earcon_latency_oracle'` → 0 failures, exit 0. (Requires T008 and T012/T013 green.)
- Device leg: `tauri android build --target aarch64` produces artifact; quickstart §3.1 sheets (10) inspected; accent `#F2F0E6`; no body/rate/voice/mark; warning static reason + Ostinato.
- Blind-test (quickstart §3.3): 7 confirmation types × 3 trials = 21 pass; burst latest-wins; interruption resumes; diag event written per emission failure; latency ≤100ms measured.
- `cargo clippy --workspace -- -D warnings` clean; `cargo deny check advisories licenses bans sources` clean; `cargo audit` clean.
- All edits have atomic conventional commit (`feat/test/fix/polish/verify:` prefix, file/line summary, no empty messages).
- If verification fails at any point, halt, document in `specs/005-adopt-design-sona/verification-fail.md` (new file permitted in plan mode for failure record; not cleanup), do NOT claim done.

## Assumptions & contingencies (user-overridable; record, don't decide in-place)

- A1 (load-bearing): Repo tech stack is Tauri 2.12.1 + Rust 1.85 (Ed 2024) + Kotlin Android + `redb`/`blake3` (per `AGENTS.md` and `plan.md`). If reality is different (e.g., repo is pure Kotlin with no Rust), fallback: stop Wave 2/4/6; record `specs/005-adopt-design-sona/assumption-mismatch.md`; ask user which crate to edit instead of inventing file paths.
- A2 (load-bearing): Subagent `scout-*` / `tdd-*` / `impl-*` / `verify-*` have access to `cargo`, `git`, and the repo root (same workspace). If a subagent lacks them (e.g., container without cargo), fallback: that subagent waits (shared-tool queue); the one agent with cargo executes the command, others observe output via file; never kill/interrupt.
- A3 (load-bearing): `MediaMirror.kt` block `:728-743` exists and uses literal `0xFFFFFFFF`; if file/line drifted (edit since spec), fallback: re-read file first, adjust slice to current `DESIGN_SONA_PROGRESS_FILL` occurrence, record new line in commit body (`line shifted from 728 to N`); do NOT assume fixed line.
- A4 (load-bearing): TDD order — tests MUST be written and MUST FAIL before implementation. If a RED test accidentally passes (e.g., file already fixed), fallback: check git status; if pre-fixed, record `pre-existing fix` in commit; skip RED-only step for that test but keep sequence.
- A5 (load-bearing): Verification tasks V001–V006 exist in `tasks.md`; if some are missing, fulfill the ones that exist; report missing to user (not invent).
- A6: Parallel subagent count capped at 3 concurrent disjoint-file agents (matching CPU cores and shared-tool serialization). If user wants more, they must approve; default = 3.
- No design decision deferred: wave structure, test-first, verification-only-at-end, atomic commits, shared-tool serialization — all fixed above. Implementer never invents a wave, never skips RED, never claims done before Phase 8, never kills another agent's tool.

## Execution order memo (for implementer, no decisions)

1. Plan approved → full write restored.
2. Phase 1 (this agent): prerequisites, hooks (none), checklists (PASS), ignore files, load context, commit.
3. Phase 2 (subagents scout-us1/us2/us3, read-only): deliver fragments; block on all three.
4. Phase 3 (subagents tdd-us1/us2, parallel): RED test files; block until both FAIL confirmed via `cargo test` (serial).
5. Phase 4 (subagents impl-us1/impl-us2, sequential per-file): T008 + T012/T013; block until both GREEN.
6. Phase 5 (subagents tdd-us3/us4): RED; block FAIL.
7. Phase 6 (subagents impl-us3/impl-us4): GREEN; block.
8. Phase 7 (polish): sequential; commit per edit.
9. Phase 8 (verify-us1/us2/us3/us4): attestation ONLY; if any fail → `verification-fail.md`; else `verify:` commit; then report to user with summary.
10. Every file edit gets atomic conventional commit (no batch commit across files). If edit touches `MediaMirror.kt`, message must include `slice 728-743` and literal `0xFFF2F0E6`. If edit touches `spec.md` / `plan.md` / `data-model.md`, only edit if spec error found — do not edit spec as part of implementation; spec edits are out-of-scope unless verification reveals contradiction.

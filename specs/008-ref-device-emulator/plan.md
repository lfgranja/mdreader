# Implementation Plan: VERIF-EMULATOR — Structural Verification Layer (Option A)

**Branch**: `005-adopt-design-sona` (designated; no branch rename required; feature tracks on current branch)  
**Date**: 2026-10-29  
**Spec**: `specs/008-ref-device-emulator/spec.md` (Draft; Option A selected; `go` verdict from `.specify/assessments/ref-device-emulator/decision.md` 2026-10-29; 6 clarifications resolved by user)  

**Input**: Assessment pipeline `ref-device-emulator` (`intake` → `research` → `problem` → `concept` → `decision`: `go`) + `specs/003-reference-device/plan.md` (140l, Draft) + `ROADMAP.md` line 27 / 111 / 112 (VERIF-EMULATOR layer for structural verification of 003; physical `gate-8-full` preserved; `SC-044` deferred).

---

## Summary

Deliver a documented structural-verification path (`VERIF-EMULATOR`) for `003-reference-device` using Android SDK AVD (API 29+ baseline), without substituting physical `gate-8-full`, without altering `verify.sh`, and without modifying `crates/*/src` or `src-tauri/src/`. The layer produces an attestation document (`VERIF-EMULATOR.md`) referencing verified framework contracts (manifest/service/notification/focus/diag/resume/rate) and explicitly listing exclusions (audio-output, `SC-009`, Bluetooth/car routing, 10/10 on-device). All artifacts live in `specs/008-ref-device-emulator/`; no new crate needed; `cleanroom-core` stays empty.

---

## Technical Context

**Language / Version**: N/A — documentation / test script only; no compilation required; `cargo check` deferred to T002 per `specs/003-reference-device/tasks.md`.

**Primary Dependencies**: None (no new crate, no `Cargo.toml` change, no `Cargo.lock` change). Uses existing `verify.sh` rules (physical gate only) and `AGENTS.md` governance (constitution v1.12.0).

**Storage / Data**: No database changes; no `redb` bookmark; no new `blake3` fingerprint; attestation is document (markdown) stored in feature directory.

**Test / Verification**: Document-consistency checks (grep for exclusions, verification of `VERIF-EMULATOR.md` references); no `cargo test` required (no source code); structural checks observable via AVD framework APIs / `adb logcat` / AVD extended controls.

**Target Platform**: Android SDK AVD (API 29+) on maintainer workstation or isolated CI container; physical reference device (`API 29+`, e.g., Redmi referenced in `003`) remains required for `gate-8-full`.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I (Library-First)**: PASS — no crate created, removed, or repurposed; no organiser-only artifact.
- **II (CLI-First)**: PASS — no new CLI surface (attestation is document, not binary).
- **III (Test-First)**: PASS — verification is doc-check + structural framework observation; `tests/` for 003 remain out of scope for this feature; attestation is verified by review, not unit test.
- **IV (Latest Stable / Supply-Chain)**: PASS — no dependency changes; `vendor/` untouched; `deny.toml` unchanged; `Tauri` `2.12.1` unchanged.
- **V (Deterministic Gates)**: PASS — checks are `grep` / document review / manual AVD observation; no judgment input required for structural layer; no `unwrap/expect` added; existing `expect` only in `crates/*/src` unchanged.
- **VI (Engine-Neutral Core)**: PASS — no adapter / trait / engine change; `engine-core` closed-set untouched.
- **VII (Closed Sets)**: PASS — no enum added; no `Other` / `Unknown`; no `_ =>`; `EventKind` 9 variants unchanged; `EngineError` 7 variants unchanged.
- **VIII (Driving-First)**: PASS — meaningful for driving verification (structural evidence for `003`) without altering interaction surfaces.
- **IX (Offline-First)**: PASS — no network required for structural verification; AVD runs locally; attestation document does not transmit.
- **X (Free-Tier Budgeted)**: PASS — no quota-affecting change; AVD is free SDK; no external service cost.
- **XI (Credential / Content Sovereignty)**: PASS — no secret, no `.env`, no `.pem`; no telemetry; attestation stays local; no external endpoint.

No violations. No Complexity Tracking entries required.

---

## Project Structure

```text
specs/008-ref-device-emulator/
├── plan.md              # This file
├── spec.md              # Feature specification (Draft)
├── data-model.md        # To be created (Phase 1) — attestation document model + structural-state references
├── contracts/           # To be created (Phase 1) — reference contract to 003 contracts / manifest / service / diag
├── quickstart.md        # To be created (Phase 1) — structural verification quickstart
├── checklists/          # Requirements checklist (already exists; validated)
│   └── requirements.md
└── verification/        # To be created (Phase 1) — VERIF-EMULATOR.md (attestation) output directory
```

Note: This feature delivers documentation and verification scripts only; it does not alter `crates/`, `src-tauri/`, `android-shell/`, or source. The `IMPL_PLAN` reflects this by stating Phase 1 artifacts (docs/contracts/quickstart) and Phase 2 (validation / attestation), with no `tasks.md` or `implementation/` phase for source code.

---

## Phase 0: Research (already complete)

- `research.md` (`/speckit.assess.research`) completed; 6 gaps resolved by user (G1–G6 all Option A); `decision.md` upgraded to `go`.
- Evidence: AVD is official SDK (`developer.android.com`); emulator has framework limits (no Bluetooth, no real audio output, no telephony/interruption simulation); structural verification is possible; physical device required for `gate-8-full`.
- Key finding: `VERIF-EMULATOR.md` should reference existing contracts (`status-surface`, `earcon-mapping`, `surface-audit` from `005-adopt-design-sona`) and `003-reference-device/data-model.md`; must not claim equivalence.

---

## Phase 1: Design (current)

### 1.1 Data Model

No new database entities; verification attestation is document-level. Existing references:

- `VERIF-EMULATOR` attestation document (markdown): records verified framework contracts, exclusions, AVD version, branch/commit, sign-off path.
- `003-reference-device` structural-state: existing `data-model.md` entities (`§SurfaceInventory`, `§NotificationSurface`, etc.) verified by structural layer; no new entities.
- `blake3` fingerprint (`session` crate): existing mechanism for resume; unchanged.

### 1.2 Contracts

Reference contract to `003-reference-device` contracts (existing):
- `contracts/status-surface.md` (S1–S7): notification surface rules (title + section + progress only; no body/rate/voice; determinate/indeterminate based on total known).
- `contracts/earcon-mapping.md` (E1–E7): earcon confirmations (structural verification confirms framework provides earcon hooks; does not verify audio rendering).
- `contracts/surface-audit.md` (A1–A4): inventory / audit rules.

No new contract needed; `VERIF-EMULATOR.md` references these by path.

### 1.3 Quickstart

`quickstart.md`: How a maintainer runs structural verification locally.

```markdown
1. Prerequisites: Android SDK AVD (API 29), `mdreader` build (branch 005 or feature), `adb` available.
2. Launch AVD; install build.
3. Open `003-reference-device/plan.md`; confirm structural checklist items (manifest/service/notification/focus/diag/resume/rate).
4. Run structural observation (manual or scripted via `tests/` stub if defined); record in `VERIF-EMULATOR.md`.
5. Confirm exclusions explicitly listed; reference `verify.sh` for physical gate items; do NOT claim `gate-8-full`.
```

---

## Phase 2: Tasks / Implementation (future; not this plan)

Per `spec.md`, `tasks.md` is a Phase 2 output (`/speckit.tasks`). Not created here. Implementation tasks (when `go` confirmed and T002 passes):
- Define `VERIF-EMULATOR.md` template and exclusions (reference `AGENTS.md` / `spec.md` / `problem.md`).
- Create `tests/` stub for structural observation (optional; if added, must not claim gate-8-full).
- Document CI isolation instruction (isolated AVD / container; not in dirty `005-adopt-design-sona` working tree).
- Update `ROADMAP.md` (already done: line 27 / 111 / 112).
- Record attestation with `decision.md` reference.

---

## Verification

- [ ] `spec.md` exists at `specs/008-ref-device-emulator/spec.md` (Draft; Option A selected)
- [ ] Checklists pass (`checklists/requirements.md` validated)
- [ ] No `before_plan` hook required (`hooks: {}`); no mandatory hook missed
- [ ] Constitution check: all gates pass (no source edit; docs-only; no new enum/crate/dependency)
- [ ] `VE` boundary preserved (`VERIF-EMULATOR` ≠ `gate-8-full`); `SC-044` deferred; physical device required
- [ ] `cargo check` deferred to T002; not claimed PASS at this stage

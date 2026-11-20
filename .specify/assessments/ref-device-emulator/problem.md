# Problem Definition: Reference-Device Verification Gap (003)

- **Slug**: ref-device-emulator
- **Created**: 2026-10-29
- **Inputs used**: intake.md (2026-10-29); research.md (2026-10-29, medium confidence); user input (research request); internal artifacts (`specs/003-reference-device/`, `ROADMAP.md` #9267, `.specify/memory/constitution.md` v1.12.0, `AGENTS.md`)

## Problem Statement

`003-reference-device` has a fully-specified pipeline (`spec.md`, `plan.md`, `tasks.md`, `checklists/`, `contracts/`, `data-model.md`) but **zero verified implementation or convergence**: no `tests/`, no `VERIFICATION.md`, no `verify.sh` PASS on the current `005-adopt-design-sona` branch (`0c985ee`), no confirmed `cargo check`, `cleanroom-core` empty (`0 .rs`), and `gate-8-full` remains `DEFERRED`. The only verified green build is historical (`004` at `9501bba`, 2026-10-22). The verification path depends entirely on a **physical Android device** (`API 29+`, `adb devices`), which creates a hard bottleneck for structural validation of `001` (`gate-8-full`) and for any future `003` attestation — and there is currently **no documented structural-verification layer** that can run locally/free without physical hardware, despite the Android SDK offering a fully capable emulator.

## Affected Users & Stakeholders

- **Users (primary)**: mdreader spec-pipeline maintainers / verification owners — anyone responsible for confirming `001` `gate-8-full`, `003` device conformance, and `004` dogfood reproducibility. They experience the bottleneck as: `verify.sh` cannot be run reliably on the current working tree (interrupted `bg_14`, 51 dirty files, interrupted `cargo check`); only historical `9501bba` is green; physical-device access is required for every structural check.
- **Users (secondary)**: `005-adopt-design-sona` implementers — the active branch (`005-adopt-design-sona`) needs `003` structural proof to unblock `001` `gate-8-full`; the `tasks.md` protocol (TDD RED, subagent waves, `commit-hook.sh`) assumes a verifiable baseline.
- **Stakeholders**: constitution / governance (`AGENTS.md` v1.12.0) — gates 1–8, `SC-035` (Kotlin mirror only), `SC-051` (engine-failure mechanism preserved), supply-chain rules (`deny.toml`); they decide whether `VERIF-EMULATOR` is admissible given the manifest-edit rule and the closed-set contract.
- **Stakeholders**: `002-fr-024-synthesis-adapter` / synthesis adapter maintainers — `SC-044` (`consent` / cloud TTS) deferred; `001` `gate-8-full` blocked until `002` closes; `003` verification is on the critical path to `SC-044` resolution.
- **[NEEDS CLARIFICATION: Is there an external QA / device-labs team that must approve any emulator-based verification layer? The intake identified only process-internal users; no external stakeholder was named.]**

## Goals

- **G1 (clear, measurable)**: Define whether a local/free emulator layer (`VERIF-EMULATOR`) can provide sufficient structural verification for `003-reference-device` (manifest, `MediaSession`, notification, focus, diag, resume, rate ladder UI) so that developers can confirm those components without physical `adb` access — documented with evidence, not assumed.
- **G2 (measurable)**: Preserve `gate-8-full` as physical-device-only for audio-output / interruption / Bluetooth / `SC-009` / 10-on-device benchmarks — document explicitly which gates emulator can/can't verify, so `verify.sh` and `ROADMAP.md` don't silently claim equivalence.
- **G3 (process)**: Update `003-reference-device` spec/docs (`plan.md`, `spec.md`, `data-model.md`) with the emulator-layer decision (either `VERIF-EMULATOR` added or explicitly rejected) so `003` can move from `Plan/Tasks` to `Implement` or `Kill/Clarify` with evidence.
- **G4 (optional, only if G1 confirms feasibility)**: Add `VERIF-EMULATOR` attestation to `003`, without altering `verify.sh` for physical gates — a second, clearly-labeled verification path.

## Non-Goals

- **NOT building the Android Emulator itself** (official SDK, free, already exists) — the work is defining how it integrates with `003`, not creating new instrumentation.
- **NOT replacing the physical device for `gate-8-full`** — `SC-009` ≤3 s play-to-audio, real `FOREGROUND_SERVICE_MEDIA_PLAYBACK` with hardware audio, real telephony/alarm interruption (`MAY_DUCK` → ~20% vs pause + reason), Bluetooth/car routing (`FR` on device switch), and 10/10 on-device trials require a real device. The emulator is a structural layer at best.
- **NOT solving `SC-044` (consent / cloud TTS)** — `002` adapter remains `offline_default`; `SC-044` deferred; emulator verification of 003 does not unlock `SC-044`. (This was the 80/20 switch decision, not a dependency claim.)
- **NOT changing `Tauri` version** (pinned `2.12.1`) or manifest-edit mechanism (direct XML, not `mobile::update_manifest` which doesn't exist).
- **NOT redesigning the synthesis adapter** (`adapter-synthesis` 12 .rs) or engine-contract (`crates/engine-core` `closed-set`, 7-rate ladder, `EngineError` variants) — emulator doesn't alter synthesis.
- **NOT adding a new crate** unless `cleanroom-core` (currently `0 .rs`) requires it; the verification layer should live in `specs/003-reference-device/` (docs + maybe `tests/`) or `scripts/verify.sh`, not a new crate.
- **NOT inventing users/stakeholders beyond what's verified** — `directory-reading` assessment is blocked at intake; `ressoa-mobile-app` is complete; no external QA team cited.

## Success Metrics

- **Hard (measurable)**: `003-reference-device` has either (a) a documented `VERIF-EMULATOR` layer with explicit boundary conditions (what it verifies / what it doesn't) and `verify.sh` unchanged for physical gates, OR (b) a documented decision that emulator is not useful (with evidence from `research.md` 5 evidence-against points), linking to `problem.md` and `decision.md`.
- **Hard (measurable)**: `specs/003-reference-device/` moves from `Plan/Tasks` to at least `Implement` (or `Kill/Clarify`) with a clear artifact (`problem.md` + `shape.md` + either `plan.md` update or `decision.md`).
- **Qualitative**: Maintain `no Other/Unknown` / `no _ =>` / `expect-only` / `clippy deny` / supply-chain rules across any new verification code; no `unwrap/expect/panic`; no secrets.
- **Baseline / current**: `003` has `plan.md`, `tasks.md`, `checklists/`, `contracts/`, `data-model.md`; `tests/` absent; `verification` absent; `verify.sh` unconfirmed on current branch; `cargo check` interrupted (`bg_14`); only historical `9501bba` (`004`) green.

## Cost of Inaction

- `003-reference-device` remains unverified; `001-android-media-shell` `gate-8-full` stays `DEFERRED`; `005-adopt-design-sona` continues on an unverified base; `004` green (`9501bba`) remains the only attestable build but is historical; new feature work (design-system adoption, synthesis adapter) builds on unverified ground. The risk is not the emulator itself — it's the absence of a verified verification path for `003`, which blocks convergence of `001`.
- Without this problem definition, the 80/20 switch (to 003) risks becoming an open-ended exploration instead of a bounded decision (`VERIF-EMULATOR` yes/no with evidence).

## Open Questions (carried from intake/research; must resolve before decide/shape)

- [NEEDS CLARIFICATION: Which exact emulator? Android SDK AVD (recommended by docs) vs Genymotion vs BlueStack vs Waydroid? Each has different audio/focus/notification fidelity.] 
- [NEEDS CLARIFICATION: Is the emulator a sub-layer (`VERIF-EMULATOR`) alongside physical `gate-8-full`, or a proposed replacement? User's ask is ambiguous; `research.md` notes split in `C3-A`. Decision needed.] 
- [NEEDS CLARIFICATION: What precise success criteria define `VERIF-EMULATOR`? `verify.sh` exit 0 on AVD? Separate attestation document? Benchmark `SC-009` on AVD (infeasible per evidence)?] 
- [NEEDS CLARIFICATION: Is there an external QA/device-labs team that must approve any emulator-based verification? Not cited in intake/research.] 
- [NEEDS CLARIFICATION: Does `cleanroom-core` (0 .rs) need to be filled before any verification layer can be cleanly placed, or can `VERIF-EMULATOR` live in `specs/003/` directly?] 
- [NEEDS CLARIFICATION: What is the CI/hardware budget for running AVD (memory, GPU, disk, time)? The working tree is already dirty with 51 files and `cargo check` interrupted.] 
- [NEEDS CLARIFICATION: Should this remain a separate assessment (`ref-device-emulator`) or be demoted to a note inside `003-reference-device`? The intake created the assessment; reversal requires user confirmation.] 

## Evidence Quality Note

Every claim above is either (a) cited to an internal artifact (`spec.md`, `plan.md`, `tasks.md`, `intake.md`, `research.md`, `ROADMAP.md`, `AGENTS.md`, `constitution.md`, `Cargo.toml`, directory listings), (b) cited to an external high-reliability source (`github.io/spec-kit`, `developer.android.com`, `stackoverflow.com`), or (c) marked `[NEEDS CLARIFICATION]`. No assumption is presented as verified fact. The biggest gap — whether emulator actually accelerates 003 — is explicitly flagged; this definition does not decide it, only frames the problem so the decision can be made with evidence.

# Idea Research: AVD Emulator Setup

- **Slug**: avd-emulator-setup
- **Created**: 2026-11-20
- **Evidence confidence (overall)**: high for environment prerequisites; medium for full boot on the current host

## Users & Demand

- Developers need a reachable emulator to run device-dependent benchmarks/tests/gates for mdreader/Ressoa; without it, SC-009 and structural verification remain blocked — [source: `specs/011-avd-emulator-setup/spec.md:13-50`] (confidence: high)
- A prior assessment (`emulator-setup`, 2025-11-04) already recorded `go` for reusing `mdreader_avd` — [source: `.specify/assessments/emulator-setup/decision.md`] (confidence: high)

## Prior Art

- `specs/008-ref-device-emulator/quickstart.md` defines the structural verification path (manifest, service, notification, focus/ducking, diag JSONL, resume by blake3, rate-step UI) that this setup must enable — [source: spec FR-007, tasks T015–T016] (confidence: high)
- `specs/011-avd-emulator-setup/{avd-config-review.md,emulator-startup-notes.md,research-debug-hooks.md}` capture config review, startup flags (`-gpu swiftshader_indirect -no-boot-anim -no-audio`) and debug-hook research — [source: local artifacts] (confidence: high)

## Market & Context

- Cost of doing nothing: SC-009 and gate 9 remain device-blocked; structural verification cannot be exercised locally.

## Data & Constraints

- Host is x86_64 with KVM support expected; AVD ABI must be x86_64; gate 9 builds aarch64 — ABI mismatch must be documented, not hidden (FR-001, FR-004, FR-008/SC-006) — [source: `spec.md:73-80`] (confidence: high)
- Pure-offline: no INTERNET permission added; no source changes in `crates/*`, `src-tauri/src`, `android-shell/*` — [source: `plan.md:Constitution Check`, spec SC constraints] (confidence: high)
- Ledger: T001–T009 complete (SDK/KVM/AVD/ADB checks, config review, startup attempts, notes); T010–T023 open (build/install/launch, structural verification, VERIF-EMULATOR attestation, docs polish) — [source: `converge.md`, `tasks.md`] (confidence: high)
- `emulator-startup-notes.md` records a boot blocked by absent `/dev/kvm` inside a container; full boot therefore depends on the execution environment — [source: local artifact] (confidence: high)

## Evidence Against the Idea

- Emulator latency measurements are not equivalent to the arm64 reference class; mitigation: explicit caveats on SC-009 results (FR-008, SC-006). — [source: `spec.md:67`] (confidence: high)
- Environment may lack nested virtualization; mitigation: document blocker and keep harness procedures reproducible. — [source: `emulator-startup-notes.md`] (confidence: high)

## Gaps & Open Questions

- Full build/install/launch and end-to-end structural verification on the emulator are environment-gated (T010–T023); non-blocking for documenting the setup.

## Sources

- `specs/011-avd-emulator-setup/{spec.md,plan.md,tasks.md,converge.md,avd-config-review.md,emulator-startup-notes.md}` (host: local repo)
- `specs/008-ref-device-emulator/quickstart.md` (host: local repo)
- `.specify/assessments/emulator-setup/decision.md` (host: local repo)

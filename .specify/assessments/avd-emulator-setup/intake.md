# Idea Intake: AVD Emulator Setup

- **Slug**: avd-emulator-setup
- **Created**: 2026-11-20
- **Source**: `specs/011-avd-emulator-setup/spec.md` (user request: "Set up AVD emulator with appropriate system architecture to run benchmarks, tests, validations and gates necessary to mdreader/Ressoa")
- **Type**: infrastructure / tooling

## Idea (as captured)

> "Set up AVD emulator with appropriate system architecture to run benchmarks, tests, validations and gates necessary to mdreader/Ressoa." — `specs/011-avd-emulator-setup/spec.md:9`

## Restated

Verify and use the existing `mdreader_avd` (API 33 Google APIs x86_64, Pixel 5) on the KVM-capable Linux host: check SDK/ADB/KVM, boot with stable flags, establish ADB connectivity, build/install the app with the right ABI, and execute the structural verification procedures of spec 008 — documenting explicit caveats for emulator-dependent measurements such as SC-009.

## Origin & Context

- **Raised by**: developer environment need, 2025-11-04.
- **Trigger**: device-dependent validations (SC-009, T184/T226 support, gate 9) cannot run without a reachable emulator/device; related prior assessment `emulator-setup` (verdict go) is recorded at `.specify/assessments/emulator-setup/`.

## First-Glance Unknowns

- [NEEDS CLARIFICATION: whether `/dev/kvm` is available in the execution container — resolved in research as environment-dependent; `emulator-startup-notes.md` records a container without `/dev/kvm`, so full boot is environment-gated]

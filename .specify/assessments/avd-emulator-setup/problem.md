# Problem Definition: AVD Emulator Setup

- **Slug**: avd-emulator-setup
- **Created**: 2026-11-20
- **Inputs used**: intake.md | research.md

## Problem Statement

Device-dependent checks (SC-009 latency, gate 9-adjacent install/launch, spec 008 structural verification) have no reproducible local path: although an AVD (`mdreader_avd`), SDK, ADB and KVM-capable host exist, the configuration/boot/connectivity/build/install sequence is not validated end-to-end and emulator-specific measurement caveats are not consistently documented.

## Affected Users & Stakeholders

- **Users**: developers and verification maintainers who need a reachable Android target for tests and gates.
- **Stakeholders**: QA/verification owner (SC-009, spec 008 attestation), maintainer (no source-surface drift).

## Goals

- Verify AVD architecture/API/KVM and hardware-accelerated boot; establish ADB connectivity (FR-001..FR-003).
- Build/install/launch the app with the emulator-compatible ABI and follow spec 008 structural procedures (FR-004..FR-007).
- Document emulator caveats for any SC-009 measurement (FR-008, SC-006).

## Non-Goals

- No product source edits, no new system image, no physical device provisioning, no claim of physical-device performance parity.
- No new INTERNET permission or telemetry.

## Success Metrics

- SC-001..SC-006: AVD boots and ADB connects; artifact builds/installs; app launches; structural flows observable; measurements caveated.
- Documented, repeatable quickstart plus truthful record of environment blockers.

## Cost of Inaction

SC-009 and structural verification stay blocked; device regressions are discovered late on physical hardware only.

## Open Questions

- Full boot requires `/dev/kvm` / nested virtualization; the container where startup was attempted lacked it (T010–T023 environment-gated).
